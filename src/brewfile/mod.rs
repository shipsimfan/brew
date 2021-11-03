use std::{
    collections::HashSet,
    path::PathBuf,
    process::{Command, Stdio},
};

mod error;
mod language;
mod object;

pub use language::Language;
pub use object::Object;

use crate::arguments::Options;

pub enum BrewType {
    Executable,
    Library,
    Group,
    None,
}

pub struct Brewfile {
    name: Option<String>,
    brew_type: BrewType,
    languages: HashSet<Language>,
    dependencies: HashSet<String>,
    objects: Vec<Object>,
    priority: Vec<PathBuf>,
}

impl Brewfile {
    pub fn new() -> Self {
        Brewfile {
            name: None,
            brew_type: BrewType::None,
            dependencies: HashSet::new(),
            languages: HashSet::new(),
            objects: Vec::new(),
            priority: Vec::new(),
        }
    }

    pub fn set_name(&mut self, name: String) -> Result<(), Box<dyn std::error::Error>> {
        match self.name {
            Some(_) => Err(Box::new(error::Error::NameDefinedTwice)),
            None => Ok(self.name = Some(name)),
        }
    }

    pub fn set_brew_type(&mut self, brew_type: BrewType) -> Result<(), Box<dyn std::error::Error>> {
        match self.brew_type {
            BrewType::None => Ok(self.brew_type = brew_type),
            _ => Err(Box::new(error::Error::BrewTypeDefinedTwice)),
        }
    }

    pub fn add_language(&mut self, language: Language) -> Result<(), Box<dyn std::error::Error>> {
        if !self.languages.insert(language) {
            Err(Box::new(error::Error::LanguageDefinedTwice(language)))
        } else {
            Ok(())
        }
    }

    pub fn add_dependency(&mut self, dependency: String) -> Result<(), Box<dyn std::error::Error>> {
        if !self.dependencies.insert(dependency.clone()) {
            Err(Box::new(error::Error::DependencyDefinedTwice(dependency)))
        } else {
            Ok(())
        }
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn add_priority(&mut self, priority: PathBuf) {
        self.priority.push(priority);
    }

    fn install_include_directory(
        source_path: PathBuf,
        destination_path: PathBuf,
        options: &Options,
    ) -> Result<(), error::Error> {
        let directory = match std::fs::read_dir(&source_path) {
            Ok(directory) => directory,
            Err(error) => return Err(error::Error::DirectoryReadError(source_path, error)),
        };

        for entry in directory {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => return Err(error::Error::DirectoryReadError(source_path, error)),
            };

            let path = entry.path();
            let new_dest_path = destination_path.join(path.file_name().unwrap());
            if path.is_dir() {
                match std::fs::create_dir_all(&new_dest_path) {
                    Ok(()) => {}
                    Err(error) => {
                        return Err(error::Error::DirectoryCreationError(new_dest_path, error))
                    }
                }
                Self::install_include_directory(path, new_dest_path, options)?
            } else {
                if !options.quiet() {
                    println!("Installing {} . . .", path.to_string_lossy());
                }
                match std::fs::copy(&path, new_dest_path) {
                    Ok(_) => {}
                    Err(error) => {
                        return Err(error::Error::InstallTargetError(
                            format!("{}", path.to_string_lossy()),
                            error,
                        ))
                    }
                }
            }
        }

        Ok(())
    }

