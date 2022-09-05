use crate::error::SeedError;

pub trait Seeder {
    /// Seed ebur
    fn seed<F>(&mut self, forced: bool, f: F) -> Result<(), SeedError>
    where
        F: FnMut(u64, Frame) -> Result<(), SeedError>;
    /// Return true if opus
    ///
    /// Run after seeding as we do not have guarantee that
    /// seeder has this data on init.
    fn is_opus(&self) -> bool;

    fn info(&self) -> AudioInfo;
}

pub struct AudioInfo {
    pub rate: u32,
    pub channels: u32,
}

pub enum FrameType<'a, T> {
    Packed(&'a [T]),
    Planar(&'a [&'a [T]]),
}

pub enum Frame<'a> {
    I16(FrameType<'a, i16>),
    I32(FrameType<'a, i32>),
    F32(FrameType<'a, f32>),
    F64(FrameType<'a, f64>),
}
