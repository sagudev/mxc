use std::fmt::Write as FmtWrite;
use std::io::Write as IoWrite;

use console::style;
use indicatif::ProgressStyle;
use mxc::walker::RGE;
use mxc::AudioFile;

pub fn pp_report(rge: &RGE, afs: &[AudioFile]) -> String {
    format!(
        "[{}] {rge}\n{}{}",
        style('✔').green().bold(),
        afs.iter()
            .map(|af| {
                let mut s = if rge.is_album() {
                    af.file.display().to_string() + "\n"
                } else {
                    // on single this will prevent double line printing
                    String::new()
                };
                // we know this two will never panic
                if let Some(track_rg) = af.track_rg {
                    writeln!(s, "{track_rg}").unwrap();
                }
                if let Some(dr_score) = af.dr_score {
                    writeln!(s, "DR14 Score: {dr_score}").unwrap();
                }
                s
            })
            .collect::<String>(),
        if rge.is_album() {
            let mut s = rge.to_string() + "\n";
            // we know this two will never panic
            if let Some(track_rg) = afs[0].album_rg {
                writeln!(s, "{track_rg}").unwrap();
            }
            if let Some(dr_score) = afs[0].album_dr_score {
                writeln!(s, "DR14 Score: {dr_score}").unwrap();
            }
            s
        } else {
            String::new()
        }
    )
}

// from dano
pub fn print_err<S: AsRef<str>>(err_buf: S) {
    // mutex keeps threads from writing over each other
    let err = std::io::stderr();
    let mut err_locked = err.lock();
    err_locked.write_all(err_buf.as_ref().as_bytes()).unwrap();
    err_locked.flush().unwrap();
}

pub fn print_out<S: AsRef<str>>(output_buf: S) {
    // mutex keeps threads from writing over each other
    let out = std::io::stdout();
    let mut out_locked = out.lock();
    out_locked
        .write_all(output_buf.as_ref().as_bytes())
        .unwrap();
    out_locked.flush().unwrap();
}

pub fn progress_style() -> ProgressStyle {
    ProgressStyle::with_template("{wide_msg} {prefix} [{bar:40.cyan/blue}]")
        .unwrap()
        .progress_chars("=> ")
}

pub fn post_album_style() -> ProgressStyle {
    ProgressStyle::with_template("{wide_msg} [{bar:40.green}]")
        .unwrap()
        .progress_chars("=> ")
}

pub fn post_single_style() -> ProgressStyle {
    ProgressStyle::with_template("{wide_msg} {spinner}").unwrap()
}
