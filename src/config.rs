pub const BREWFILE_NAME: &str = "./brewfile";

pub const DEFAULT_PREFIX: &str = "/los/";
pub const DEFAULT_SYSROOT: &str = "/";

pub const SOURCES_PATH: &str = "./src";
pub const OBJECTS_PATH: &str = "./obj";

pub const C_COMPILER: &str = "clang";
pub const C_COMPILER_FLAGS: [&str; 5] = ["--target=x86_64-los", "-Wall", "-g", "-c", "-I./include"];

pub const CPP_COMPILER: &str = "clang++";
pub const CPP_COMPILER_FLAGS: [&str; 5] = C_COMPILER_FLAGS;

pub const ASSEMBLER: &str = "nasm";
pub const ASSEMBLER_FLAGS: [&str; 5] = ["-f", "elf64", "-g", "-F", "dwarf"];

pub const LINKER: &str = "clang";
pub const LINKER_FLAGS: [&str; 1] = ["--target=x86_64-los"];

pub const ARCHIVER: &str = "ar";
pub const ARCHIVER_FLAGS: [&str; 1] = ["rcs"];
