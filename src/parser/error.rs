#[derive(Debug)]
pub enum BrewfileError {
    ReadBrewfileError(std::io::Error),
    UnknownCharacter(char, usize, usize),
    UnexpectedToken(&'static str, String),
    InvalidNumberOfParameters(String, usize, usize),
    AtleastParameters(String, usize, usize),
    UnknownBrewType(String),
    TokenizeError(Box<dyn std::error::Error>),
    BrewfileError(crate::brewfile::error::Error),
}

impl std::error::Error for BrewfileError {}

impl std::fmt::Display for BrewfileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BrewfileError::ReadBrewfileError(error) =>
                    format!("Unable to read brewfile ({})", error),
                BrewfileError::UnknownCharacter(character, line, column) => format!(
                    "Unknown character '{}' in brewfile at {}:{}",
                    character, line, column
                ),
                BrewfileError::UnexpectedToken(expected, actual) => format!(
                    "Expected {}, instead found {} in brewfile",
                    expected, actual
                ),
                BrewfileError::InvalidNumberOfParameters(command, expected, actual) => format!(
                    "{} requires {} paramters but {} are specified in brewfile",
                    command, expected, actual
                ),
                BrewfileError::UnknownBrewType(brew_type) =>
                    format!("Unknown brew type \"{}\" in brewfile", brew_type),
                BrewfileError::AtleastParameters(command, expected, actual) => format!(
                    "{} requires at least {} parameters but {} are specified in brewfile",
                    command, expected, actual
                ),
                BrewfileError::TokenizeError(error) => format!("{}", error),
                BrewfileError::BrewfileError(error) => format!("{}", error),
            }
        )
    }
}

impl From<crate::brewfile::error::Error> for BrewfileError {
    fn from(error: crate::brewfile::error::Error) -> Self {
        BrewfileError::BrewfileError(error)
    }
}
