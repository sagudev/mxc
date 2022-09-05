use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use ffmpeg::format::sample::Type;
use ffmpeg::format::Sample;
use ffmpeg::util::frame::audio::Audio as FAudio;
use ffmpeg::{codec, Error};
use log::{debug, info, trace, warn};
use {ffmpeg_next as ffmpeg, taglibxx as taglib};

use crate::error::{MetaError, SeedError};
use crate::seeders::{Frame, FrameType, Seeder};
use crate::taggers::Tagger;

static FFMPEG_STATE: AtomicUsize = AtomicUsize::new(0);

// There are three different states that we care about: the ffmpeg's
// uninitialized, the ffmpeg's initializing, or the ffmpeg's initialized.
const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

// super safe ffmpeg init
fn maybe_init() -> Result<(), Error> {
    let old_state = match FFMPEG_STATE.compare_exchange(
        UNINITIALIZED,
        INITIALIZING,
        Ordering::SeqCst,
        Ordering::SeqCst,
    ) {
        Ok(s) | Err(s) => s,
    };
    match old_state {
        UNINITIALIZED => {
            debug!("FFMPEG initiating");
            ffmpeg_next::init()?;
            ffmpeg_next::log::set_level(ffmpeg_next::log::Level::Quiet);
            debug!("FFMPEG initialized");
            FFMPEG_STATE.store(INITIALIZED, Ordering::SeqCst);
        }
        INITIALIZING => {
            while FFMPEG_STATE.load(Ordering::SeqCst) == INITIALIZING {
                core::hint::spin_loop()
            }
        }
        _ => { /* No need to do anything */ }
    }
    Ok(())
}

#[allow(clippy::upper_case_acronyms)]
enum AvContainer {
    MP3,
    FLAC,
    OGG,
    MP4,
    ASF,
    WAV,
    WV,
    AIFF,
    APE,
    Unsupported(String),
}

impl AvContainer {
    fn new(s: &str) -> AvContainer {
        // FFmpeg container short names
        match s {
            "mp3" => AvContainer::MP3,
            "flac" => AvContainer::FLAC,
            "ogg" => AvContainer::OGG,
            //"mov" | "mp4" | "m4a" | "3gp" | "3g2" | "mj2" => AvContainer::MP4,
            "mov,mp4,m4a,3gp,3g2,mj2" => AvContainer::MP4,
            "asf" => AvContainer::ASF,
            "wav" => AvContainer::WAV,
            "wv" => AvContainer::WV,
            "aiff" => AvContainer::AIFF,
            "ape" => AvContainer::APE,
            _ => AvContainer::Unsupported(s.to_owned()),
        }
    }
}

/// FFmpeg ang Taglib instance
pub struct FFtag {
    file: String,
    ictx: ffmpeg::format::context::Input,
    input_idx: usize,
    decoder: ffmpeg::codec::decoder::Audio,
    container: AvContainer,
    codec_id: ffmpeg::codec::Id,
}

impl FFtag {
    fn new(path: &Path) -> Result<(Self, u64), Error> {
        maybe_init()?;
        let ictx = ffmpeg::format::input(&path)?;
        let input = ictx
            .streams()
            .best(ffmpeg::media::Type::Audio)
            .ok_or(ffmpeg::Error::StreamNotFound)?;
        let input_idx = input.index();
        let mut decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters())?
            .decoder()
            .audio()?;
        decoder.set_parameters(input.parameters())?;
        let len = input.duration() as u64;
        Ok((
            Self {
                file: path.as_os_str().to_string_lossy().to_string(),
                container: AvContainer::new(ictx.format().name()),
                codec_id: decoder.id(),
                ictx,
                input_idx,
                decoder,
            },
            len,
        ))
    }
}

