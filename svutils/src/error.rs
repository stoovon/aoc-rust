use std::{error::Error, fmt, num::ParseIntError};

#[derive(Debug)]
pub enum SolveError {
    InvalidNumber(ParseIntError),
    ArithmeticOverflow,
}

impl fmt::Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNumber(error) => write!(f, "could not parse puzzle number: {error}"),
            Self::ArithmeticOverflow => write!(f, "puzzle arithmetic exceeded the i64 range"),
        }
    }
}

impl Error for SolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidNumber(error) => Some(error),
            Self::ArithmeticOverflow => None,
        }
    }
}

impl From<ParseIntError> for SolveError {
    fn from(error: ParseIntError) -> Self {
        Self::InvalidNumber(error)
    }
}
