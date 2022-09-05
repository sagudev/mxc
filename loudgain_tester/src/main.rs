use std::path::PathBuf;

use indicatif::ParallelProgressIterator;
mod backend;
mod cli;
mod expect;
mod temp_dir;

use rayon::prelude::*;

use crate::cli::{progress_style, MOutput, Opts};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // get opts
    let opts = Opts::parse();
    // walk the line
    let mut v = Vec::new();
    for a in &opts.paths {
        mxc::walker::walker(&mut v, &PathBuf::from(a))?;
    }
    // sort
    v.sort_unstable();
    // now print
    println!("Detected Album/Single structure:");
    for rge in &v {
        println!("{rge}");
    }

    // do the job
    rayon::ThreadPoolBuilder::new()
        .num_threads(opts.jobs())
        .build_global()?;

    let test_rge = if opts.write {
        crate::backend::test_rge_tag
    } else {
        crate::backend::test_rge_ro
    };

    match opts.output {
        cli::Output::Tui => {
            //let p = ProgressBar::new(v.len() as u64).with_style(progress_style());
            let iter = v
                .par_iter()
                .progress_count(v.len() as u64)
                .with_style(progress_style());

            let p = iter.progress.clone();

            let o = MOutput::Tui(p);
            iter.try_for_each(|rge| test_rge(rge, &o))?;
        }
        cli::Output::PrettyPrint => {
            v.par_iter()
                .try_for_each(|rge| test_rge(rge, &MOutput::PrettyPrint))?;
        }
        cli::Output::Log => {
            v.par_iter()
                .try_for_each(|rge| test_rge(rge, &MOutput::Log))?;
        }
    }

    Ok(())
}
