use std::path::Path;

use crate::error::NError;
use crate::seeders::Seeder;
use crate::taggers::Tagger;

pub trait Nerics: Tagger + Seeder {
    fn new(path: &Path) -> Result<(Self, u64), NError>
    where
        Self: Sized;
}
