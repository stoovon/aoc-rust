use std::{error::Error, fmt, num::ParseIntError};

#[derive(Debug)]
pub enum SolveError {
    EmptyInput,
    InvalidInput(String),
    InvalidNumber(ParseIntError),
    ArithmeticOverflow,
    ResourceLimit(&'static str),
    AtLine { line: usize, error: Box<SolveError> },
    AtStep { step: usize, error: Box<SolveError> },
}

impl SolveError {
    pub fn at_step(self, step: usize) -> Self {
        Self::AtStep {
            step,
            error: Box::new(self),
        }
    }

    pub fn at_line(self, line: usize) -> Self {
        Self::AtLine {
            line,
            error: Box::new(self),
        }
    }
}

impl fmt::Display for SolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => write!(f, "puzzle input is empty"),
            Self::InvalidInput(message) => write!(f, "invalid puzzle input: {message}"),
            Self::InvalidNumber(error) => write!(f, "could not parse puzzle number: {error}"),
            Self::ResourceLimit(message) => write!(f, "resource limit exceeded: {message}"),
            Self::ArithmeticOverflow => write!(f, "puzzle arithmetic exceeded the i64 range"),
            Self::AtLine { line, error } => write!(f, "line {line}: {error}"),
            Self::AtStep { step, error } => write!(f, "step {step}: {error}"),
        }
    }
}

impl Error for SolveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidNumber(error) => Some(error),
            Self::AtLine { error, .. } | Self::AtStep { error, .. } => Some(error.as_ref()),
            Self::EmptyInput
            | Self::InvalidInput(_)
            | Self::ArithmeticOverflow
            | Self::ResourceLimit(_) => None,
        }
    }
}

impl From<ParseIntError> for SolveError {
    fn from(error: ParseIntError) -> Self {
        Self::InvalidNumber(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_context_preserves_the_number_error_chain() {
        let parse_error = "nope".parse::<i64>().unwrap_err();
        let error = SolveError::from(parse_error).at_line(3);
        assert!(error
            .to_string()
            .starts_with("line 3: could not parse puzzle number:"));
        let source = error.source().unwrap();
        assert!(source.downcast_ref::<SolveError>().is_some());
        assert!(source
            .source()
            .unwrap()
            .downcast_ref::<ParseIntError>()
            .is_some());
    }
}