impl Seeder for FFtag {
    fn seed<F>(&mut self, forced: bool, mut f: F) -> Result<(), SeedError>
    where
        F: FnMut(u64, Frame) -> Result<(), SeedError>,
    {
        let mut sample_type = self.decoder.format();

        let req_resample = match sample_type {
            // empty
            Sample::None => panic!("No samples"),
            // libs cannot handle them so we need to resample
            Sample::U8(_) | Sample::I64(_) => {
                sample_type = Sample::I16(Type::Packed);
                info!("Resampling will be used!");
                true
            }
            // it's fine
            Sample::I16(_) => false,
            Sample::I32(_) => false,
            Sample::F32(_) => false,
            Sample::F64(_) => false,
        };

        /*
            TODO: something that even loudgain does not do
            proper channel layout mapping
            https://www.ffmpeg.org/doxygen/3.0/af__sofalizer_8c_source.html#l00362
            https://www.itu.int/dms_pubrec/itu-r/rec/bs/R-REC-BS.1770-4-201510-I!!PDF-E.pdf
            https://www.itu.int/dms_pubrec/itu-r/rec/bs/R-REC-BS.2051-3-202205-I!!PDF-E.pdf

            mapping:
            https://github.com/jiixyj/loudness-scanner/blob/761dfbe8c2dd9c248e371623654632832e1e54e8/src/ffmpeg_example.c#L72
            offical docs:
            https://www.ffmpeg.org/doxygen/3.0/af__sofalizer_8c_source.html#l00362
            https://www.itu.int/dms_pubrec/itu-r/rec/bs/R-REC-BS.2051-3-202205-I!!PDF-E.pdf#page=15

            but libebur128 and ebur128 berly looked at them
            https://github.com/jiixyj/libebur128/blob/master/ebur128/ebur128.c#L737-L746
            ????
        */
        //e.set_channel_map(&map_channel_map(
        //    decoder.channel_layout(),
        //    decoder.channels(),
        //)).unwrap();

        info!(
            "{} {} channels {}Hz",
            self.file,
            self.decoder.channels(),
            self.decoder.rate()
        );

        debug!("sample: {sample_type:#?}");

        for (packet_stream, packet) in self.ictx.packets() {
            if packet_stream.index() == self.input_idx {
                if let Err(e) = self.decoder.send_packet(&packet) {
                    if !forced {
                        return Err(e.into());
                    }
                    warn!("Error while sending a packet to the decoder {e}");
                    break;
                }
                let mut decoded = FAudio::empty();
                while self.decoder.receive_frame(&mut decoded).is_ok() {
                    if req_resample {
                        let mut resampler = self.decoder.resampler(
                            Sample::I16(Type::Packed),
                            self.decoder.channel_layout(),
                            self.decoder.rate(),
                        )?;
                        let mut resampled = FAudio::empty();
                        resampler.run(&decoded, &mut resampled)?;
                        decoded = resampled;
                    }

                    let planes = decoded.planes();
                    trace!("airplanes: {planes}");
                    debug_assert_eq!(decoded.format(), sample_type);
                    //let d = dts(&decoded);
                    // good enought
                    let d = decoded.pts().unwrap_or_default() as u64;
                    match sample_type {
                        Sample::I16(t) => match t {
                            Type::Packed => f(d, Frame::I16(FrameType::Packed(plane(&decoded, 0)))),
                            Type::Planar => {
                                let l: Vec<_> = (0..planes).map(|x| plane(&decoded, x)).collect();
                                f(d, Frame::I16(FrameType::Planar(&l)))
                            }
                        },
                        Sample::I32(t) => match t {
                            Type::Packed => f(d, Frame::I32(FrameType::Packed(plane(&decoded, 0)))),
                            Type::Planar => {
                                let l: Vec<_> = (0..planes).map(|x| plane(&decoded, x)).collect();
                                f(d, Frame::I32(FrameType::Planar(&l)))
                            }
                        },
                        Sample::F32(t) => match t {
                            Type::Packed => f(d, Frame::F32(FrameType::Packed(plane(&decoded, 0)))),
                            Type::Planar => {
                                let l: Vec<_> = (0..planes).map(|x| plane(&decoded, x)).collect();
                                f(d, Frame::F32(FrameType::Planar(&l)))
                            }
                        },
                        Sample::F64(t) => match t {
                            Type::Packed => f(d, Frame::F64(FrameType::Packed(plane(&decoded, 0)))),
                            Type::Planar => {
                                let l: Vec<_> = (0..planes).map(|x| plane(&decoded, x)).collect();
                                f(d, Frame::F64(FrameType::Planar(&l)))
                            }
                        },

                        Sample::None | Sample::U8(_) | Sample::I64(_) => panic!("should not be"),
                    }?;
                }
            }
        }

        Ok(())
    }

