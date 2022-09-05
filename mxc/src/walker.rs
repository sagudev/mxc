use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use log::debug;

use crate::error::WalkerError;

fn is_one_art_folder(folders: &[PathBuf]) -> bool {
    if folders.len() == 1 {
        let folder_name = folders[0]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_ascii_lowercase();
        return folder_name == "cover"
            || folder_name == "covers"
            || folder_name == "arts"
            || folder_name == "art"
            || folder_name == "scans";
    }
    false
}

#[derive(Debug)]
enum Formats {
    Flac,
    Opus,
    Mp3,
    Ogg,
    Oga,
    Spx,
    Mp2,
    M4a,
    Wma,
    Asf,
    Wav,
    Aif,
    Aiff,
    Wv,
    Ape,
}

impl Formats {
    fn args(&self) -> Vec<&str> {
        match self {
            Formats::Flac => vec!["-k", "-s", "e"],
            Formats::Opus => vec!["-k", "-s", "e"],
            Formats::Mp3 => vec!["-I", "3", "-S", "-L", "-k", "-s", "e"],
            // coped from: rgbpm2:
            Formats::Ogg => vec!["-k", "-s", "e"],
            Formats::Oga => vec!["-k", "-s", "e"],
            Formats::Spx => vec!["-k", "-s", "e"],
            Formats::Mp2 => vec!["-I", "3", "-S", "-L", "-k", "-s", "e"],
            Formats::M4a => vec!["-L", "-k", "-s", "e"],
            Formats::Wma => vec!["-L", "-k", "-s", "e"],
            Formats::Asf => vec!["-L", "-k", "-s", "e"],
            Formats::Wav => vec!["-I", "3", "-L", "-k", "-s", "e"],
            Formats::Aif => vec!["-I", "3", "-L", "-k", "-s", "e"],
            Formats::Aiff => vec!["-I", "3", "-L", "-k", "-s", "e"],
            Formats::Wv => vec!["-S", "-k", "-s", "e"],
            Formats::Ape => vec!["-S", "-k", "-s", "e"],
        }
    }
}

impl FromStr for Formats {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "flac" => Self::Flac,
            "ogg" => Self::Ogg,
            "oga" => Self::Oga,
            "spx" => Self::Spx,
            "opus" => Self::Opus,
            "mp2" => Self::Mp2,
            "mp3" => Self::Mp3,
            "m4a" => Self::M4a,
            "wma" => Self::Wma,
            "asf" => Self::Asf,
            "wav" => Self::Wav,
            "aif" => Self::Aif,
            "aiff" => Self::Aiff,
            "wv" => Self::Wv,
            "ape" => Self::Ape,
            _ => return Err(()),
        })
    }
}

fn is_audio_path(path: &Path) -> bool {
    let ext = path
        .extension()
        .unwrap_or_else(|| std::ffi::OsStr::new(""))
        .to_str()
        .unwrap()
        .to_ascii_lowercase();
    // from less to more obscure
    ext == "flac"
        || ext == "opus"
        || ext == "mp3"
        || ext == "ogg"
        || ext == "ape"
        || ext == "m4a"
        || ext == "wav"
        || ext == "oga"
        || ext == "spx"
        || ext == "wma"
        || ext == "asf"
        || ext == "mp2"
        || ext == "aif"
        || ext == "aiff"
        || ext == "wv"
}

pub fn walker(v: &mut Vec<RGE>, path: &Path) -> Result<(), WalkerError> {
    use npath::NormPathExt;
    debug!("Wallked in {path:?}");
    if path.is_file() {
        let path = path.normalized();
        // insert single
        debug!("Single: {path:?}");
        v.push(RGE::Single(path));
    } else if path.is_dir() {
        let entries: Vec<_> = fs::read_dir(path)?.collect::<Result<Vec<_>, std::io::Error>>()?;
        let folders: Vec<_> = entries
            .iter()
            .filter_map(|x| {
                if x.metadata().unwrap().is_dir() {
                    Some(x.path().normalized())
                } else {
                    None
                }
            })
            .collect();
        let audio_files: Vec<_> = entries
            .iter()
            .filter(|x| x.metadata().unwrap().is_file())
            .filter_map(|x| {
                let path = x.path();
                if is_audio_path(&path) {
                    Some(path.normalized())
                } else {
                    None
                }
            })
            .collect();
        if folders.is_empty() || is_one_art_folder(&folders) {
            if !audio_files.is_empty() {
                debug!("Album folder: {path:?}");

                // insert album
                v.push(RGE::Album(audio_files))
            }
        } else {
            for folder in folders {
                walker(v, &folder)?
            }
            // insert singles
            v.extend(audio_files.into_iter().map(|path| {
                debug!("Single: {path:?}");
                RGE::Single(path)
            }))
        }
    } else {
        return Err(WalkerError::QuantumError);
    }
    Ok(())
}

