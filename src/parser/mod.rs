use crate::brewfile::{BrewType, Brewfile, Language, Object};
use std::path::PathBuf;

mod error;
mod lexer;

pub fn parse_brewfile() -> Result<Brewfile, error::BrewfileError> {
    // Read brewfile
    let source = match std::fs::read_to_string(crate::config::BREWFILE_NAME) {
        Ok(source) => source,
        Err(error) => return Err(error::BrewfileError::ReadBrewfileError(error)),
    };

    // Tokenize
    let mut tokens = match compiler::lexer::tokenize(
        source,
        lexer::get_next_token,
        lexer::TokenClass::EndOfFile,
        compiler::lexer::WhitespaceIgnore::AllExceptNewline(lexer::TokenClass::Newline),
    ) {
        Ok(tokens) => tokens,
        Err(error) => return Err(error::BrewfileError::TokenizeError(error)),
    };

    // Parse
    let mut brewfile = Brewfile::new();
    loop {
        let token = tokens.next();
        match token.class() {
            lexer::TokenClass::EndOfFile => return Ok(brewfile),
            lexer::TokenClass::Newline => {}
            lexer::TokenClass::String(command) => {
                let token = tokens.next();
                match token.class() {
                    lexer::TokenClass::Equals => {}
                    lexer::TokenClass::Newline | lexer::TokenClass::EndOfFile => {
                        parse_command(command, Vec::new(), &mut brewfile)?;
                        match token.class() {
                            lexer::TokenClass::EndOfFile => return Ok(brewfile),
                            _ => {}
                        }
                    }
                    _ => {
                        return Err(error::BrewfileError::UnexpectedToken(
                            "equals or newline",
                            token.to_string(),
                        ))
                    }
                }

                let mut parameters = Vec::new();

                let last_token = 'parameter_loop: loop {
                    let token = tokens.next();
                    match token.class() {
                        lexer::TokenClass::String(parameter) => {
                            parameters.push(parameter.to_owned())
                        }
                        _ => {
                            return Err(error::BrewfileError::UnexpectedToken(
                                "parameter",
                                token.to_string(),
                            ))
                        }
                    }

                    let token = tokens.next();
                    match token.class() {
                        lexer::TokenClass::Comma => {}
                        lexer::TokenClass::Newline | lexer::TokenClass::EndOfFile => {
                            break 'parameter_loop token
                        }
                        _ => {
                            return Err(error::BrewfileError::UnexpectedToken(
                                "comma or newline",
                                token.to_string(),
                            ))
                        }
                    }
                };

                parse_command(command, parameters, &mut brewfile)?;

                match last_token.class() {
                    lexer::TokenClass::EndOfFile => return Ok(brewfile),
                    _ => {}
                }
            }
            _ => {
                return Err(error::BrewfileError::UnexpectedToken(
                    "command",
                    token.to_string(),
                ))
            }
        }
    }
}

fn parse_command(
    command: &str,
    parameters: Vec<String>,
    brewfile: &mut Brewfile,
) -> Result<(), error::BrewfileError> {
    match command {
        "name" => {
            if parameters.len() != 1 {
                return Err(error::BrewfileError::InvalidNumberOfParameters(
                    command.to_owned(),
                    1,
                    parameters.len(),
                ));
            }

            brewfile.set_name(parameters.get(0).unwrap().to_owned())?;

            Ok(())
        }
        "type" => {
            if parameters.len() != 1 {
                return Err(error::BrewfileError::InvalidNumberOfParameters(
                    command.to_owned(),
                    1,
                    parameters.len(),
                ));
            }

            let brew_type_str = parameters.get(0).unwrap();
            let brew_type = match brew_type_str.as_str() {
                "executable" => BrewType::Executable,
                "library" => BrewType::Library,
                "group" => BrewType::Group,
                _ => {
                    return Err(error::BrewfileError::UnknownBrewType(
                        brew_type_str.to_owned(),
                    ))
                }
            };

            brewfile.set_brew_type(brew_type)?;

            Ok(())
        }
        "languages" => {
            if parameters.len() == 0 {
                return Err(error::BrewfileError::AtleastParameters(
                    command.to_owned(),
                    1,
                    parameters.len(),
                ));
            }

            for parameter in parameters {
                let language = Language::parse(&parameter)?;
                brewfile.add_language(language)?;
            }

            Ok(())
        }
        "dependencies" => {
            if parameters.len() == 0 {
                return Err(error::BrewfileError::AtleastParameters(
                    command.to_owned(),
                    1,
                    parameters.len(),
                ));
            }

            for parameter in parameters {
                brewfile.add_dependency(parameter)?;
            }

            Ok(())
        }
        "priority" => {
            if parameters.len() == 0 {
                return Err(error::BrewfileError::AtleastParameters(
                    command.to_owned(),
                    1,
                    parameters.len(),
                ));
            }

            for parameter in parameters {
                brewfile.add_priority(PathBuf::from(parameter));
            }

            Ok(())
        }
        _ => {
            if parameters.len() != 3 {
                return Err(error::BrewfileError::InvalidNumberOfParameters(
                    format!("object {}", command),
                    3,
                    parameters.len(),
                ));
            }

            let object_name = command.to_owned();
            let object_language = Language::parse(parameters.get(0).unwrap())?;
            let object_source = parameters.get(1).unwrap().to_owned();
            let install_target = parameters.get(2).unwrap().to_owned();

            brewfile.add_object(Object::new(
                object_name,
                object_language,
                object_source,
                install_target,
            ));

            Ok(())
        }
    }
}
