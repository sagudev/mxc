use std::io::Write;
use std::path::{Path, PathBuf};

use loudgainer::record::Record;
use mxc::walker::RGE;

use crate::cli::{print_out, MOutput};
use crate::expect::{ExpectOk, ExpectResult};
use crate::temp_dir::MaybeTempDir;
use crate::{expect_eq, expect_eq_delta};

fn sha256_digest<P: AsRef<Path>>(path: P) -> String {
    use std::fs::File;
    use std::io::{BufReader, Read};

    use data_encoding::HEXUPPER;
    use ring::digest::{Context, SHA256};

    let input = File::open(path).unwrap();
    let mut reader = BufReader::new(input);

    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = reader.read(&mut buffer).unwrap();
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }

    HEXUPPER.encode(context.finish().as_ref())
}

fn loudgain(rge: &RGE, tag: bool) -> Result<Vec<Record>, Box<dyn std::error::Error + Sync + Send>> {
    let mut args = vec!["--output-new", "--quiet", "--noclip"];
    if rge.is_album() {
        args.push("-a");
    }
    if tag {
        args.extend_from_slice(&["-s", "e"])
    }
    let output = loudgain::loudgain()
        .args(args)
        .args(rge.sliced())
        .output()
        .expect("failed to execute process");

    std::io::stderr().write_all(&output.stderr)?;
    //std::io::stdout().write_all(&output.stdout).unwrap();
    assert!(output.status.success());

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .has_headers(true)
        .from_reader(&*output.stdout);
    let out: Vec<Record> = rdr.deserialize().collect::<Result<_, _>>()?;

    Ok(out)
}

fn loudgainer(
    rge: &RGE,
    tag: bool,
) -> Result<Vec<Record>, Box<dyn std::error::Error + Sync + Send>> {
    use loudgainer::options::LoudgainOpts;
    let opts: LoudgainOpts = LoudgainOpts {
        files: rge.sliced().to_vec(),
        clip_prevention: true,
        do_album: rge.is_album(),
        mode: if tag {
            loudgainer::options::Mode::WriteExtended
        } else {
            Default::default()
        },
        ..Default::default()
    };
    Ok(loudgainer::loudgain(opts, true))
}

fn expect(lr: &Record, la: &Record) -> ExpectResult {
    expect_eq!(lr.File, la.File)?;
    expect_eq!(lr.Loudness, la.Loudness)?;
    expect_eq!(lr.Range, la.Range)?;
    let d1 = expect_eq_delta!(lr.True_Peak, la.True_Peak, 0.000_002)?;
    expect_eq!(lr.True_Peak_dBTP, la.True_Peak_dBTP)?;
    expect_eq!(lr.Reference, la.Reference)?;
    expect_eq!(lr.Will_clip, la.Will_clip)?;
    expect_eq!(lr.Clip_prevent, la.Clip_prevent)?;
    expect_eq!(lr.Gain, la.Gain)?;
    let d2 = expect_eq_delta!(lr.New_Peak, la.New_Peak, 0.000_002)?;
    expect_eq!(lr.New_Peak_dBTP, la.New_Peak_dBTP)?;
    if let ExpectOk::WithDelta(dd1) = d1 {
        if let ExpectOk::WithDelta(dd2) = d2 {
            // if both have delta use the biggest (by absolute value)
            Ok(ExpectOk::WithDelta(
                *[dd1, dd2]
                    .iter()
                    .max_by(|a, b| a.abs().total_cmp(&b.abs()))
                    .unwrap(),
            ))
        } else {
            Ok(d1)
        }
    } else {
        Ok(d2)
    }
}

/// Read only
pub fn test_rge_ro(
    rge: &RGE,
    output: &MOutput,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let MOutput::Tui(p) = output {
        p.set_message(rge.to_string())
    } else {
        print_out(rge.to_string())
    }
    let rrge = rge.clone();
    // we spawn loudgainer in thread
    let thread_join_handle = std::thread::spawn(move || loudgainer(&rrge, false));
    // do loudgain in this thread
    let loudgain = loudgain(rge, false)?;
    // and now join loudgainer thread
    let loudgainer = thread_join_handle.join().unwrap()?;
    for (lr, la) in loudgainer.iter().zip(loudgain.iter()) {
        let file = if let Some(lr_file) = lr.file() {
            lr_file.display().to_string()
        } else {
            "Album".to_string()
        };
        match expect(lr, la) {
            Ok(ExpectOk::Exact) => output.print_out('✔', console::Color::Green, &file),
            Ok(ExpectOk::WithDelta(d)) => output.print_out(
                'Δ',
                console::Color::Yellow,
                format!("{file} with delta {d}"),
            ),
            Err(e) => {
                output.print_err(
                    '✗',
                    console::Color::Red,
                    format!("{file} ERROR: assertion failed: {e}"),
                );
                return Err(e.into());
            }
        }
    }
    expect_eq!(loudgainer.len(), loudgain.len())?;
    Ok(())
}

