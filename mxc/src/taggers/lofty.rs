use crate::tagger::Tagger;

pub struct Lofty {}

impl Lofty {
    /// Creates a new [`Lofty`].
    pub fn new() -> Self {
        Self {}
    }
}

impl Tagger for Lofty {}
