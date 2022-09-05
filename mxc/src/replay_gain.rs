use ebur128::{EbuR128, Error};
use log::debug;

use crate::audiofile::AudioFile;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
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

impl std::fmt::Display for ReplayGain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Loudness: {:8.2} LUFS", self.loudness)?;
        writeln!(f, "Range:    {:8.2} dB", self.loudness_range)?;
        writeln!(
            f,
            "Peak:     {:8.6} ({:.2} dBTP)",
            self.peak,
            lufs_to_dbtp(self.peak)
        )?;
        write!(f, "Gain:     {:8.2} dB", self.gain)
    }
}

impl ReplayGain {
    pub fn display(&self, unit: &str) {
        println!("Loudness: {:8.2} LUFS", self.loudness);
        println!("Range:    {:8.2} {unit}", self.loudness_range);
        println!(
            "Peak:     {:8.6} ({:.2} dBTP)",
            self.peak,
            lufs_to_dbtp(self.peak)
        );
        println!("Gain:     {:8.2} {unit}", self.gain);
    }

    /// Detect clip and prevent it if requested
    pub fn clipper(&mut self, max_true_peak_level: f64, prevent: bool) -> Clipper {
        let peak_limit = dbtp_to_lufs(max_true_peak_level);
        // new peak after gain
        let new_peak = dbtp_to_lufs(self.gain) * self.peak;

        if new_peak > peak_limit {
            if prevent {
                let new_new_peak = new_peak.min(peak_limit);
                debug!("Clipping prevented");
                self.gain -= lufs_to_dbtp(new_peak / new_new_peak);
                return (*self, (false, true, new_new_peak));
            }
            (*self, (true, false, new_peak))
        } else {
            (*self, (false, false, new_peak))
        }
    }
}

/// (will_clip, clip_prevented, new_peak)
pub type Clipper = (ReplayGain, (bool, bool, f64));

/// Calculates ReplayGain(2) with -18.00 LUFS
pub fn track_rg(e: &EbuR128, pregain: f64) -> Result<ReplayGain, Error> {
    let global = e.loudness_global()?;
    let range = e.loudness_range()?;
    let peak = (0..e.channels())
        .map(|i| e.true_peak(i))
        // TODO: remove this allocation with try_map
        .collect::<Result<Vec<f64>, Error>>()?
        //
        .into_iter()
        .reduce(f64::max)
        .unwrap();
    //println!("{:#?}", peak);
    //let peak = peak.into_iter().reduce(f64::max).unwrap();

    Ok(ReplayGain {
        gain: lufs_to_rg(global) + pregain,
        peak,
        loudness: global,
        loudness_range: range,
        loudness_reference: lufs_to_rg(-pregain),
    })
}

pub fn album_rg(files: &[AudioFile], pregain: f64) -> Result<ReplayGain, Error> {
    let global = EbuR128::loudness_global_multiple(files.iter().map(|x| x.ebur.as_ref().unwrap()))?;
    let range = EbuR128::loudness_range_multiple(files.iter().map(|x| x.ebur.as_ref().unwrap()))?;

    let peak = files
        .iter()
        .map(|x| x.track_rg.unwrap().peak)
        .reduce(f64::max)
        .unwrap();

    Ok(ReplayGain {
        gain: lufs_to_rg(global) + pregain,
        peak,
        loudness: global,
        loudness_range: range,
        loudness_reference: lufs_to_rg(-pregain),
    })
}

#[inline]
pub fn lufs_to_rg(l: f64) -> f64 {
    -18.0 - l
}

#[inline]
/// The equation to convert to dBTP is: 20 * log10(n)
pub fn lufs_to_dbtp(n: f64) -> f64 {
    20.0 * (n).log10()
}

#[inline]
/// The equation to convert to LUFS is: 10 ** (n / 20.0)
pub fn dbtp_to_lufs(n: f64) -> f64 {
    10.0_f64.powf(n / 20.0)
}

#[inline]
/// Opus stores gain Q7.8
pub fn opus_gain(gain: f64) -> i32 {
    // convert float to Q7.8 number: Q = round(f * 2^8)
    (gain * 256.0).round() as i32 // 2^8 = 256
}
/*
// try_map
pub trait TryMapTrait {
    fn try_map<F, A, B, E, R>(self, func: F) -> Result<R, E>
    where
        Self: Sized + Iterator<Item = A>,
        F: FnMut(A) -> Result<B, E>,
        R: Sized + Iterator<Item = B>,
    {
        self.map(func)
            .collect::<Result<Vec<R>, E>>()
            .map(|x| x.iter())
    }
}

impl<I, T> TryMapTrait for I where I: Sized + Iterator<Item = T> {}

// here lies map_ok

#[derive(Clone)]
pub struct MapOkIterator<I, F> {
    iter: I,
    f: F,
}

impl<A, B, E, I, F> Iterator for MapOkIterator<I, F>
where
    F: FnMut(A) -> B,
    I: Iterator<Item = Result<A, E>>,
{
    type Item = Result<B, E>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| x.map(&mut self.f))
    }
}

pub trait MapOkTrait {
    fn map_ok<F, A, B, E>(self, func: F) -> MapOkIterator<Self, F>
    where
        Self: Sized + Iterator<Item = Result<A, E>>,
        F: FnMut(A) -> B,
    {
        MapOkIterator {
            iter: self,
            f: func,
        }
    }
}

impl<I, T, E> MapOkTrait for I where I: Sized + Iterator<Item = Result<T, E>> {}
*/
