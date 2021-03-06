use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ArgumentError {
    TwoCommands,
    TwoSysroots,
    TwoPrefixes,
    InvalidCommand(String),
    NoSysrootAfterOption,
    NoPrefixAfterOption,
}

pub enum Command {
    Build,
    Install,
    Clean,
}

pub struct Options {
    command: Command,
    verbose: bool,
    quiet: bool,
    sysroot: PathBuf,
    prefix: PathBuf,
}

pub fn parse_arguments(arguments: Vec<String>) -> Result<Options, ArgumentError> {
    let mut command = None;
    let mut verbose = false;
    let mut quiet = false;
    let mut sysroot = None;
    let mut prefix = None;

    let mut iter = arguments.iter();
    iter.next(); // Ignore first argument
    while let Some(argument) = iter.next() {
        match argument.as_str() {
            "-v" | "--verbose" => verbose = true,
            "-q" | "--quiet" => quiet = true,
            "--sysroot" => {
                let new_sysroot = match iter.next() {
                    Some(string) => string,
                    None => return Err(ArgumentError::NoSysrootAfterOption),
                };

                match sysroot {
                    Some(_) => return Err(ArgumentError::TwoSysroots),
                    None => sysroot = Some(new_sysroot),
                }
            }
            "--prefix" => {
                let new_prefix = match iter.next() {
                    Some(string) => string,
                    None => return Err(ArgumentError::NoPrefixAfterOption),
                };

                match prefix {
                    Some(_) => return Err(ArgumentError::TwoPrefixes),
                    None => prefix = Some(new_prefix),
                }
            }
            _ => {
                let new_command = Command::parse(argument)?;
                match command {
                    Some(_) => return Err(ArgumentError::TwoCommands),
                    None => command = Some(new_command),
                }
            }
        }
    }

    Ok(Options {
        command: match command {
            Some(command) => command,
            None => Command::Build,
        },
        verbose,
        quiet,
        sysroot: PathBuf::from(match sysroot {
            Some(sysroot) => sysroot,
            None => crate::config::DEFAULT_SYSROOT,
        }),
        prefix: PathBuf::from(match prefix {
            Some(prefix) => prefix,
            None => crate::config::DEFAULT_PREFIX,
        }),
    })
}

impl Command {
    pub fn parse(string: &str) -> Result<Self, ArgumentError> {
        match string {
            "clean" => Ok(Command::Clean),
            "build" => Ok(Command::Build),
            "install" => Ok(Command::Install),
            _ => Err(ArgumentError::InvalidCommand(string.to_owned())),
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Command::Build => "build",
                Command::Install => "install",
                Command::Clean => "clean",
            }
        )
    }
}

impl std::error::Error for ArgumentError {}

impl std::fmt::Display for ArgumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ArgumentError::TwoCommands => format!("Attempting to specify two commands"),
                ArgumentError::TwoSysroots => format!("Attempting to specify two system roots"),
                ArgumentError::TwoPrefixes => format!("Attempting to specify two prefixes"),
                ArgumentError::InvalidCommand(command) =>
                    format!("Unknown command \"{}\"", command),
                ArgumentError::NoSysrootAfterOption =>
                    format!("Nothing specified after \"--sysroot\""),
                ArgumentError::NoPrefixAfterOption =>
                    format!("Nothing specified after \"--prefix\""),
            }
        )
    }
}

impl Options {
    pub fn verbose(&self) -> bool {
        self.verbose
    }

    pub fn quiet(&self) -> bool {
        self.quiet
    }

    pub fn command(&self) -> &Command {
        &self.command
    }

    pub fn sysroot(&self) -> &Path {
        &self.sysroot
    }

    pub fn prefix(&self) -> &Path {
        &self.prefix
    }
}

impl std::fmt::Display for Options {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Command: {}", self.command)?;
        writeln!(f, "Verbose: {}", self.verbose)?;
        writeln!(f, "System Root: {}", self.sysroot.to_string_lossy())?;
        writeln!(f, "Prefix: {}", self.prefix.to_string_lossy())
    }
}
