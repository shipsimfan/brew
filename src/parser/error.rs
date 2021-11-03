#[derive(Debug)]
pub enum Error {
    ReadBrewfileError(std::io::Error),
    UnknownCharacter(char, usize, usize),
    UnexpectedToken(&'static str, String),
    InvalidNumberOfParameters(String, usize, usize),
    AtleastParameters(String, usize, usize),
    UnknownBrewType(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::ReadBrewfileError(error) => format!("Unable to read brewfile ({})", error),
                Error::UnknownCharacter(character, line, column) => format!(
                    "Unknown character '{}' in brewfile at {}:{}",
                    character, line, column
                ),
                Error::UnexpectedToken(expected, actual) => format!(
                    "Expected {}, instead found {} in brewfile",
                    expected, actual
                ),
                Error::InvalidNumberOfParameters(command, expected, actual) => format!(
                    "{} requires {} paramters but {} are specified in brewfile",
                    command, expected, actual
                ),
                Error::UnknownBrewType(brew_type) =>
                    format!("Unknown brew type \"{}\" in brewfile", brew_type),
                Error::AtleastParameters(command, expected, actual) => format!(
                    "{} requires at least {} parameters but {} are specified in brewfile",
                    command, expected, actual
                ),
            }
        )
    }
}
