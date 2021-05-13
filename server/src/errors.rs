use thiserror::Error;
use std::str::Utf8Error;
use std::string::FromUtf8Error;
use std::num::ParseIntError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("ProtocolError")]
    ProtocolError,
    #[error("IOError: {0}")]
    IOError(#[from] std::io::Error),
    #[error("CodecError: {0}")]
    CodecError(#[from] Utf8Error),
    #[error("FromUtf8Error: {0}")]
    FromUtf8Error(#[from] FromUtf8Error),
    #[error("ParseIntError: {0}")]
    ParseIntError(#[from] ParseIntError),
}