/// This present one ReplayGain unit
///
/// It can ether be Album or single.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RGE {
    Album(Vec<PathBuf>),
    Single(PathBuf),
}

impl PartialOrd for RGE {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // call cmp
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for RGE {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RGE::Album(x) => write!(f, "Album: {}", x[0].parent().unwrap().display()),
            RGE::Single(x) => write!(f, "Single: {}", x.display()),
        }
    }
}

impl RGE {
    /// Get slice of all paths (for single there is only one)
    pub fn sliced(&self) -> &[PathBuf] {
        match self {
            RGE::Album(x) => x,
            RGE::Single(x) => std::slice::from_ref(x),
        }
    }

    pub fn relevant_path(&self) -> &Path {
        match self {
            RGE::Album(x) => x[0].parent().unwrap(),
            RGE::Single(x) => x,
        }
    }

    /// Returns true if [RGE] is album
    pub const fn is_album(&self) -> bool {
        matches!(self, RGE::Album(_))
    }

    /// Run loudgain from path on this RGE unit
    pub fn loudgain(&self) {
        match self {
            RGE::Album(v) => {
                println!("---------------");
                println!("{:?}", v[0]);
                println!("---------------");
                // TODO: Here we assume same formats in album
                let ext = v[0]
                    .extension()
                    .unwrap_or_else(|| std::ffi::OsStr::new(""))
                    .to_str()
                    .unwrap();
                assert!(Command::new("loudgain")
                    .arg("-a")
                    .args(Formats::from_str(ext).unwrap().args().iter())
                    .args(v)
                    .status()
                    .expect("failed to execute process")
                    .success());
            }
            RGE::Single(x) => {
                println!("---------------");
                println!("{x:?}");
                println!("---------------");
                let ext = x
                    .extension()
                    .unwrap_or_else(|| std::ffi::OsStr::new(""))
                    .to_str()
                    .unwrap();
                assert!(Command::new("loudgain")
                    .args(Formats::from_str(ext).unwrap().args().iter())
                    .arg(x)
                    .status()
                    .expect("failed to execute process")
                    .success());
            }
        }
    }
}

fn albumed<P: AsRef<Path>>(p: P) -> PathBuf {
    PathBuf::from(
        p.as_ref()
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_uppercase(),
    )
}

fn singled<P: AsRef<Path>>(p: P) -> PathBuf {
    PathBuf::from(p.as_ref().to_string_lossy().to_uppercase())
}

// hardcore sorting algoritm
// its goal is to provide windows explorer like sorted results
impl Ord for RGE {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            RGE::Album(s) => {
                let s = albumed(&s[0]);
                match other {
                    RGE::Album(o) => s.cmp(&albumed(&o[0])),
                    RGE::Single(o) => ff_cmp(singled(o), s).reverse(),
                }
            }
            RGE::Single(s) => {
                let s = singled(s);
                match other {
                    RGE::Album(o) => ff_cmp(s, albumed(&o[0])),
                    RGE::Single(o) => single_cmp(s, &singled(o)),
                }
            }
        }
    }
}

// folder file cmp
// sameroot:
// less: folder
// great: file
fn ff_cmp<P: AsRef<Path>, R: AsRef<Path>>(file: P, folder: R) -> Ordering {
    let mut file_compnents = file.as_ref().components();
    let fl_len = file_compnents.clone().count() - 1;
    let mut folder_compnents = folder.as_ref().components();
    for (i, (filec, folderc)) in file_compnents
        .by_ref()
        .zip(folder_compnents.by_ref())
        .enumerate()
    {
        let c = filec.cmp(&folderc);
        if c.is_eq() {
            continue;
        } else {
            if i == fl_len {
                return Ordering::Greater;
            }
            return c;
        }
    }
    file.as_ref().cmp(folder.as_ref())
}

// compares two singles
fn single_cmp<P: AsRef<Path>, R: AsRef<Path>>(s1: P, s2: R) -> Ordering {
    let mut s1_compnents = s1.as_ref().components();
    let s1_len = s1_compnents.clone().count() - 1;
    let mut s2_compnents = s2.as_ref().components();
    let s2_len = s2_compnents.clone().count() - 1;
    for (i, (c1, c2)) in s1_compnents.by_ref().zip(s2_compnents.by_ref()).enumerate() {
        let c = c1.cmp(&c2);
        if c.is_eq() {
            continue;
        } else if s1_len == i {
            if s2_len == i {
                break;
            } else {
                return Ordering::Greater;
            }
        } else if s2_len == i {
            return Ordering::Less;
        } else {
            return c;
        }
    }
    s1.as_ref().cmp(s2.as_ref())
}

