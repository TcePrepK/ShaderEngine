use crate::quote;
use crate::utils::colorized_text::Colorize;
use crate::utils::html_logger::HTMLLogger;
use regex::{Captures, Regex};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub const SHADER_FILE_PREFIX: &str = "res/shaders/";
const IGNORE_START: &str = r"\/\*\s*<ignore>\s*\*\/";
const IGNORE_END: &str = r"\/\*\s*<\/ignore>\s*\*\/";
const INCLUDE_PATTERN: &str = r####"#include\s+\"(.+)\""####;
const UNIFORM_PATTERN: &str = r"uniform\s+(.+)\s+(.+)\s*;";

/// A struct to contain the necessary information
/// through the shader transpilation process
/// Also used while formatting the error messages
#[derive(Debug)]
pub(crate) struct ProcessData {
    pub(crate) processed_source: String,
    pub(crate) included_files: Vec<String>,
    pub(crate) line_to_source: Vec<(String, usize)>,
    pub(crate) uniforms: Vec<ProcessedUniform>,
}

#[derive(Clone, Debug)]
pub(crate) struct ProcessedUniform {
    pub(crate) name: String,
    pub(crate) ty: String,
}

/// Reads a file and returns its contents as a string
fn read_file(path: &str) -> Result<String, String> {
    let full_path = Path::new(SHADER_FILE_PREFIX)
        .join(path)
        .canonicalize()
        .unwrap();
    let mut file = File::open(full_path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    Ok(contents)
}

/// Processes a shader file and returns the necessary information
pub fn process_shader(logger: &mut HTMLLogger, file_name: &str) -> Result<ProcessData, String> {
    let mut data = ProcessData {
        processed_source: String::new(),
        included_files: Vec::new(),
        line_to_source: Vec::new(),
        uniforms: Vec::new(),
    };

    logger.open_scope("Processing ".yellow());
    match handle_file(logger, Path::new(file_name).to_path_buf(), &mut data) {
        Ok(_) => {
            if !data.uniforms.is_empty() {
                logger.open_scope("Uniforms".yellow());
                for uniform in data.uniforms.iter() {
                    logger.info(quote!(uniform.name).magenta() + ": ".cyan() + uniform.ty.green());
                }
                logger.close_scope();
            } else {
                logger.log("! ".cyan() + "No Uniforms".yellow() + " !".cyan())
            }
            logger.close_scope();
        }
        Err(e) => {
            return Err(e);
        }
    }

    Ok(data)
}

fn handle_file(
    logger: &mut HTMLLogger,
    file_path: PathBuf,
    data: &mut ProcessData,
) -> Result<(), String> {
    let file_name = file_path.to_str().unwrap();
    logger.info("Including ".cyan() + quote!(file_name).magenta());

    // Check if the file has already been included
    if data.included_files.contains(&file_name.to_string()) {
        return Err(format!("\"{}\" included multiple times", file_name));
    }
    data.included_files.push(file_name.to_string());

    let file_contents = read_file(file_name)?;
    let mut ignore = false;

    // Turn the regex strings into regex objects that rust can use
    let ignore_start = Regex::new(IGNORE_START).unwrap();
    let ignore_end = Regex::new(IGNORE_END).unwrap();
    let include_pattern = Regex::new(INCLUDE_PATTERN).unwrap();

    let file_base = file_path.parent().unwrap().to_path_buf();
    for (line_number, line) in file_contents.lines().enumerate() {
        // The first thing we check is the ignore flag
        if ignore_start.is_match(line) {
            ignore = true;
        }

        if ignore_end.is_match(line) {
            ignore = false;
        }

        // Then we check for the imports
        if let Some(capture) = include_pattern.captures(line) {
            if ignore {
                continue;
            }

            include_capture(logger, line, &capture, &file_base, data)?;
            continue;
        }

        // First things first, we add the line to the line_to_source map
        data.line_to_source
            .push((file_name.to_string(), line_number + 1));

        // If it isn't an import, we can add it to the transpiled source
        data.processed_source.push_str(line);
        data.processed_source.push('\n');

        // Then we check for the uniforms
        uniform_capture(line, data);
    }

    Ok(())
}

fn include_capture(
    logger: &mut HTMLLogger,
    line: &str,
    capture: &Captures,
    file_base: &Path,
    data: &mut ProcessData,
) -> Result<(), String> {
    if line.starts_with("//") {
        return Ok(());
    }

    let include_file = capture.get(1).unwrap().as_str();
    let new_file_path = file_base.join(include_file);
    handle_file(logger, new_file_path, data)
}

fn uniform_capture(line: &str, data: &mut ProcessData) {
    if line.starts_with("//") {
        return;
    }

    let uniform_pattern = Regex::new(UNIFORM_PATTERN).unwrap();
    let uniform_capture = uniform_pattern.captures(line);
    if let Some(uniform_capture) = uniform_capture {
        let uniform_type = uniform_capture.get(1).unwrap().as_str();
        let uniform_name = uniform_capture.get(2).unwrap().as_str();
        data.uniforms.push(ProcessedUniform {
            name: uniform_name.to_string(),
            ty: uniform_type.to_string(),
        });
    }
}
