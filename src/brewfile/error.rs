use std::path::PathBuf;

use super::Language;

#[derive(Debug)]
pub enum Error {
    NoBrewType,
    NameDefinedTwice,
    BrewTypeDefinedTwice,
    LanguageDefinedTwice(Language),
    DependencyDefinedTwice(String),
    UnknownLanguage(String),
    DirectoryCreationError(PathBuf, std::io::Error),
    DirectoryReadError(PathBuf, std::io::Error),
    UncompiledFile(PathBuf),
    RemoveObjectsDirectoryError(std::io::Error),
    RemoveTargetError(String, std::io::Error),
    RunCompilerError(&'static str, std::io::Error),
    CompileError(PathBuf),
    RunLinkerError(std::io::Error),
    LinkerError,
    RunBrewError(std::io::Error),
    BrewError(PathBuf),
    NoName,
    InstallTargetError(String, std::io::Error),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::NoBrewType => format!("No brew type specified in brewfile"),
                Error::NameDefinedTwice =>
                    format!("Attempting to specify more than one name in brewfile"),
                Error::BrewTypeDefinedTwice =>
                    format!("Attempting to specificy for than one brew type in brewfile"),
                Error::LanguageDefinedTwice(language) => format!(
                    "Attempting to specify language '{}' twice in brewfile",
                    language
                ),
                Error::DependencyDefinedTwice(dependency) => format!(
                    "Attempting to specify dependency '{}' twice in brewfile",
                    dependency
                ),
                Error::UnknownLanguage(language) =>
                    format!("Unknown language \"{}\" in brewfile", language),
                Error::DirectoryCreationError(path, error) => format!(
                    "Unable to create directory {} ({})",
                    path.to_string_lossy(),
                    error
                ),
                Error::DirectoryReadError(path, error) => format!(
                    "Unable to read directory {} ({})",
                    path.to_string_lossy(),
                    error
                ),
                Error::UncompiledFile(path) => format!(
                    "Could not find appropriate language for {}",
                    path.to_string_lossy()
                ),
                Error::RemoveObjectsDirectoryError(error) =>
                    format!("Failed to remove objects directory ({})", error),
                Error::RemoveTargetError(target, error) =>
                    format!("Failed to remove {} ({})", target, error),
                Error::RunCompilerError(compiler, error) =>
                    format!("Unable to run {} compiler ({})", compiler, error),
                Error::CompileError(file) =>
                    format!("Error while compiling {}", file.to_string_lossy()),
                Error::NoName => format!("No name specified in brewfile"),
                Error::RunLinkerError(error) => format!("Unable to run linker ({})", error),
                Error::LinkerError => format!("Error while linkning"),
                Error::RunBrewError(error) => format!("Unable to run brew ({})", error),
                Error::BrewError(path) => format!("Error while brewing {}", path.to_string_lossy()),
                Error::InstallTargetError(target, error) =>
                    format!("Error while installing {} ({})", target, error),
            }
        )
    }
}
