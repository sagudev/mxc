use std::convert::Infallible;
use std::process::exit;
use std::str::FromStr;

use gumdrop::Options;
use mxc::options::Id3v2version;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Output {
    #[default]
    /// Sexy multi printer
    Tui,
    /// Just PrettyPrint nothing very fancy
    PrettyPrint,
    /// Log that you can pipe
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
pub struct MxcOptions {
    /// Show help switch
    #[options(help = "Show help")]
    help: bool,

    // The `command` option will delegate option parsing to the command type,
    // starting at the first free argument.
    #[options(command)]
    pub command: Option<Command>,
}

impl MxcOptions {
    /// This function parse and returns MXC options.
    pub fn parse() -> MxcOptions {
        MxcOptions::parse_args_default_or_exit()
    }
}

// The set of commands and the options each one accepts.
//
// Each variant of a command enum should be a unary tuple variant with only
// one field. This field must implement `Options` and is used to parse arguments
// that are given after the command name.
#[derive(Debug, Options)]
pub enum Command {
    #[options(help = "Show help for a command")]
    Help(HelpOpts),
    #[options(help = "Delete all computed information from files")]
    Delete(DeleteOpts),
    #[options(help = "Do read-only calculation")]
    Calc(Opts),
    #[options(help = "Calculate & write tags")]
    Write(Opts),
    #[options(help = "Show version numbers of underlying libraries")]
    Version(HelpOpts),
}

impl Default for Command {
    fn default() -> Self {
        Self::Calc(Default::default())
    }
}

// Options accepted for the `help` command
#[derive(Debug, Options)]
pub struct HelpOpts {
    #[options(free)]
    free: Vec<String>,
}

impl HelpOpts {
    pub fn help(&self) -> ! {
        if self.free.is_empty() {
            // simulate --help
            println!("{}", MxcOptions::usage());
            println!(
                "Available commands:\n{}",
                MxcOptions::command_list().unwrap()
            );
        } else {
            println!(
                "{}",
                MxcOptions::command_usage(&self.free[0]).expect("Wrong command to show help!")
            );
        }
        exit(0)
    }
}

// Options accepted for the `help` command
#[derive(Debug, Options)]
pub struct DeleteOpts {
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
    #[options(help = "Yes to all (aka. do not question, I trust)")]
    pub yes: bool,

    /// Outputing mode
    #[options(help = "Outputing mode (Tui, PrettyPrint or Log if you want to pipe output)")]
    pub output: Output,

    /// Force stripping tags
    #[options(
        help = "Strip tag types other than ID3v2 from MP2/MP3 files (i.e. ID3v1, APEv2). Strip tag types other than APEv2 from WavPack/APE files (i.e. ID3v1)"
    )]
    pub strip_uncommon_tags: bool,

    /// ID3v2 version
    #[options(
        help = "Write ID3v2.X tags to MP2/MP3/WAV/AIFF files (only 3 and 4 are supported)",
        meta = "X"
    )]
    pub id3v2version: Id3v2version,
}

#[derive(Debug, Options, Default)]
pub struct Opts {
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
    #[options(help = "Yes to all (aka. do not question, I trust)")]
    pub yes: bool,

    /// Outputing mode
    #[options(help = "Outputing mode (Tui, PrettyPrint or Log if you want to pipe output)")]
    pub output: Output,

    /// Calculate DR score
    #[options(help = "Calculate DR14 score")]
    pub dr: bool,

    /// Do not calculate replay gain.
    #[options(help = "Skips feeding Ebur128, but does not produce ReplayGain results")]
    pub no_rg: bool,

    /// Write non-opus standard tags
    #[options(help = "Writes non-standard tags for opus that are commonly used.")]
    pub non_standard_opus: bool,

    // TODO: output to log

    /* Options that nobody should use */
    /// Do not calculate album values (track only)
    #[options(help = "Do not calculate album values (track only)")]
    pub no_album: bool,

    /// Disables clip prevention
    #[options(help = "Disables clipping prevention")]
    pub no_clip_prevention: bool,

    /// Max True peak Level
    #[options(
        help = "Set max true peak level = n dBTP",
        meta = "n",
        default_expr = "-1.0"
    )]
    pub maxtpl: f64,

    /// Pregain
    #[options(
        help = "Apply n dB/LU pre-gain value (use -5 for -23 LUFS target)",
        meta = "n",
        default_expr = "0.0"
    )]
    pub pregain: f64,

    /// Force lowercase tags where possible
    #[options(
        help = "Force lowercase tags (MP2/MP3/MP4/ASF/WMA/WAV/AIFF only). This is non-standard, but sometimes needed"
    )]
    pub lowercase_tags: bool,

    /// Force stripping tags
    #[options(help = "Force doing corrupted files")]
    pub allow_corrupted: bool,

    /// Force stripping tags
    #[options(
        help = "Strip tag types other than ID3v2 from MP2/MP3 files (i.e. ID3v1, APEv2). Strip tag types other than APEv2 from WavPack/APE files (i.e. ID3v1)"
    )]
    pub strip_uncommon_tags: bool,

    /// ID3v2 version
    #[options(
        help = "Write ID3v2.X tags to MP2/MP3/WAV/AIFF files (only 3 and 4 are supported)",
        meta = "X"
    )]
    pub id3v2version: Id3v2version,
}

impl Opts {
    pub const fn do_album(&self) -> bool {
        !self.no_album
    }

    pub const fn do_rg(&self) -> bool {
        !self.no_rg
    }

    pub const fn do_dr(&self) -> bool {
        self.dr
    }
}

pub fn version() -> ! {
    println!("MXC version {VERSION} - using:");
    println!(
        "\tlibtag {}.{}.{}",
        mxc::version::tag_version_major(),
        mxc::version::tag_version_minor(),
        mxc::version::tag_version_patch()
    );
    println!("\tebur128 v0.1.6 based on libebur128 1.2.6");
    let lavf_ver = mxc::version::libavformat_version();
    println!(
        "\tlibavformat {}.{}.{}",
        lavf_ver >> 16,
        lavf_ver >> 8 & 0xff,
        lavf_ver & 0xff
    );
    let swr_ver = mxc::version::libswr_version();
    println!(
        "\tlibswresample {}.{}.{}",
        swr_ver >> 16,
        swr_ver >> 8 & 0xff,
        swr_ver & 0xff
    );
    exit(0)
}
