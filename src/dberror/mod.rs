extern crate rmp;

use std;
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum DBError<'a> {
    ProtocolError(String),
    FileFormatError(String),
    ParseStringError(rmp::decode::DecodeStringError<'a>),
    ParseValueError(rmp::decode::ValueReadError),
    SendValueError(rmp::encode::ValueWriteError),
    UTF8Error(std::string::FromUtf8Error),
    IOError(std::io::Error),
    SyncError,
}

impl<'a> From<rmp::decode::DecodeStringError<'a>> for DBError<'a> {
    fn from(err: rmp::decode::DecodeStringError<'a>) -> DBError<'a> {
        DBError::ParseStringError(err)
    }
}

impl<'a> From<rmp::decode::ValueReadError> for DBError<'a> {
    fn from(err: rmp::decode::ValueReadError) -> DBError<'a> {
        DBError::ParseValueError(err)
    }
}

impl<'a> From<rmp::encode::ValueWriteError> for DBError<'a> {
    fn from(err: rmp::encode::ValueWriteError) -> DBError<'a> {
        DBError::SendValueError(err)
    }
}

impl<'a> From<std::string::FromUtf8Error> for DBError<'a> {
    fn from(err: std::string::FromUtf8Error) -> DBError<'a> {
        DBError::UTF8Error(err)
    }
}

impl<'a> From<std::io::Error> for DBError<'a> {
    fn from(err: std::io::Error) -> DBError<'a> {
        DBError::IOError(err)
    }
}

impl<'a, T> From<std::sync::PoisonError<T>> for DBError<'a> {
    fn from(_: std::sync::PoisonError<T>) -> DBError<'a> {
        DBError::SyncError
    }
}

impl<'a> fmt::Display for DBError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DBError::ProtocolError(ref err) => write!(f, "Protocol Error: {}", err),
            DBError::FileFormatError(ref err) => write!(f, "FileFormat Error: {}", err),
            DBError::ParseStringError(ref err) => write!(f, "Parse String Error: {}", err),
            DBError::ParseValueError(ref err) => write!(f, "Parse Value Error: {}", err),
            DBError::SendValueError(ref err) => write!(f, "Send Value Error: {}", err),
            DBError::UTF8Error(ref err) => write!(f, "UTF8 Error: {}", err),
            DBError::IOError(ref err) => write!(f, "IO error: {}", err),
            DBError::SyncError => write!(f, "Sync error"),
        }
    }
}

impl<'a> error::Error for DBError<'a> {
    fn description(&self) -> &str {
        match *self {
            DBError::ProtocolError(ref desc) => desc,
            DBError::FileFormatError(ref desc) => desc,
            DBError::ParseStringError(ref err) => err.description(),
            DBError::ParseValueError(ref err) => err.description(), 
            DBError::SendValueError(ref err) => err.description(), 
            DBError::UTF8Error(ref err) => err.description(), 
            DBError::IOError(ref err) => err.description(), 
            DBError::SyncError => "one thread paniced while holding a lock to the db",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DBError::ProtocolError(_) => None,
            DBError::FileFormatError(_) => None,
            DBError::ParseStringError(ref err) => Some(err), 
            DBError::ParseValueError(ref err) => Some(err), 
            DBError::SendValueError(ref err) => Some(err), 
            DBError::UTF8Error(ref err) => Some(err), 
            DBError::IOError(ref err) => Some(err), 
            DBError::SyncError => None,
        }
    }
}
