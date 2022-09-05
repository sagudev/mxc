use std::path::{Path, PathBuf};

use drmeter::DRMeter;
use ebur128::{EbuR128, Mode};

use crate::error::{Error, NError, SeedError};
use crate::fftag::FFtag;
use crate::nerics::Nerics;
use crate::options;
use crate::replay_gain::{track_rg, ReplayGain};
use crate::seeders::{Frame, FrameType, Seeder};
use crate::taggers::{Tagger, WriteOptions};

pub type DRscore = u8;

/// This struct represents one file. Each file has its own:
/// - Tagger (metadata reader)
/// - Filler (decoder; that generates samples from file)
pub struct AudioFile {
    /// FIle that is this struct about
    pub file: PathBuf,

    internal: FFtag,

    /// This is for progress bar
    pub len: u64,

    /// Here lies [EbuR128] instance
    pub ebur: Option<EbuR128>,

    /// Here lies [DRMeter] instance
    pub dr_meter: Option<DRMeter>,

    /// Here are track RG results stored after being calculated
    pub track_rg: Option<ReplayGain>,

    /// Here are album RG results stored after being filled
    pub album_rg: Option<ReplayGain>,

    /// Here is DR score that is available right after seeding (if enabled)
    pub dr_score: Option<DRscore>,

    /// Here is DR score
    pub album_dr_score: Option<DRscore>,
}

pub const NONE: Option<fn(u64)> = None::<fn(u64)>;

impl AudioFile {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, NError> {
        let (internal, len) = crate::fftag::FFtag::new(path.as_ref())?;
        Ok(Self {
            file: path.as_ref().to_path_buf(),
            internal,
            len,
            ebur: None,
            dr_meter: None,
            track_rg: None,
            album_rg: None,
            dr_score: None,
            album_dr_score: None,
        })
    }

    // TODO: make this pipeline async (constantly suck data from ffmpeg and feed it to libs)
    /// Feed all libraries that are needed with data.
    /// Those libraries consumes data on frame basis (so we do not store whole file in memory)
    /// and do their own computations.
    ///
    /// Forced option will allow damaged files
    pub fn seed<F>(
        &mut self,
        ebur: bool,
        dr_meter: bool,
        forced: bool,
        mut progress: Option<F>,
    ) -> Result<(), SeedError>
    where
        F: FnMut(u64),
    {
        let info = self.internal.info();
        if ebur {
            self.ebur = Some(EbuR128::new(
                info.channels,
                info.rate,
                // global loudnes | loudness range
                // EBUR128_MODE_S and EBUR128_MODE_SAMPLE_PEAK are also hidden inside
                Mode::I | Mode::LRA | Mode::TRUE_PEAK,
            )?);
        }
        if dr_meter {
            self.dr_meter = Some(DRMeter::new(info.channels, info.rate)?)
        }
        self.internal.seed(forced, |d, frame| {
            // send progress if required
            if let Some(p) = progress.as_mut() {
                p(d)
            }
            match frame {
                Frame::I16(FrameType::Packed(x)) => {
                    if let Some(e) = self.ebur.as_mut() {
                        e.add_frames_i16(x)?;
                    }
                    if let Some(dr) = self.dr_meter.as_mut() {
                        dr.add_frames_i16(x)?;
                    }
                    Ok(())
                }
                Frame::I32(FrameType::Packed(x)) => {
                    if let Some(e) = self.ebur.as_mut() {
                        e.add_frames_i32(x)?;
                    }
                    if let Some(dr) = self.dr_meter.as_mut() {
                        dr.add_frames_i32(x)?;
                    }
                    Ok(())
                }
                Frame::F32(FrameType::Packed(x)) => {
                    if let Some(e) = self.ebur.as_mut() {
                        e.add_frames_f32(x)?;
                    }
                    if let Some(dr) = self.dr_meter.as_mut() {
                        dr.add_frames_f32(x)?;
                    }
                    Ok(())
                }
                Frame::F64(FrameType::Packed(x)) => {
                    if let Some(e) = self.ebur.as_mut() {
                        e.add_frames_f64(x)?;
                    }
                    if let Some(dr) = self.dr_meter.as_mut() {
                        dr.add_frames_f64(x)?;
                    }
                    Ok(())
                }
                Frame::I16(FrameType::Planar(x)) => {
                    if let Some(e) = self.ebur.as_mut() {
                        e.add_frames_planar_i16(x)?;
                    }
                    if let Some(dr) = self.dr_meter.as_mut() {
                        dr.add_frames_planar_i16(x)?;
                    }
                    Ok(())
                }
                Frame::I32(FrameType::Planar(x)) => {
                    if let Some(e) = self.ebur.as_mut() {
                        e.add_frames_planar_i32(x)?;
                    }
                    if let Some(dr) = self.dr_meter.as_mut() {
                        dr.add_frames_planar_i32(x)?;
                    }
                    Ok(())
                }
                Frame::F32(FrameType::Planar(x)) => {
                    if let Some(e) = self.ebur.as_mut() {
                        e.add_frames_planar_f32(x)?;
                    }
                    if let Some(dr) = self.dr_meter.as_mut() {
                        dr.add_frames_planar_f32(x)?;
                    }
                    Ok(())
                }
                Frame::F64(FrameType::Planar(x)) => {
                    if let Some(e) = self.ebur.as_mut() {
                        e.add_frames_planar_f64(x)?;
                    }
                    if let Some(dr) = self.dr_meter.as_mut() {
                        dr.add_frames_planar_f64(x)?;
                    }
                    Ok(())
                }
            }
        })?;
        // finalize progress
        if let Some(p) = progress.as_mut() {
            p(self.len)
        }
        // finalize and store DR score
        if let Some(dr) = self.dr_meter.as_mut() {
            dr.finalize()?;
            self.dr_score = Some(dr.dr_score()?);
        }
        Ok(())
    }

    pub fn delete_tags(
        &mut self,
        strip: bool,
        id3v2version: options::Id3v2version,
    ) -> Result<(), crate::error::MetaError> {
        self.internal
            .do_meta(strip, id3v2version, None, self.track_rg, self.album_rg)
    }

    pub fn write_tags(
        &mut self,
        strip: bool,
        id3v2version: options::Id3v2version,
        extended: bool,
        unit: &str,
        lowercase: bool,
        non_standard_opus: bool,
    ) -> Result<(), crate::error::MetaError> {
        self.internal.do_meta(
            strip,
            id3v2version,
            Some(WriteOptions {
                extended,
                unit: unit.to_owned(),
                lowercase,
                non_standard_opus,
            }),
            self.track_rg,
            self.album_rg,
        )
    }

    pub fn track_gain(&mut self, mut pregain: f64, non_standard_opus: bool) -> Result<(), Error> {
        if self.track_rg.is_none() {
            if let Some(ebur) = self.ebur.as_ref() {
                if !non_standard_opus && self.internal.is_opus() {
                    pregain -= 5.0;
                }
                self.track_rg = Some(track_rg(ebur, pregain)?);
            } else {
                return Err(Error::NotComputed);
            }
        }
        Ok(())
    }

    pub fn fill_album(&mut self, rg: ReplayGain, non_standard_opus: bool) {
        if !non_standard_opus && self.internal.is_opus() {
            todo!("OPUS albums are not supported")
        } else {
            self.album_rg = Some(rg);
        }
    }
}
