use crate::utils::colorized_text::Colorize;
use crate::utils::html_logger::HTMLLogger;
use crate::{error, quote};
use regex::Regex;
use std::fs::File;
use std::io::Read;
use std::path::Path;

const FILE_PREFIX: &str = "res/shaders/";
const IGNORE_START: &str = r"\/\*\s*<ignore>\s*\*\/";
const IGNORE_END: &str = r"\/\*\s*<\/ignore>\s*\*\/";
const INCLUDE_PATTERN: &str = r####"#include\s+\"(.+)\""####;
const UNIFORM_PATTERN: &str = r"uniform\s+(.+)\s+(.+)\s*;";

/// A struct to contain the necessary information
/// through the shader transpilation process
/// Also used while formatting the error messages
#[derive(Debug)]
pub(crate) struct TranspiledData {
    pub(crate) transpiled_source: String,
    pub(crate) included_files: Vec<String>,
    pub(crate) line_to_source: Vec<(String, usize)>,
    pub(crate) uniforms: Vec<TranspiledUniform>,
}

#[derive(Clone, Debug)]
pub struct TranspiledUniform {
    pub(crate) name: String,
    pub(crate) ty: String,
}

/// Reads a file and returns its contents as a string
fn read_file(path: &str) -> Result<String, String> {
    let full_path = Path::new(FILE_PREFIX).join(path).canonicalize();
    match full_path {
        Ok(path) => {
            let mut file = File::open(path).unwrap();
            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();
            Ok(contents)
        }
        Err(e) => {
            error!("{:?}", e);
            Err(e.to_string())
        }
    }
}

/// Transpiles a shader file and returns the necessary information
pub fn transpile_shader(
    logger: &mut HTMLLogger,
    file_name: &str,
) -> Result<TranspiledData, String> {
    let mut data = TranspiledData {
        transpiled_source: String::new(),
        included_files: Vec::new(),
        line_to_source: Vec::new(),
        uniforms: Vec::new(),
    };

    logger.open_scope("Transpiling ".yellow());
    match handle_file(logger, file_name, &mut data) {
        Ok(_) => {
            if data.uniforms.len() > 0 {
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
    file_name: &str,
    data: &mut TranspiledData,
) -> Result<(), String> {
    logger.info("Including ".cyan() + quote!(file_name).magenta());

    // Check if the file has already been included
    if data.included_files.contains(&file_name.to_string()) {
        return Err(format!("File \"{}\" has already been included", file_name));
    }
    data.included_files.push(file_name.to_string());

    let file_contents = read_file(file_name)?;
    let mut ignore = false;

    // Turn the regex strings into regex objects that rust can use
    let ignore_start = Regex::new(IGNORE_START).unwrap();
    let ignore_end = Regex::new(IGNORE_END).unwrap();
    let include_pattern = Regex::new(INCLUDE_PATTERN).unwrap();
    let uniform_pattern = Regex::new(UNIFORM_PATTERN).unwrap();

    for (line_number, line) in file_contents.lines().enumerate() {
        // First things first, we add the line to the line_to_source map
        data.line_to_source
            .push((file_name.to_string(), line_number + 1));

        // The first thing we check is the ignore flag
        if ignore_start.is_match(line) {
            ignore = true;
        }

        if ignore_end.is_match(line) {
            ignore = false;
        }

        // Then we check for the imports
        let include_capture = include_pattern.captures(line);
        if let Some(include_capture) = include_capture {
            // If we are ignoring, straight up skip the line
            if ignore {
                continue;
            }

            let include_file = include_capture.get(1).unwrap().as_str();
            handle_file(logger, include_file, data)?;
            continue;
        }

        // If it isn't an import, we can add it to the transpiled source
        data.transpiled_source.push_str(line);
        data.transpiled_source.push('\n');

        // Then we check for the uniforms
        let uniform_capture = uniform_pattern.captures(line);
        if let Some(uniform_capture) = uniform_capture {
            let uniform_type = uniform_capture.get(1).unwrap().as_str();
            let uniform_name = uniform_capture.get(2).unwrap().as_str();
            data.uniforms.push(TranspiledUniform {
                name: uniform_name.to_string(),
                ty: uniform_type.to_string(),
            });
        }
    }

    Ok(())
}
