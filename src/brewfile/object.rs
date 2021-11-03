use crate::arguments::Options;

use super::{error::Error, Language};
use std::path::PathBuf;

pub struct Object {
    input_filename: PathBuf,
    output_filename: PathBuf,
    language: Language,
    install_target: PathBuf,
}

impl Object {
    pub fn new(name: String, language: Language, source: String, install_target: String) -> Self {
        Object {
            input_filename: PathBuf::from(source),
            output_filename: PathBuf::from(name),
            language,
            install_target: PathBuf::from(install_target),
        }
    }

    pub fn compile(&self, options: &Options) -> Result<(), Error> {
        match self
            .language
            .compile(&self.input_filename, &self.output_filename, options)
        {
            Ok(_) => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub fn install(&self, options: &Options) -> Result<(), Error> {
        let install_path = options.prefix().join(&self.install_target);

        if !options.quiet() {
            println!(
                "Installing {} to {} . . .",
                self.output_filename.to_string_lossy(),
                install_path.to_string_lossy(),
            );
        }

        match std::fs::copy(&self.output_filename, install_path) {
            Ok(_) => Ok(()),
            Err(error) => Err(Error::InstallTargetError(
                format!("{}", self.output_filename.to_string_lossy()),
                error,
            )),
        }
    }

    pub fn clean(&self) -> Result<(), Error> {
        if self.output_filename.exists() {
            match std::fs::remove_file(&self.output_filename) {
                Ok(_) => Ok(()),
                Err(error) => Err(Error::RemoveTargetError(
                    format!("{}", self.output_filename.to_string_lossy()),
                    error,
                )),
            }
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} <-- {} ({}) ({})",
            self.output_filename.to_string_lossy(),
            self.input_filename.to_string_lossy(),
            self.language,
            self.install_target.to_string_lossy(),
        )
    }
}
