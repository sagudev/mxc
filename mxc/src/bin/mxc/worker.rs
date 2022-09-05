use console::style;
use indicatif::{ProgressBar, ProgressIterator};
use mxc::replay_gain::album_rg;
use mxc::walker::RGE;
use mxc::{AudioFile, Error, MetaError};

use crate::options::{DeleteOpts, Opts, Output};
use crate::printe::{post_album_style, post_single_style, pp_report, print_err, print_out};

pub fn delete_on_rge(rge: &RGE, opts: &DeleteOpts) -> Result<(), Error> {
    let mut files = rge
        .sliced()
        .iter()
        .map(AudioFile::new)
        .collect::<Result<Vec<AudioFile>, _>>()?;

    for audio_file in &mut files {
        audio_file.delete_tags(opts.strip_uncommon_tags, opts.id3v2version)?
    }

    Ok(())
}

/// This function "mach" (eng. does) one RGE unit.
/// Whatever that means in the context of RGE.
pub fn mach_rge(rge: &RGE, write: bool, opts: &Opts, pb: Option<ProgressBar>) -> Result<(), Error> {
    match mach_rge_for_real(rge, &pb, opts, write) {
        Ok(files) => {
            match opts.output {
                Output::Tui => {
                    if let Some(p) = pb.as_ref() {
                        p.println(pp_report(rge, &files));
                        p.finish_and_clear();
                    } else {
                        panic!("TUI not working!!!")
                    }
                }
                Output::PrettyPrint => print_out(&pp_report(rge, &files)),
                Output::Log => todo!(),
            };
            Ok(())
        }
        Err(err) => {
            match opts.output {
                Output::Tui => {
                    if let Some(p) = pb.as_ref() {
                        p.println(format!(
                            "[{}] {rge} {}",
                            style('x').red().bold(),
                            style(err.to_string()).red()
                        ));
                        p.finish_and_clear();
                    } else {
                        panic!("TUI not working!!!")
                    }
                }
                Output::PrettyPrint => print_err(format!(
                    "[{}] {rge} {}",
                    style('x').red().bold(),
                    style(err.to_string()).red()
                )),
                Output::Log => todo!(),
            };
            Err(err)
        }
    }
}

// we capture results with this function for pretty printing
fn mach_rge_for_real(
    rge: &RGE,
    pb: &Option<ProgressBar>,
    opts: &Opts,
    write: bool,
) -> Result<Vec<AudioFile>, Error> {
    let mut files = rge
        .sliced()
        .iter()
        .map(AudioFile::new)
        .collect::<Result<Vec<AudioFile>, _>>()?;

    // this is used for progress
    let len = if rge.is_album() {
        Some(files.len())
    } else {
        None
    };

    // seed files
    files.iter_mut().enumerate().try_for_each(|(i, af)| {
        if let Some(p) = pb.as_ref() {
            p.set_length(af.len);
            if let Some(l) = len {
                p.set_prefix(
                    style(format!("[{}/{l}]", i + 1))
                        .yellow()
                        .bold()
                        .to_string(),
                )
            }
        }
        af.seed(
            opts.do_rg(),
            opts.do_dr(),
            opts.allow_corrupted,
            pb.as_ref().map(|p| |pos: u64| p.set_position(pos)),
        )
    })?;

    // do RG
    if opts.do_rg() {
        files
            .iter_mut()
            .try_for_each(|af| af.track_gain(opts.pregain, opts.non_standard_opus))?;
    }

    // do album RG
    let album_rg = if opts.do_album() && opts.do_rg() {
        Some(
            album_rg(&files, opts.pregain)?
                .clipper(opts.maxtpl, !opts.no_clip_prevention)
                .0,
        )
    } else {
        None
    };

    // do album DR
    let album_dr = if opts.do_album() && opts.do_dr() {
        Some(drmeter::DRMeter::dr_score_multiple(
            files.iter().map(|af| af.dr_meter.as_ref().unwrap()),
        )?)
    } else {
        None
    };

    // post-seeding progress bar
    if let Some(p) = pb.as_ref() {
        p.set_length(files.len() as u64);
        if files.len() == 1 {
            // if single we use spinner
            p.set_style(post_single_style())
        } else {
            // if album we use bar
            p.set_style(post_album_style())
        }
        p.tick()
    }

    // now insert albums [AudioFile]s instances
    files
        .iter_mut()
        // in case we do not use tui we just use hidden progressbar that does nothing
        .progress_with(pb.clone().unwrap_or_else(ProgressBar::hidden))
        .try_for_each(|audio_file| -> Result<(), MetaError> {
            // check clipping and maybe prevent it
            if let Some(track_rg) = audio_file.track_rg.as_mut() {
                track_rg.clipper(opts.maxtpl, !opts.no_clip_prevention);
            }

            if opts.do_album() {
                audio_file.album_rg = album_rg;
                audio_file.album_dr_score = album_dr;
            }

            // write tags if requested
            if write {
                audio_file.write_tags(
                    opts.strip_uncommon_tags,
                    opts.id3v2version,
                    true,
                    "dB",
                    opts.lowercase_tags,
                    opts.non_standard_opus,
                )
            } else {
                Ok(())
            }
        })?;

    Ok(files)
}
