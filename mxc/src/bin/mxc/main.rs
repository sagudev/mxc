use std::process::exit;

use indicatif::{MultiProgress, ParallelProgressIterator, ProgressBar};
use log::debug;
use mxc::walker::{self, RGE};
use options::Output;
use rayon::prelude::*;

use crate::printe::progress_style;
use crate::worker::{delete_on_rge, mach_rge};

mod options;
mod printe; //rs
mod worker;

fn confirm() -> bool {
    println!("Confirm [y/N]? ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input = input.trim().to_ascii_lowercase();
    input == "y" || input == "j"
}

fn main() -> Result<(), mxc::Error> {
    let opts = options::MxcOptions::parse();

    debug!("{:#?}", opts);

    match opts.command.as_ref().unwrap() {
        options::Command::Help(x) => x.help(),
        options::Command::Delete(d) => {
            let v = walk_and_ask(&d.paths, d.yes, d.output)?;
            build_thread_pool(d.jobs);
            // this is very fast on per file basis
            // so the only relevant progress bar is overall status
            v.par_iter()
                .progress_count(v.len() as u64)
                .with_style(progress_style())
                .try_for_each(|x| delete_on_rge(x, d))
        }
        options::Command::Calc(o) => {
            let v = walk_and_ask(&o.paths, o.yes, o.output)?;
            build_thread_pool(o.jobs);
            let mp = if o.output.is_tui() {
                Some(MultiProgress::new())
            } else {
                None
            };
            v.par_iter()
                .map(|x| {
                    mach_rge(
                        x,
                        false,
                        o,
                        mp.as_ref().map(|m| {
                            m.add(
                                ProgressBar::new(100)
                                    .with_style(progress_style())
                                    .with_message(format!("{x}")),
                            )
                        }),
                    )
                })
                .collect::<Result<Vec<()>, mxc::Error>>()?;
            Ok(())
        }
        options::Command::Write(o) => {
            let v = walk_and_ask(&o.paths, o.yes, o.output)?;
            build_thread_pool(o.jobs);
            let mp = if o.output.is_tui() {
                Some(MultiProgress::new())
            } else {
                None
            };
            v.par_iter()
                .map(|x| {
                    mach_rge(
                        x,
                        true,
                        o,
                        mp.as_ref().map(|m| {
                            m.add(
                                ProgressBar::new(0)
                                    .with_style(progress_style())
                                    .with_message(format!("{x}")),
                            )
                        }),
                    )
                })
                .collect::<Result<Vec<()>, mxc::Error>>()?;
            Ok(())
        }
        options::Command::Version(_) => options::version(),
    }
}

fn walk_and_ask(paths: &[String], yes: bool, output: Output) -> Result<Vec<RGE>, mxc::Error> {
    let spinner = if output.is_tui() {
        Some(
            ProgressBar::new_spinner()
                .with_message("Walking and generating Album/Single structure."),
        )
    } else {
        None
    };
    // Walk the folders and create tree of RGE units
    let mut v = Vec::new();
    for path in paths {
        walker::walker(&mut v, &std::path::PathBuf::from(path)).unwrap();
    }
    // sort to have const results
    v.sort_unstable();
    if let Some(spin) = spinner {
        spin.finish_with_message("Finished generating Album/Single structure.")
    }
    // now print
    if output.is_dynamic() {
        println!("Detected Album/Single structure:");
        for rge in &v {
            println!("{rge}");
        }
    }
    // you sure?
    if yes || output.is_dynamic() && confirm() {
        // do the job, son
        Ok(v)
    } else {
        // User does not trust us :(
        println!("User said no!");
        exit(0)
    }
}

fn build_thread_pool(j: usize) {
    rayon::ThreadPoolBuilder::new()
        .num_threads(j)
        .build_global()
        .unwrap();
    //rayon::ThreadPoolBuilder::new().num_threads(hj).build()
}
