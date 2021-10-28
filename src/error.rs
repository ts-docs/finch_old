

#[derive(Debug)]
pub enum FinchError {
    ExpectedFound(char, char),
    Expected(char),
    Unexpected(char),
    MissingPropName,
    InvalidNumber,
    PropNotExist(String),
    TemplateNotExist(String),
    InvalidArg(i32),
    None
}

impl std::fmt::Display for FinchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExpectedFound(expected, found) => write!(f, "Expected character '{}', but found '{}'", expected, found),
            Self::Expected(expected) => write!(f, "Expected character '{}'", expected),
            Self::None => write!(f, "Expected a character, but found end of file"),
            Self::Unexpected(unexpected) => write!(f, "Unexpected character '{}'", unexpected),
            Self::InvalidNumber => write!(f, "Could not parse number to a 32-bit floating point"),
            Self::MissingPropName => write!(f, "Expected property name after dot (.)"),
            Self::PropNotExist(prop) => write!(f, "Property '{}' does not exist", prop),
            Self::InvalidArg(n) => write!(f, "Argument {} is invalid", n),
            Self::TemplateNotExist(temp_name) => write!(f, "The template {} doesn't exist", temp_name)
        }
    }
}

impl std::error::Error for FinchError {}