// sorting tests
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    /// Creates album from one file for tests
    fn album(s: &str) -> RGE {
        RGE::Album(vec![PathBuf::from(s)])
    }

    /// Creates album from one file for tests
    fn single(s: &str) -> RGE {
        RGE::Single(PathBuf::from(s))
    }

    macro_rules! assert_lg {
        ($left:expr, $right:expr $(,)?) => {
            assert!($left < $right);
            assert!($right > $left);
        };
    }

    #[test]
    fn brian_wilson_dir() {
        assert_lg!(
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            album("lol/Brian Wilson/Smile/smile.flac")
        );
        assert_lg!(
            album("lol/Brian Wilson/Smile/smile.flac"),
            single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac")
        );
        assert_lg!(
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac")
        );
        assert_lg!(
            single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac"),
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac")
        );

        assert_lg!(
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac")
        );
        assert_lg!(
            album("lol/Brian Wilson/Smile/smile.flac"),
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac")
        );
        assert_lg!(
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac")
        );

        let mut rges = vec![
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac"),
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac"),
            album("lol/Brian Wilson/Smile/smile.flac"),
        ];
        rges.sort_unstable();
        assert_eq!(
            rges,
            vec![
                // less
                album("lol/Brian Wilson/Brian Wilson/lol.flac"),
                album("lol/Brian Wilson/Smile/smile.flac"),
                single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac"),
                single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac"),
                // greatest
            ]
        );
    }

    #[test]
    fn brian_wilson_and_others_dir() {
        // neighbur
        assert_lg!(
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            album("lol/Brian Wilson/Smile/smile.flac")
        );
        assert_lg!(
            album("lol/Brian Wilson/Smile/smile.flac"),
            single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac")
        );
        assert_lg!(
            single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac"),
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac")
        );
        assert_lg!(
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac"),
            album("lol/The Beach Boys/Smile/smile.flac")
        );

        // 1st
        assert_lg!(
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac")
        );
        assert_lg!(
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac")
        );
        assert_lg!(
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            album("lol/The Beach Boys/Smile/smile.flac")
        );
        assert_lg!(
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            single("lol/abba.flac")
        );
        // 2
        assert_lg!(
            album("lol/Brian Wilson/Smile/smile.flac"),
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac")
        );
        assert_lg!(
            album("lol/Brian Wilson/Smile/smile.flac"),
            album("lol/The Beach Boys/Smile/smile.flac")
        );
        assert_lg!(
            album("lol/Brian Wilson/Smile/smile.flac"),
            single("lol/abba.flac")
        );
        // 3rd
        assert_lg!(
            single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac"),
            album("lol/The Beach Boys/Smile/smile.flac")
        );
        assert_lg!(
            single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac"),
            single("lol/abba.flac")
        );
        //4rd
        assert_lg!(
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac"),
            single("lol/abba.flac")
        );

        println!("\n\n-------------------------------\n");
        let mut rges = vec![
            single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac"),
            single("lol/abba.flac"),
            album("lol/Brian Wilson/Brian Wilson/lol.flac"),
            album("lol/The Beach Boys/Smile/smile.flac"),
            single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac"),
            album("lol/Brian Wilson/Smile/smile.flac"),
        ];
        rges.sort();
        assert_eq!(
            rges,
            vec![
                // less
                album("lol/Brian Wilson/Brian Wilson/lol.flac"),
                album("lol/Brian Wilson/Smile/smile.flac"),
                single("lol/Brian Wilson/Brian Wilson - Right Where I Belong.flac"),
                single("lol/Brian Wilson/Paul Shaffer - Metal Beach.flac"),
                album("lol/The Beach Boys/Smile/smile.flac"),
                single("lol/abba.flac"),
                // greatest
            ]
        );
    }

    #[test]
    fn file_folder() {
        // neighbur
        assert_eq!(ff_cmp("file.flac", "folder/"), Ordering::Greater);
        assert_eq!(ff_cmp("a.flac", "folder/"), Ordering::Greater);
        assert_eq!(ff_cmp("folder/file.flac", "folder/f1"), Ordering::Greater);
    }

    #[test]
    fn file_single() {
        // neighbur
        assert_eq!(
            single_cmp("file.flac", "folder/file.flac"),
            Ordering::Greater
        );
        assert_eq!(single_cmp("a.flac", "folder/file.flac"), Ordering::Greater);
        assert_eq!(
            single_cmp("folder/file.flac", "folder/a.flac"),
            Ordering::Greater
        );
    }
}
