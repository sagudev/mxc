use thiserror::Error as SuperError;

// tagger
#[derive(SuperError, Debug)]
pub enum MetaError {
    #[error("Couldn't write to: {0}")]
    Write(String),
    #[error("File has something not supported: {0}")]
    Unsupported(String),
    #[error("RG is empty")]
    NotComputed,
    #[error("Internal error: {0}")]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

// seed
#[derive(SuperError, Debug)]
pub enum SeedError {
    #[error("File has something not supported: {0}")]
    Unsupported(String),
    #[error("File has something not supported: {0}")]
    Ebur(#[from] ebur128::Error),
    #[error("File has something not supported: {0}")]
    DRMeter(#[from] drmeter::Error),
    #[error("Internal error: {0}")]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl From<ffmpeg_next::Error> for SeedError {
    fn from(x: ffmpeg_next::Error) -> Self {
        Self::Internal(Box::new(x))
    }
}

//
#[derive(SuperError, Debug)]
pub enum NError {
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Internal error: {0}")]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("File type is not supported: {0}")]
    Unsupported(String),
}

impl From<ffmpeg_next::Error> for NError {
    fn from(x: ffmpeg_next::Error) -> Self {
        Self::Internal(Box::new(x))
    }
}

/// General errors
#[derive(SuperError, Debug)]
pub enum WalkerError {
    #[error("quantum computer is not supported!")]
    QuantumError,
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

/// General errors
#[derive(SuperError, Debug)]
pub enum Error {
    #[error("You did not computed this value")]
    NotComputed,
    #[error("Libebur128 error: {0}")]
    Ebur(#[from] ebur128::Error),
    #[error("DR meter error: {0}")]
    Dr(#[from] drmeter::Error),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Internal error: {0}")]
    Internal(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("File has something not supported: {0}")]
    Unsupported(String),
}

impl From<NError> for Error {
    fn from(x: NError) -> Self {
        match x {
            NError::IO(x) => Error::IO(x),
            NError::Internal(x) => Error::Internal(x),
            NError::Unsupported(x) => Error::Unsupported(x),
        }
    }
}

impl From<SeedError> for Error {
    fn from(x: SeedError) -> Self {
        match x {
            SeedError::Unsupported(x) => Self::Unsupported(x),
            SeedError::Ebur(x) => Self::Ebur(x),
            SeedError::DRMeter(x) => Self::Dr(x),
            SeedError::Internal(x) => Self::Internal(x),
        }
    }
}

impl From<MetaError> for Error {
    fn from(me: MetaError) -> Self {
        match me {
            MetaError::Write(_) => Self::Internal(me.into()),
            MetaError::Unsupported(x) => Self::Unsupported(x),
            MetaError::NotComputed => Self::NotComputed,
            MetaError::Internal(x) => Self::Internal(x),
        }
    }
}
