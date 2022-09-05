use crate::error::MetaError;
use crate::replay_gain::ReplayGain;

pub struct WriteOptions {
    pub extended: bool,
    pub unit: String,
    pub lowercase: bool,
    pub non_standard_opus: bool,
}

pub trait Tagger {
    fn do_meta(
        &self,
        strip: bool,
        id3v2version: crate::options::Id3v2version,
        write: Option<WriteOptions>,
        track: Option<ReplayGain>,
        album: Option<ReplayGain>,
    ) -> Result<(), MetaError>;
}
