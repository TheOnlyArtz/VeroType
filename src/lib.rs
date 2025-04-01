use buffer::VeroBufReaderError;
use tables::TableEncodingError;
use thiserror::Error;

pub mod buffer;
pub mod tables;

#[derive(Debug, Error)]
pub enum VeroTypeError {
    #[error(transparent)]
    TableEncodingError(#[from] TableEncodingError),

    #[error(transparent)]
    VeroBufReaderError(#[from] VeroBufReaderError)
}
