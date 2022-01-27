use std::convert::From;
use std::error::Error as ErrorTrait;
use std::ffi::NulError;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::Error as IoError;

#[derive(Debug)]
pub enum Error {
  NullCharacter(NulError),
  OpenLibrary(IoError),
  GetSymbol(IoError),
  NullSymbol,
}

impl ErrorTrait for Error {
  fn cause(&self) -> Option<&dyn ErrorTrait> {
    use self::Error::*;
    match self {
      &NullCharacter(ref val) => Some(val),
      &OpenLibrary(_) | &GetSymbol(_) | &NullSymbol => None,
    }
  }
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> FmtResult {
    use self::Error::*;
    match *self {
      NullCharacter(_) => write!(f, "Unexpected null character in string"),
      OpenLibrary(ref msg) => {
        write!(f, "Failed to open library: {}", msg)
      }
      GetSymbol(ref msg) => {
        write!(f, "Failed to obtain symbol from the library: {}", msg)
      }
      NullSymbol => write!(f, "Symbol is null"),
    }
  }
}

impl From<NulError> for Error {
  fn from(val: NulError) -> Error {
    Error::NullCharacter(val)
  }
}
