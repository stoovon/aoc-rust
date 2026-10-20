use std::{error::Error, fmt, num::ParseIntError};

#[derive(Debug)]
pub enum SolveError {
    EmptyInput,
    InvalidInput(String),
    InvalidNumber(ParseIntError),
    ArithmeticOverflow,
    MissingDigit { line: usize },
}

impl fmt::Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "puzzle input is empty"),
            Self::InvalidInput(message) => write!(f, "invalid puzzle input: {message}"),
            Self::InvalidNumber(error) => write!(f, "could not parse puzzle number: {error}"),
            Self::ArithmeticOverflow => write!(f, "puzzle arithmetic exceeded the i64 range"),
            Self::MissingDigit { line } => write!(f, "line {line} contains no recognised digit"),
        }
    }
}

impl Error for SolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidNumber(error) => Some(error),
            Self::EmptyInput
            | Self::InvalidInput(_)
            | Self::ArithmeticOverflow
            | Self::MissingDigit { .. } => None,
        }
    }
}

impl From<ParseIntError> for SolveError {
    fn from(error: ParseIntError) -> Self {
        Self::InvalidNumber(error)
    }
}
