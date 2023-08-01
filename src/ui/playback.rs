use rodio::{PlayError, StreamError};

#[derive(Debug)]
pub enum AudioError {
    IOError(std::io::Error),
    PlayError(PlayError),
    StreamError(StreamError),
}

impl std::error::Error for AudioError {}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::IOError(err) => write!(f, "{}", err),
            AudioError::PlayError(err) => write!(f, "{}", err),
            AudioError::StreamError(err) => write!(f, "{}", err),
        }
    }
}

impl From<std::io::Error> for AudioError {
    fn from(err: std::io::Error) -> Self {
        AudioError::IOError(err)
    }
}

impl From<PlayError> for AudioError {
    fn from(err: PlayError) -> Self {
        AudioError::PlayError(err)
    }
}

impl From<StreamError> for AudioError {
    fn from(err: StreamError) -> Self {
        AudioError::StreamError(err)
    }
}

impl From<rodio::decoder::DecoderError> for AudioError {
    fn from(err: rodio::decoder::DecoderError) -> Self {
        AudioError::PlayError(PlayError::DecoderError(err))
    }
}
