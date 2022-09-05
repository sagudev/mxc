use log::warn;
use mxc::replay_gain::{album_rg, ReplayGain};
use mxc::AudioFile;
use options::LoudgainOpts;
use record::Record;

pub mod options;
pub mod record;

//#[deprecated(note = "This function is only to be used here only for loudgain_tester")]
/// DO NOT USE THIS FUNCTION!!!
///
/// This function is here only loudgain_tester
pub fn loudgain(opts: LoudgainOpts, total_quiet: bool) -> Vec<Record> {
    if !total_quiet {
        match opts.output {
            options::OutputMode::Human => println!("Scanning all files."),
            options::OutputMode::Old => println!("File\tMP3 gain\tdB gain\tMax Amplitude\tMax global_gain\tMin global_gain"),
            options::OutputMode::New => println!("File\tLoudness\tRange\tTrue_Peak\tTrue_Peak_dBTP\tReference\tWill_clip\tClip_prevent\tGain\tNew_Peak\tNew_Peak_dBTP"),
        };
    }

    let mut files = opts
        .files
        .iter()
        .map(AudioFile::new)
        .collect::<Result<Vec<AudioFile>, _>>()
        .unwrap();

    files
        .iter_mut()
        // loudgain allows damaged files
        .try_for_each(|x| x.seed(true, false, true, mxc::NONE))
        .unwrap();

    files
        .iter_mut()
        .try_for_each(|x| x.track_gain(opts.pre_gain, false))
        .unwrap();

    let mut records: Vec<Record> = files
        .iter()
        .map(|x| Record::new(x, opts.unit.clone()))
        .collect();

    let album: Option<ReplayGain> = if opts.do_album {
        let mut album_rg = album_rg(&files, opts.pre_gain).unwrap();
        records.push(Record::new_album(album_rg, opts.unit.clone()));
        records.last_mut().unwrap().fill(
            album_rg.clipper(opts.max_true_peak_level, opts.clip_prevention),
            opts.unit.clone(),
        );
        Some(album_rg)
    } else {
        None
    };

    for (audio_file, record) in files.iter_mut().zip(records.iter_mut()) {
        // check clipping and maybe prevent it
        record.fill(
            audio_file
                .track_rg
                .as_mut()
                .unwrap()
                .clipper(opts.max_true_peak_level, opts.clip_prevention),
            opts.unit.clone(),
        );
        audio_file.album_rg = album;

        // do requested stuff on file
        match opts.mode {
            options::Mode::WriteExtended => audio_file
                .write_tags(
                    opts.strip,
                    opts.id3v2version,
                    true,
                    &opts.unit,
                    opts.lowercase,
                    false,
                )
                .unwrap(),
            options::Mode::Write => audio_file
                .write_tags(
                    opts.strip,
                    opts.id3v2version,
                    false,
                    &opts.unit,
                    opts.lowercase,
                    false,
                )
                .unwrap(),
            options::Mode::Noop => { /* no-op */ }
            options::Mode::Delete => audio_file
                .delete_tags(opts.strip, opts.id3v2version)
                .unwrap(),
        }

        if !total_quiet {
            record.display(opts.output);
            if opts.warn_clip && record.Will_clip {
                warn!("This track will clip!");
            }
        }

        //println!("{:#?}", audio_file.track_rg.unwrap());
    }
    // print album for last
    if !total_quiet && album.is_some() {
        records.last().unwrap().display(opts.output);
        if opts.warn_clip && records.last().unwrap().Will_clip {
            warn!("This album will clip!");
        }
    }

    records
}
