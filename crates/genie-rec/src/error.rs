use std::io;

#[derive(Debug, thiserror::Error)]
pub enum SyncError {
    #[error("Got a sync message, but the log header said there would be a sync message {0} ticks later. The recorded game file may be corrupt")]
    UnexpectedSync(u32),
    #[error("Expected a sync message at this point, the recorded game file may be corrupt")]
    ExpectedSync,
}

/// Errors that may occur while reading a recorded game file.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error(transparent)]
    SyncError(#[from] SyncError),
    #[error(transparent)]
    DecodeStringError(#[from] genie_support::DecodeStringError),
    #[error("Could not read embedded scenario data: {0}")]
    ReadScenarioError(#[from] genie_scx::Error),
    #[error("Failed to parse DE JSON chat message: {0}")]
    DEChatMessageJsonError(#[from] serde_json::Error),
    #[error(
        "Failed to parse DE JSON chat message, JSON is missing the key {0}, or value is invalid"
    )]
    ParseDEChatMessageError(&'static str),
    #[error(
    "Failed to find static marker in recording (expected {1:#x} ({1}), found {2:#x} ({2}), version {0}, {3}:{4}, found next {1:#x} ({1}) {5} bytes further)"
    )]
    MissingMarker(f32, u128, u128, &'static str, u32, u64),
    #[error("Failed parsing header at position {0}: {1}")]
    HeaderError(u64, Box<Error>),
}

impl From<genie_support::ReadStringError> for Error {
    fn from(err: genie_support::ReadStringError) -> Self {
        match err {
            genie_support::ReadStringError::DecodeStringError(inner) => inner.into(),
            genie_support::ReadStringError::IoError(inner) => inner.into(),
        }
    }
}
