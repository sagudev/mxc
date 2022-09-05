use std::path::{Path, PathBuf};
use std::str::FromStr;

use mxc::replay_gain::{lufs_to_dbtp, Clipper, ReplayGain};
use mxc::AudioFile;

use crate::options::OutputMode;

#[derive(Debug)]
pub enum Aile {
    Album,
    Track(PathBuf),
}

impl Aile {
    pub fn maybe_path(&self) -> Option<&Path> {
        match self {
            Aile::Album => None,
            Aile::Track(x) => Some(x),
        }
    }
}

impl std::fmt::Display for Aile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Aile::Album => write!(f, "Album"),
            Aile::Track(p) => write!(f, "{}", p.display()),
        }
    }
}

impl PartialEq for Aile {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Track(l0), Self::Track(r0)) => l0.file_name() == r0.file_name(),
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Aile {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        if s.to_lowercase() == "album" {
            Ok(Aile::Album)
        } else {
            Ok(Aile::Track(PathBuf::from_str(s).map_err(D::Error::custom)?))
        }
    }
}

#[derive(Debug, PartialEq)]
#[allow(non_snake_case)]
#[derive(serde::Deserialize)]
pub struct Record {
    pub File: Aile,
    pub Loudness: Num,
    pub Range: Num,
    pub True_Peak: f64,
    pub True_Peak_dBTP: Num,
    pub Reference: Num,
    #[serde(deserialize_with = "from_yn")]
    pub Will_clip: bool,
    #[serde(deserialize_with = "from_yn")]
    pub Clip_prevent: bool,
    pub Gain: Num,
    pub New_Peak: f64,
    pub New_Peak_dBTP: Num,
}

impl Record {
    /// old-style mp3gain-compatible list
    pub fn display_old(&self) {
        println!(
            "{}\t0\t{}\t{}\t0\t0",
            self.File,
            self.Gain,
            self.True_Peak * 32768.0,
        );
    }

    /// output new style list: File;Loudness;Range;Gain;Reference;Peak;Peak dBTP;Clipping;Clip-prevent
    pub fn display_new(&self) {
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.File,
            self.Loudness,
            self.Range,
            self.True_Peak,
            self.True_Peak_dBTP,
            self.Reference,
            yn(self.Will_clip),
            yn(self.Clip_prevent),
            self.Gain,
            self.New_Peak,
            self.New_Peak_dBTP
        );
    }

    /// output something human-readable
    pub fn display_human(&self) {
        println!("{}", self.File);
        println!("Loudness: {}", self.Loudness);
        println!("Range:    {}", self.Range);
        println!("Peak:     {} ({})", self.True_Peak, self.True_Peak_dBTP);
        println!(
            "Gain:     {}{}",
            self.Gain,
            if self.Clip_prevent {
                " (corrected to prevent clipping)"
            } else {
                ""
            }
        )
    }

    pub fn display(&self, outmode: OutputMode) {
        match outmode {
            OutputMode::Human => self.display_human(),
            OutputMode::Old => self.display_old(),
            OutputMode::New => self.display_new(),
        };
    }

    pub fn file(&self) -> Option<&Path> {
        self.File.maybe_path()
    }

    pub fn new(af: &AudioFile, unit: String) -> Self {
        Self::newer(Aile::Track(af.file.clone()), af.track_rg.unwrap(), unit)
    }

    pub fn new_album(rg: ReplayGain, unit: String) -> Self {
        Self::newer(Aile::Album, rg, unit)
    }

    fn newer(file: Aile, rg: ReplayGain, unit: String) -> Self {
        Record {
            File: file,
            Loudness: Num::newr(rg.loudness, "LUFS", 2),
            Range: Num::newr(rg.loudness_range, &unit, 2),
            True_Peak: round(rg.peak, 6),
            True_Peak_dBTP: Num::newr(lufs_to_dbtp(rg.peak), "dBTP", 2),
            Reference: Num::newr(rg.loudness_reference, "LUFS", 2),
            Will_clip: false,
            Clip_prevent: false,
            Gain: Num::newr(rg.gain, &unit, 2),
            New_Peak: 69.42,
            New_Peak_dBTP: Num::empty(),
        }
    }

    pub fn fill(&mut self, clipper: Clipper, unit: String) {
        let (rg, (will_clip, clip_prevented, new_peak)) = clipper;
        self.Will_clip = will_clip;
        self.Clip_prevent = clip_prevented;
        self.Gain = Num::newr(rg.gain, &unit, 2);
        self.New_Peak = round(new_peak, 6);
        self.New_Peak_dBTP = Num::newr(lufs_to_dbtp(new_peak), "dBTP", 2);
    }
}

fn yn(b: bool) -> char {
    if b {
        'Y'
    } else {
        'N'
    }
}

#[derive(Debug, PartialEq)]
pub struct Num {
    numeral: f64,
    unit: String,
}

impl Num {
    pub fn new(numeral: f64, unit: &str) -> Self {
        Self {
            numeral,
            unit: unit.to_owned(),
        }
    }
    pub fn newr(numeral: f64, unit: &str, round_decimals: u32) -> Self {
        Self {
            numeral: round(numeral, round_decimals),
            unit: unit.to_owned(),
        }
    }
    pub fn empty() -> Self {
        Self {
            numeral: 69.42,
            unit: "xx".to_string(),
        }
    }
}

fn round(x: f64, decimals: u32) -> f64 {
    let y = 10i32.pow(decimals) as f64;
    (x * y).round() / y
}

impl std::fmt::Display for Num {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.numeral, self.unit)
    }
}

use serde::de::Error;

impl<'de> serde::Deserialize<'de> for Num {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        let mut wi = s.split_ascii_whitespace();
        if let Some(numeral) = wi.next() {
            Ok(Num {
                numeral: numeral.parse().map_err(D::Error::custom)?,
                unit: wi.next().unwrap_or_default().to_owned(),
            })
        } else {
            Err(D::Error::custom("Empty loudgain number"))
        }
    }
}

fn from_yn<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    if let Some(c) = s.chars().next() {
        match c {
            'Y' | 'y' => Ok(true),
            'N' | 'n' => Ok(false),
            _ => Err(D::Error::custom("Parsing loudgain bool field failed")),
        }
    } else {
        Err(D::Error::custom("Empty loudgain bool field"))
    }
}
