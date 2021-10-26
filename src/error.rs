

#[derive(Debug)]
pub enum FinchErrorKind {
    ExpectedFound(char, char),
    Expected(char),
    Unexpected(char),
    MissingPropName,
    InvalidNumber,
    None
}

#[derive(Debug)]
pub struct FinchError(pub FinchErrorKind);

impl std::fmt::Display for FinchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            FinchErrorKind::ExpectedFound(expected, found) => write!(f, "Expected character '{}', but found '{}'", expected, found),
            FinchErrorKind::Expected(expected) => write!(f, "Expected character '{}'", expected),
            FinchErrorKind::None => write!(f, "Expected a character, but found end of file"),
            FinchErrorKind::Unexpected(unexpected) => write!(f, "Unexpected character '{}'", unexpected),
            FinchErrorKind::InvalidNumber => write!(f, "Could not parse number to a 32-bit floating point"),
            FinchErrorKind::MissingPropName => write!(f, "Expected property name after dot (.)")
        }
    }
}

impl std::error::Error for FinchError {}

impl FinchError {

    pub fn none() -> Self {
        Self(FinchErrorKind::None)
    }
}