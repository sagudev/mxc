/* Exporting */
mod audiofile;
pub use audiofile::*;
mod error;
pub use error::*;
// here are generic options, that are to be used as lib
pub mod options;
pub mod replay_gain;
pub mod walker;

pub mod version {
    pub use ffmpeg_next::format::version as libavformat_version;
    pub use ffmpeg_next::software::resampling::version as libswr_version;
    pub use taglibxx::{tag_version_major, tag_version_minor, tag_version_patch};
}

// internal modules that are not exported
mod fftag;
mod nerics;
mod seeders;
mod taggers;
