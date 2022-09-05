use std::convert::Infallible;
use std::io::Write;
use std::str::FromStr;

use console::{style, Color};
use gumdrop::Options;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Output {
    #[default]
    /// Sexy multi printer
    Tui,
    /// no progress bars
    PrettyPrint,
    /// no colors
    Log,
}

impl Output {
    /// Can user respond to us
    pub const fn is_dynamic(&self) -> bool {
        matches!(self, Self::Tui | Self::PrettyPrint)
    }

    pub const fn is_tui(&self) -> bool {
        matches!(self, Self::Tui)
    }

    pub const fn is_pp(&self) -> bool {
        matches!(self, Self::PrettyPrint)
    }

    pub const fn is_log(&self) -> bool {
        matches!(self, Self::Log)
    }
}

#[derive(Debug, Clone)]
pub enum MOutput {
    /// Sexy multi printer
    Tui(ProgressBar),
    /// no progress bars
    PrettyPrint,
    /// no colors
    Log,
}

impl MOutput {
    /// Can user respond to us
    pub const fn is_dynamic(&self) -> bool {
        matches!(self, Self::Tui(_) | Self::PrettyPrint)
    }

    pub const fn is_tui(&self) -> bool {
        matches!(self, Self::Tui(_))
    }

    pub const fn is_pp(&self) -> bool {
        matches!(self, Self::PrettyPrint)
    }

    pub const fn is_log(&self) -> bool {
        matches!(self, Self::Log)
    }

    pub fn print_out<S: AsRef<str>>(&self, status: char, color: Color, msg: S) {
        match self {
            MOutput::Tui(p) => {
                p.println(format!("[{}] {}\n", style(status).fg(color), msg.as_ref()))
            }
            MOutput::PrettyPrint => {
                print_out(format!("[{}] {}\n", style(status).fg(color), msg.as_ref()))
            }
            MOutput::Log => print_out(format!("[{status}] {}\n", msg.as_ref())),
        }
    }

    pub fn print_double<S: AsRef<str>>(&self, status1: char, status2: char, color: Color, msg: S) {
        match self {
            MOutput::Tui(p) => p.println(format!(
                "[{}{}] {}\n",
                style(status1).fg(color),
                style(status2).fg(color),
                msg.as_ref()
            )),
            MOutput::PrettyPrint => print_out(format!(
                "[{}{}] {}\n",
                style(status1).fg(color),
                style(status2).fg(color),
                msg.as_ref()
            )),
            MOutput::Log => print_out(format!("[{status1}{status2}] {}\n", msg.as_ref())),
        }
    }

    pub fn print_err<S: AsRef<str>>(&self, status: char, color: Color, msg: S) {
        match self {
            MOutput::Tui(p) => {
                p.println(format!("[{}] {}\n", style(status).fg(color), msg.as_ref()))
            }
            MOutput::PrettyPrint => {
                print_err(format!("[{}] {}\n", style(status).fg(color), msg.as_ref()))
            }
            MOutput::Log => print_err(format!("[{status}] {}\n", msg.as_ref())),
        }
    }
}

impl FromStr for Output {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "t" | "tui" | "ui" => Ok(Self::Tui),
            "p" | "pp" | "sexy" | "prettyprint" => Ok(Self::PrettyPrint),
            "l" | "log" => Ok(Self::Log),
            _ => panic!("Wrong option provided"),
        }
    }
}

/// MXC computes different properties of music files (for example ReplayGain 2, DR score) and then print or store them.
#[derive(Debug, Options)]
pub struct Opts {
    /// Show help switch
    #[options(help = "Show help")]
    help: bool,

    /// Contains paths to be scanned
    ///
    /// Folders are walked, files are threated as singles
    #[options(free)]
    pub paths: Vec<String>,

    /// Max number of parallel jobs
    ///
    /// We use half of this for global (for album) pool and half for RGE pool.
    #[options(help = "Max number of parallel jobs", default_expr = "num_cpus::get()")]
    pub jobs: usize,

    /// No questions asked
    #[options(help = "I we are allowed to write (to temp)")]
    pub write: bool,

    /// Outputing mode
    #[options(help = "Outputing mode (Tui, PrettyPrint or Log if you want to pipe output)")]
    pub output: Output,
}

impl Opts {
    /// This function parse and returns MXC options.
    pub fn parse() -> Opts {
        Self::parse_args_default_or_exit()
    }

    pub const fn jobs(&self) -> usize {
        (self.jobs + 1) / 2
    }
}

// from dano
pub fn print_err<S: AsRef<str>>(err_buf: S) {
    // mutex keeps threads from writing over each other
    let err = std::io::stderr();
    let mut err_locked = err.lock();
    err_locked.write_all(err_buf.as_ref().as_bytes()).unwrap();
    err_locked.flush().unwrap();
}

#[macro_export]
macro_rules! eeprintln {
    () => {
        $crate::cli::print_err("\n")
    };
    ($($arg:tt)*) => {{
        $crate::cli::print_err(format!($($arg)*));
    }};
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

#[macro_export]
macro_rules! pprintln {
    () => {
        $crate::print_out("\n")
    };
    ($($arg:tt)*) => {{
        $crate::print_out(format!($($arg)*));
    }};
}

pub fn progress_style() -> ProgressStyle {
    ProgressStyle::with_template("{wide_msg} [{bar:40.cyan/blue}]")
        .unwrap()
        .progress_chars("=> ")
}