    fn is_opus(&self) -> bool {
        self.codec_id == codec::Id::OPUS
    }

    fn info(&self) -> crate::seeders::AudioInfo {
        crate::seeders::AudioInfo {
            rate: self.decoder.rate(),
            channels: self.decoder.channel_layout().channels() as u32,
        }
    }
}

/*
#[inline]
fn dts(f: &FAudio) -> Option<i64> {
    unsafe {
        match (*f.as_ptr()).pkt_dts {
            ffmpeg::ffi::AV_NOPTS_VALUE => None,
            dts => Some(dts as i64),
        }
    }
}*/

impl From<crate::replay_gain::ReplayGain> for taglib::ReplayGain {
    fn from(x: crate::replay_gain::ReplayGain) -> Self {
        Self {
            gain: x.gain,
            peak: x.peak,
            loudness_range: x.loudness_range,
            loudness_reference: x.loudness_reference,
            loudness: x.loudness,
        }
    }
}

impl Tagger for FFtag {
    fn do_meta(
        &self,
        strip: bool,
        id3v2version: crate::options::Id3v2version,
        write: Option<crate::taggers::WriteOptions>,
        track: Option<crate::replay_gain::ReplayGain>,
        album: Option<crate::replay_gain::ReplayGain>,
    ) -> Result<(), MetaError> {
        use taglib::*;
        if let Some(wopts) = write {
            let scan = Scan {
                file: self.file.clone(),
                track: track.ok_or(MetaError::NotComputed)?.into(),
                album: album.unwrap_or_default().into(),
            };
            let do_album = album.is_some();
            let extended = wopts.extended;
            let unit = wopts.unit;
            let lowercase = wopts.lowercase;
            let non_standard_opus = wopts.non_standard_opus;
            let id3v2version = id3v2version as i32;
            // write tags
            match &self.container {
                AvContainer::MP3 => {
                    if !tag_write_mp3(
                        scan,
                        do_album,
                        extended,
                        unit,
                        lowercase,
                        strip,
                        id3v2version,
                    ) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::FLAC => {
                    if !tag_write_flac(scan, do_album, extended, unit) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::OGG => match self.codec_id {
                    codec::Id::OPUS => {
                        if non_standard_opus {
                            if !tag_write_ogg_opus_non_standard(scan, do_album, extended, unit) {
                                return Err(MetaError::Write(self.file.clone()));
                            }
                        } else if !tag_write_ogg_opus(scan, do_album, extended, unit) {
                            return Err(MetaError::Write(self.file.clone()));
                        }
                    }
                    codec::Id::VORBIS => {
                        if !tag_write_ogg_vorbis(scan, do_album, extended, unit) {
                            return Err(MetaError::Write(self.file.clone()));
                        }
                    }
                    codec::Id::FLAC => {
                        if !tag_write_ogg_flac(scan, do_album, extended, unit) {
                            return Err(MetaError::Write(self.file.clone()));
                        }
                    }
                    codec::Id::SPEEX => {
                        if !tag_write_ogg_speex(scan, do_album, extended, unit) {
                            return Err(MetaError::Write(self.file.clone()));
                        }
                    }
                    _ => return Err(MetaError::Unsupported(self.codec_id.name().to_owned())),
                },
                AvContainer::MP4 => {
                    if !tag_write_mp4(scan, do_album, extended, unit, lowercase) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::ASF => {
                    if !tag_write_asf(scan, do_album, extended, unit, lowercase) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::WAV => {
                    if !tag_write_wav(
                        scan,
                        do_album,
                        extended,
                        unit,
                        lowercase,
                        strip,
                        id3v2version,
                    ) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::WV => {
                    if !tag_write_wavpack(scan, do_album, extended, unit, lowercase, strip) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::AIFF => {
                    if !tag_write_aiff(
                        scan,
                        do_album,
                        extended,
                        unit,
                        lowercase,
                        strip,
                        id3v2version,
                    ) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::APE => {
                    if !tag_write_ape(scan, do_album, extended, unit, lowercase, strip) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::Unsupported(s) => return Err(MetaError::Unsupported(s.clone())),
            }
        } else {
            // delete tags
            match &self.container {
                AvContainer::MP3 => {
                    if !tag_clear_mp3(self.file.clone(), strip, id3v2version as i32) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::FLAC => {
                    if !tag_clear_flac(self.file.clone()) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::OGG => match self.codec_id {
                    codec::Id::OPUS => {
                        if !tag_clear_ogg_opus(self.file.clone()) {
                            return Err(MetaError::Write(self.file.clone()));
                        }
                    }
                    codec::Id::VORBIS => {
                        if !tag_clear_ogg_vorbis(self.file.clone()) {
                            return Err(MetaError::Write(self.file.clone()));
                        }
                    }
                    codec::Id::FLAC => {
                        if !tag_clear_ogg_flac(self.file.clone()) {
                            return Err(MetaError::Write(self.file.clone()));
                        }
                    }
                    codec::Id::SPEEX => {
                        if !tag_clear_ogg_speex(self.file.clone()) {
                            return Err(MetaError::Write(self.file.clone()));
                        }
                    }
                    _ => return Err(MetaError::Unsupported(self.codec_id.name().to_owned())),
                },
                AvContainer::MP4 => {
                    if !tag_clear_mp4(self.file.clone()) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::ASF => {
                    if !tag_clear_asf(self.file.clone()) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::WAV => {
                    if !tag_clear_wav(self.file.clone(), strip, id3v2version as i32) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::WV => {
                    if !tag_clear_wavpack(self.file.clone(), strip) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::AIFF => {
                    if !tag_clear_aiff(self.file.clone(), strip, id3v2version as i32) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::APE => {
                    if !tag_clear_ape(self.file.clone(), strip) {
                        return Err(MetaError::Write(self.file.clone()));
                    }
                }
                AvContainer::Unsupported(s) => return Err(MetaError::Unsupported(s.clone())),
            }
        }
        Ok(())
    }
}

impl crate::nerics::Nerics for FFtag {
    fn new(path: &Path) -> Result<(Self, u64), crate::error::NError>
    where
        Self: Sized,
    {
        Ok(FFtag::new(path)?)
    }
}

/*fn map_channel_map(channels_layout: ChannelLayout, n_channels: u16) -> Vec<ebur128::Channel> {
    let channels_layout = channels_layout.bits();
    todo!()
}*/

/// Fix from https://github.com/zmwangx/rust-ffmpeg/pull/104
#[inline]
fn plane<T: ffmpeg::frame::audio::Sample>(ss: &FAudio, index: usize) -> &[T] {
    if index >= ss.planes() {
        panic!("out of bounds");
    }
    if !<T as ffmpeg::frame::audio::Sample>::is_valid(ss.format(), ss.channels() as u16) {
        panic!("unsupported type");
    }

    if ss.is_planar() {
        unsafe { std::slice::from_raw_parts((*ss.as_ptr()).data[index] as *const T, ss.samples()) }
    } else {
        unsafe {
            std::slice::from_raw_parts(
                (*ss.as_ptr()).data[0] as *const T,
                ss.samples() * usize::from(ss.channels()),
            )
        }
    }
}
