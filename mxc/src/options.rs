use std::string::ParseError;

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Id3v2version {
    V3 = 3,
    #[default]
    V4 = 4,
}

impl std::str::FromStr for Id3v2version {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_ascii_lowercase().chars().next().unwrap() {
            '3' => Ok(Self::V3),
            '4' => Ok(Self::V4),
            _ => panic!("Invalid ID3v2 version; only 3 and 4 are supported."),
        }
    }
}