pub fn test_rge_tag(
    rge: &RGE,
    output: &MOutput,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let MOutput::Tui(p) = output {
        p.set_message(rge.to_string())
    } else {
        print_out(rge.to_string())
    }
    // prepare temp dirs
    let (mut tmp1_dir, temp1_rge) = temp(rge)?;
    let (mut tmp2_dir, temp2_rge) = temp(rge)?;
    // spawn loudgainer in thread
    let thread_join_handle = std::thread::spawn(move || loudgainer(&temp2_rge, true));
    // do Loudgain in this thread
    let loudgain = loudgain(&temp1_rge, true)?;
    // and now join loudgainer thread
    let loudgainer = thread_join_handle.join().unwrap()?;

    // check if album results have delta
    let album_delta = if rge.is_album() {
        matches!(
            expect(loudgainer.last().unwrap(), loudgain.last().unwrap()),
            Ok(ExpectOk::WithDelta(_)),
        )
    } else {
        false
    };

    for (lr, la) in loudgainer.iter().zip(loudgain.iter()) {
        let file = if let Some(lr_file) = lr.file() {
            lr_file.display().to_string()
        } else {
            "Album".to_string()
        };

        match expect(lr, la) {
            Ok(ExpectOk::Exact) => {
                if let Some(lr_file) = lr.file() {
                    match expect_eq!(sha256_digest(lr_file), sha256_digest(la.file().unwrap())) {
                        Ok(_) => output.print_double('✔', '✔', console::Color::Green, &file),
                        Err(e) => {
                            // if album results have delta we can expect checksum mismatch
                            if !album_delta {
                                output.print_double(
                                    '✗',
                                    '✔',
                                    console::Color::Red,
                                    format!("{file}\nChecksum failed: {e}"),
                                );
                                return Err(e.into());
                            } else {
                                output.print_out('✔', console::Color::Green, &file)
                            }
                        }
                    }
                } else {
                    // album
                    output.print_out('✔', console::Color::Green, &file)
                }
            }
            Ok(ExpectOk::WithDelta(d)) => output.print_out(
                'Δ',
                console::Color::Yellow,
                format!("{file} with delta {d}"),
            ),
            Err(e) => {
                output.print_err(
                    '✗',
                    console::Color::Red,
                    format!("{file} ERROR: assertion failed: {e}"),
                );
                return Err(e.into());
            }
        }
    }
    expect_eq!(loudgainer.len(), loudgain.len())?;
    // do not keep temp files
    tmp1_dir.keep = false;
    tmp2_dir.keep = false;
    Ok(())
}

fn temp(rge: &RGE) -> Result<(MaybeTempDir, RGE), std::io::Error> {
    match rge {
        RGE::Album(xx) => {
            let tmp_dir = MaybeTempDir::new(
                tempfile::Builder::new()
                    .prefix(&xx[0].parent().unwrap().components().last().unwrap())
                    .rand_bytes(6)
                    .tempdir()?,
                true,
            );
            let tmps: Vec<PathBuf> = xx
                .iter()
                .map(|x| tmp_dir.as_ref().join(x.file_name().unwrap()))
                .collect();
            for (src, dest) in xx.iter().zip(tmps.iter()) {
                std::fs::copy(src, dest)?;
            }
            Ok((tmp_dir, RGE::Album(tmps)))
        }
        RGE::Single(x) => {
            let tmp_dir = MaybeTempDir::new(
                tempfile::Builder::new()
                    .prefix(&x.file_stem().unwrap())
                    .rand_bytes(6)
                    .tempdir()?,
                true,
            );
            let tmp = tmp_dir.as_ref().join(x.file_name().unwrap());
            std::fs::copy(x, &tmp)?;
            Ok((tmp_dir, RGE::Single(tmp)))
        }
    }
}
