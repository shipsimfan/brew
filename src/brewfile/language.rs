use std::{
    path::Path,
    process::{Command, Stdio},
};

use crate::arguments::Options;

use super::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Assembly,
    C,
    CPlusPlus,
}

pub enum CompileStatus {
    None,
    Ignore,
    Complete,
}

fn get_extension(path: &Path) -> String {
    format!(
        "{}",
        match path.extension() {
            Some(str) => str,
            None => return String::new(),
        }
        .to_string_lossy()
    )
}

// Returns true if the file should be compiled
fn check_date(source: &Path, object: &Path) -> bool {
    if !object.exists() {
        return true;
    }

    let source_time = match source.metadata() {
        Ok(meta) => match meta.modified() {
            Ok(time) => time,
            Err(_) => return true,
        },
        Err(_) => return true,
    };

    let object_time = match object.metadata() {
        Ok(meta) => match meta.modified() {
            Ok(time) => time,
            Err(_) => return true,
        },
        Err(_) => return true,
    };

    return source_time > object_time;
}

impl Language {
    pub fn parse(name: &str) -> Result<Self, Error> {
        Ok(match name {
            "assembly" | "asm" => Language::Assembly,
            "c" => Language::C,
            "cpp" | "c++" | "cplusplus" => Language::CPlusPlus,
            _ => return Err(Error::UnknownLanguage(name.to_owned())),
        })
    }

    pub fn compile(
        &self,
        source_path: &Path,
        destination_path: &Path,
        options: &Options,
    ) -> Result<CompileStatus, Error> {
        match self {
            Language::Assembly => Self::compile_assembly(source_path, destination_path, options),
            Language::C => Self::compile_c(source_path, destination_path, options),
            Language::CPlusPlus => Self::compile_cpp(source_path, destination_path, options),
        }
    }

    fn compile_c(
        source_path: &Path,
        destination_path: &Path,
        options: &Options,
    ) -> Result<CompileStatus, Error> {
        // Verify extension
        match get_extension(source_path).as_str() {
            "c" => {}
            "h" => return Ok(CompileStatus::Ignore),
            _ => return Ok(CompileStatus::None),
        }

        // Verify time
        if !check_date(source_path, destination_path) {
            return Ok(CompileStatus::Complete);
        }

        // Compile file
        if !options.quiet() {
            println!(
                "Compiling {} to {} . . .",
                source_path.to_string_lossy(),
                destination_path.to_string_lossy()
            );
        }

        let mut command = Command::new(crate::config::C_COMPILER);
        command.args(crate::config::C_COMPILER_FLAGS);
        command.arg("-o");
        command.arg(destination_path);
        command.arg(source_path);
        command.arg(format!("--sysroot={}", options.sysroot().to_string_lossy()));
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command.stdin(Stdio::inherit());

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(CompileStatus::Complete)
                } else {
                    Err(Error::CompileError(source_path.to_owned()))
                }
            }
            Err(error) => Err(Error::RunCompilerError("c", error)),
        }
    }

    fn compile_cpp(
        source_path: &Path,
        destination_path: &Path,
        options: &Options,
    ) -> Result<CompileStatus, Error> {
        // Verify extension
        match get_extension(source_path).as_str() {
            "cpp" => {}
            "h" | "hpp" => return Ok(CompileStatus::Ignore),
            _ => return Ok(CompileStatus::None),
        }

        // Verify time
        if !check_date(source_path, destination_path) {
            return Ok(CompileStatus::Complete);
        }

        // Compile file
        if !options.quiet() {
            println!(
                "Compiling {} to {} . . .",
                source_path.to_string_lossy(),
                destination_path.to_string_lossy()
            );
        }

        let mut command = Command::new(crate::config::CPP_COMPILER);
        command.args(crate::config::CPP_COMPILER_FLAGS);
        command.arg("-o");
        command.arg(destination_path);
        command.arg(source_path);
        command.arg(format!("--sysroot={}", options.sysroot().to_string_lossy()));
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command.stdin(Stdio::inherit());

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(CompileStatus::Complete)
                } else {
                    Err(Error::CompileError(source_path.to_owned()))
                }
            }
            Err(error) => Err(Error::RunCompilerError("c++", error)),
        }
    }

    fn compile_assembly(
        source_path: &Path,
        destination_path: &Path,
        options: &Options,
    ) -> Result<CompileStatus, Error> {
        // Verify extension
        match get_extension(source_path).as_str() {
            "asm" | "s" => {}
            _ => return Ok(CompileStatus::None),
        }

        // Verify time
        if !check_date(source_path, destination_path) {
            return Ok(CompileStatus::Complete);
        }

        // Assemble file
        if !options.quiet() {
            println!(
                "Assembling {} to {} . . .",
                source_path.to_string_lossy(),
                destination_path.to_string_lossy()
            );
        }

        let mut command = Command::new(crate::config::ASSEMBLER);
        command.args(crate::config::ASSEMBLER_FLAGS);
        command.arg("-o");
        command.arg(destination_path);
        command.arg(source_path);
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command.stdin(Stdio::inherit());

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(CompileStatus::Complete)
                } else {
                    Err(Error::CompileError(source_path.to_owned()))
                }
            }
            Err(error) => Err(Error::RunCompilerError("assembly", error)),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Language::Assembly => "Assembly",
                Language::C => "C",
                Language::CPlusPlus => "C++",
            }
        )
    }
}
