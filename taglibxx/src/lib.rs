#[cxx::bridge]
pub mod ffi {
    pub struct ReplayGain {
        // This field must be
        pub gain: f64,
        // This is optional in rg1
        pub peak: f64,
        // This two are only used by loudgain
        pub loudness_range: f64,
        pub loudness_reference: f64,
        // This field is not written to files
        pub loudness: f64,
    }

    pub struct Scan {
        file: String,
        track: ReplayGain,
        album: ReplayGain,
    }

    unsafe extern "C++" {
        // One or more headers with the matching C++ declarations. Our code
        // generators don't read it but it gets #include'd and used in static
        // assertions to ensure our picture of the FFI boundary is accurate.
        include!("tagg.h");

        fn tag_write_mp3(scan: Scan, do_album: bool, extended: bool, unit: String, lowercase: bool, strip: bool, id3v2version: i32) -> bool;
        fn tag_clear_mp3(filee: String, strip: bool, id3v2version: i32) -> bool;

        fn tag_write_flac(scan: Scan, do_album: bool, extended: bool, unit: String) -> bool;
        fn tag_clear_flac(filee: String) -> bool;

        fn tag_write_ogg_vorbis(scan: Scan, do_album: bool, extended: bool, unit: String) -> bool;
        fn tag_clear_ogg_vorbis(filee: String) -> bool;

        fn tag_write_ogg_flac(scan: Scan, do_album: bool, extended: bool, unit: String) -> bool;
        fn tag_clear_ogg_flac(filee: String) -> bool;

        fn tag_write_ogg_speex(scan: Scan, do_album: bool, extended: bool, unit: String) -> bool;
        fn tag_clear_ogg_speex(filee: String) -> bool;

        fn tag_write_ogg_opus(scan: Scan, do_album: bool, extended: bool, unit: String) -> bool;
        fn tag_write_ogg_opus_non_standard(scan: Scan, do_album: bool, extended: bool, unit: String) -> bool;
        fn tag_clear_ogg_opus(filee: String) -> bool;

        fn tag_write_mp4(scan: Scan, do_album: bool, extended: bool, unit: String, lowercase: bool) -> bool;
        fn tag_clear_mp4(filee: String) -> bool;

        fn tag_write_asf(scan: Scan, do_album: bool, extended: bool, unit: String, lowercase: bool) -> bool;
        fn tag_clear_asf(filee: String) -> bool;

        fn tag_write_wav(scan: Scan, do_album: bool, extended: bool, unit: String, lowercase: bool, strip: bool, id3v2version: i32) -> bool;
        fn tag_clear_wav(filee: String, strip: bool, id3v2version: i32) -> bool;

        fn tag_write_aiff(scan: Scan, do_album: bool, extended: bool, unit: String, lowercase: bool, strip: bool, id3v2version: i32) -> bool;
        fn tag_clear_aiff(filee: String, strip: bool, id3v2version: i32) -> bool;

        fn tag_write_wavpack(scan: Scan, do_album: bool, extended: bool, unit: String, lowercase: bool, strip: bool) -> bool;
        fn tag_clear_wavpack(filee: String, strip: bool) -> bool;

        fn tag_write_ape(scan: Scan, do_album: bool, extended: bool, unit: String, lowercase: bool, strip: bool) -> bool;
        fn tag_clear_ape(filee: String, strip: bool) -> bool;

        fn tag_version_major() -> i32;
        fn tag_version_minor() -> i32;
        fn tag_version_patch() -> i32;
    }
}

pub use ffi::*;
