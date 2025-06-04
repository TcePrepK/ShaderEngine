use crate::shader::transpiler::TranspiledData;
use crate::utils::colorized_text::Colorize;
use crate::utils::html_logger::HTMLLogger;
use gl::types::{GLchar, GLint, GLuint};
use regex::Regex;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;

const SHADER_LINE_FORMAT: &str = r"0\((\d+)\) : error .{5}: (.+)";

/// Checks if a shader has compiled successfully or not
/// Depending on if it hasn't compiled successfully,
/// it will print out a well-formatted error message,
/// which includes the line and file the error occurred on (required for #include)
pub fn check_shader(
    logger: &mut HTMLLogger,
    shader: GLuint,
    data: &TranspiledData,
) -> Result<(), String> {
    // Success flag determines if the shader compiled successfully
    let mut success: GLint = 0;
    unsafe {
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    }

    // If the shader failed to compile, success will be 0
    if success == 0 {
        // Get the length of the error message
        let mut len: GLint = 0;
        unsafe {
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
        }

        // Create a buffer out of the length of the error message.
        // gl functions require null terminated strings, so we create a CString from the buffer
        let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
        buffer.extend([b' '].iter().cycle().take(len as usize));
        let error_message = unsafe { CString::from_vec_unchecked(buffer) };

        unsafe {
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                error_message.as_ptr() as *mut GLchar,
            );
        }

        // Format the error message and print it
        let readable_error = error_message.to_string_lossy().into_owned();
        shader_error_handler(logger, readable_error.as_str(), data);

        return Err("Error compiling shader".to_string());
    }

    Ok(())
}

/// Handles the error message from a shader
/// and formats it into an easy-to-understand, readable message
fn shader_error_handler(logger: &mut HTMLLogger, error_message: &str, data: &TranspiledData) {
    let line_number_regex = Regex::new(SHADER_LINE_FORMAT).unwrap();

    logger.open_scope("Compilation Errors".red());

    // Generate a map for each source file and its errors
    let mut errors_by_source: HashMap<String, Vec<(usize, String)>> = HashMap::new();
    for source in data.included_files.iter() {
        errors_by_source.insert(source.clone(), Vec::new());
    }

    // Handle each line of the error
    for line in error_message.lines() {
        // Skip the null terminator
        if line == "\0" {
            continue;
        }

        let capture = line_number_regex.captures(line).unwrap();
        let line_number = capture.get(1).unwrap().as_str().parse::<usize>().unwrap();
        let error = capture.get(2).unwrap().as_str().to_string();

        let (source, _) = data.line_to_source[line_number - 1].clone();
        let errors = errors_by_source.get_mut(&source).unwrap();
        errors.push((line_number, error));
    }

    // Log the errors
    for source in data.included_files.iter() {
        let errors = errors_by_source.get(source).unwrap();
        if errors.is_empty() {
            continue;
        }

        logger.open_scope(source.yellow());
        for (line_number, error) in errors.iter() {
            let actual_line_number = data.line_to_source[*line_number - 1].1;
            logger.log(format!("{}", actual_line_number).red() + ": ".cyan() + error.white());
        }
        logger.close_scope();
    }

    logger.close_scope();
}