    fn compile_directory(
        &self,
        source_path: PathBuf,
        destination_path: PathBuf,
        options: &Options,
    ) -> Result<Vec<PathBuf>, error::Error> {
        // Create output directory
        match std::fs::create_dir_all(&destination_path) {
            Ok(()) => {}
            Err(error) => {
                return Err(error::Error::DirectoryCreationError(
                    destination_path,
                    error,
                ))
            }
        }

        // Open input directory
        let source_directory = match std::fs::read_dir(&source_path) {
            Ok(directory) => directory,
            Err(error) => return Err(error::Error::DirectoryReadError(source_path, error)),
        };

        // Compile sub-directories and files
        let mut objects = Vec::new();
        for entry in source_directory {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => return Err(error::Error::DirectoryReadError(source_path, error)),
            };

            let path = entry.path();
            if path.file_name().unwrap().to_string_lossy().starts_with('.') {
                // Ignore files beginning with '.'
                continue;
            }

            let mut object_path = destination_path.join(path.file_name().unwrap());
            if path.is_dir() {
                let mut sub_objects = self.compile_directory(path, object_path, options)?;
                objects.append(&mut sub_objects);
            } else {
                object_path.set_extension("o");
                let mut compiled = false;
                'language_loop: for language in &self.languages {
                    match language.compile(&path, &object_path, options)? {
                        language::CompileStatus::Complete => {
                            compiled = true;
                            objects.push(object_path);
                            break 'language_loop;
                        }
                        language::CompileStatus::Ignore => {
                            compiled = true;
                            break 'language_loop;
                        }
                        _ => {}
                    }
                }

                if !compiled {
                    return Err(error::Error::UncompiledFile(path));
                }
            }
        }

        Ok(objects)
    }

    // Compile all objects in sources folder and return a list of those objects
    fn compile_source_directory(&self, options: &Options) -> Result<Vec<PathBuf>, error::Error> {
        if options.verbose() {
            println!()
        }

        self.compile_directory(
            PathBuf::from(crate::config::SOURCES_PATH),
            PathBuf::from(crate::config::OBJECTS_PATH),
            options,
        )
    }

    fn link_executable(
        &self,
        objects: Vec<PathBuf>,
        options: &Options,
    ) -> Result<(), error::Error> {
        let output = match &self.name {
            Some(name) => format!("{}.app", name),
            None => return Err(error::Error::NoName),
        };

        let mut command = Command::new(crate::config::LINKER);
        command.args(crate::config::LINKER_FLAGS);
        command.arg("-o");
        command.arg(&output);
        command.args(objects);
        command.arg(format!("--sysroot={}", options.sysroot().to_string_lossy()));
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command.stdin(Stdio::inherit());

        if !options.quiet() {
            println!("Linking {} . . .", output);
        }

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(error::Error::LinkerError)
                }
            }
            Err(error) => Err(error::Error::RunLinkerError(error)),
        }
    }

    fn link_library(&self, objects: Vec<PathBuf>, options: &Options) -> Result<(), error::Error> {
        let output = match &self.name {
            Some(name) => format!("lib{}.a", name),
            None => return Err(error::Error::NoName),
        };

        let mut command = Command::new(crate::config::ARCHIVER);
        command.args(crate::config::ARCHIVER_FLAGS);
        command.arg(&output);
        command.args(objects);
        command.arg(format!("--sysroot={}", options.sysroot().to_string_lossy()));
        command.stdout(Stdio::inherit());
        command.stderr(Stdio::inherit());
        command.stdin(Stdio::inherit());

        if !options.quiet() {
            println!("Linking {} . . .", output);
        }

        match command.output() {
            Ok(output) => {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(error::Error::LinkerError)
                }
            }
            Err(error) => Err(error::Error::RunLinkerError(error)),
        }
    }

    fn brew_sub_directory(&self, path: PathBuf, options: &Options) -> Result<(), error::Error> {
        // Brew sub directory
        let sysroot = if options.sysroot().has_root() {
            options.sysroot().to_owned()
        } else {
            PathBuf::from("..").join(options.sysroot())
        };

        let prefix = if options.prefix().has_root() {
            options.prefix().to_owned()
        } else {
            PathBuf::from("..").join(options.prefix())
        };

        let mut command = Command::new("brew");
        command.arg(format!("{}", options.command()));
        command.arg("--sysroot");
        command.arg(sysroot);
        command.arg("--prefix");
        command.arg(prefix);
        if options.verbose() {
            command.arg("-v");
        }
        if options.quiet() {
            command.arg("-q");
        }

        command.current_dir(&path);

        command.stderr(Stdio::inherit());
        command.stdin(Stdio::inherit());
        command.stdout(Stdio::inherit());

        if !options.quiet() {
            println!("Brewing {} . . .", path.to_string_lossy());
        }

        match command.output() {
            Ok(output) => {
                if !output.status.success() {
                    return Err(error::Error::BrewError(path));
                }
            }
            Err(error) => return Err(error::Error::RunBrewError(error)),
        }

        if !options.quiet() {
            println!();
        }

        Ok(())
    }

    fn brew_sub_folders(&self, options: Options) -> Result<(), error::Error> {
        // Brew priority first
        for priority in &self.priority {
            self.brew_sub_directory(priority.to_owned(), &options)?;
        }

        let directory = match std::fs::read_dir(".") {
            Ok(dir) => dir,
            Err(error) => return Err(error::Error::DirectoryReadError(PathBuf::from("."), error)),
        };

        'entry_loop: for entry in directory {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    return Err(error::Error::DirectoryReadError(PathBuf::from("."), error))
                }
            };

            let path = entry.path();

            if path.file_name().unwrap().to_string_lossy().starts_with('.') {
                continue;
            }

            for priority in &self.priority {
                if path.file_name().unwrap() == priority.as_os_str() {
                    continue 'entry_loop;
                }
            }

            if path.is_dir() && !path.file_name().unwrap().to_string_lossy().starts_with(".") {
                self.brew_sub_directory(path, &options)?;
            }
        }

        Ok(())
    }

    fn clean(&self) -> Result<(), error::Error> {
        // Remove object directory
        let object_directory = PathBuf::from(crate::config::OBJECTS_PATH);
        if object_directory.exists() {
            match std::fs::remove_dir_all(&object_directory) {
                Ok(()) => {}
                Err(error) => return Err(error::Error::RemoveObjectsDirectoryError(error)),
            }
        }

        // Remove objects
        for object in &self.objects {
            object.clean()?;
        }

        // Remove target
        let name = match &self.name {
            Some(name) => name,
            None => return Err(error::Error::NoName),
        };

        let target = PathBuf::from(match self.brew_type {
            BrewType::Executable => format!("{}.app", name),
            _ => format!("lib{}.a", name),
        });

        if target.exists() {
            match std::fs::remove_file(&target) {
                Ok(()) => Ok(()),
                Err(error) => Err(error::Error::RemoveTargetError(
                    format!("{}", target.to_string_lossy()),
                    error,
                )),
            }
        } else {
            Ok(())
        }
    }

    pub fn execute(self, options: Options) -> Result<(), Box<dyn std::error::Error>> {
        match self.brew_type {
            BrewType::Group => return Ok(self.brew_sub_folders(options)?),
            BrewType::None => return Err(Box::new(error::Error::NoBrewType)),
            _ => {}
        }

        match options.command() {
            crate::arguments::Command::Build | crate::arguments::Command::Install => {
                let objects = self.compile_source_directory(&options)?;

                if options.verbose() {
                    println!("Objects to link:");
                    for object in &objects {
                        println!(" - {}", object.to_string_lossy());
                    }
                }

                match self.brew_type {
                    BrewType::Executable => self.link_executable(objects, &options)?,
                    _ => self.link_library(objects, &options)?,
                }

                // Compile objects
                for object in &self.objects {
                    object.compile(&options)?;
                }

                match options.command() {
                    crate::arguments::Command::Install => {
                        // Install objects
                        for object in &self.objects {
                            object.install(&options)?;
                        }

                        // Install target
                        let mut target_path = options.prefix().to_owned();
                        let source = match self.brew_type {
                            BrewType::Executable => {
                                target_path.push("bin");
                                format!(
                                    "{}.app",
                                    match self.name {
                                        Some(name) => name,
                                        None => return Err(Box::new(error::Error::NoName)),
                                    }
                                )
                            }
                            _ => {
                                target_path.push("lib");
                                format!(
                                    "lib{}.a",
                                    match self.name {
                                        Some(name) => name,
                                        None => return Err(Box::new(error::Error::NoName)),
                                    }
                                )
                            }
                        };

                        target_path.push(&source);

                        if !options.quiet() {
                            println!("Installing {} . . .", &source);
                        }

                        match std::fs::copy(&source, target_path) {
                            Ok(_) => {}
                            Err(error) => {
                                return Err(Box::new(error::Error::InstallTargetError(
                                    source, error,
                                )))
                            }
                        }

                        // Install headers
                        match self.brew_type {
                            BrewType::Library => {
                                let include_path = PathBuf::from("./include");
                                if include_path.exists() {
                                    Self::install_include_directory(
                                        include_path,
                                        options.prefix().join("include"),
                                        &options,
                                    )?;
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }

                Ok(())
            }
            crate::arguments::Command::Clean => Ok(self.clean()?),
        }
    }
}

impl std::fmt::Display for Brewfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.name {
            None => {}
            Some(name) => writeln!(f, "Name: {}", name)?,
        }

        writeln!(f, "Brew Type: {}", self.brew_type)?;

        if self.languages.len() > 0 {
            writeln!(f, "Languages:")?;
            for language in &self.languages {
                writeln!(f, " - {}", language)?;
            }
        }

        if self.dependencies.len() > 0 {
            writeln!(f, "Dependencies:")?;
            for dependency in &self.dependencies {
                writeln!(f, " - {}", dependency)?;
            }
        }

        if self.objects.len() > 0 {
            writeln!(f, "Objects:")?;
            for object in &self.objects {
                writeln!(f, " - {}", object)?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for BrewType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                BrewType::None => "None",
                BrewType::Executable => "Executable",
                BrewType::Library => "Library",
                BrewType::Group => "Group",
            }
        )
    }
}
