use crate::quote;
use crate::shader::error_handler::check_shader;
use crate::shader::transpiler::{transpile_shader, TranspiledData};
use crate::utils::colorized_text::Colorize;
use crate::utils::nested_console_logger::NestedConsoleLogger;
use gl::types::GLuint;
use std::ffi::CString;
use std::ptr;

pub struct Shader {
    pub(crate) id: GLuint,
    pub(crate) data: TranspiledData,
}

impl Shader {
    pub fn from_file(
        logger: &mut NestedConsoleLogger,
        file_name: &str,
        shader_type: GLuint,
    ) -> Shader {
        logger.open_scope("Compiling ".yellow() + quote!(file_name).magenta());
        let data = match transpile_shader(logger, file_name) {
            Ok(data) => data,
            Err(e) => {
                logger.close_scope("Compiling ".yellow() + "Failed".red());
                panic!("{}", "! Error ! ".red() + e.as_str().red());
            }
        };
        let id = unsafe { gl::CreateShader(shader_type) };

        let source = data.transpiled_source.bytes().collect::<Vec<u8>>();
        let c_source = CString::new(source).unwrap();

        unsafe {
            gl::ShaderSource(id, 1, &c_source.as_ptr(), ptr::null());
            gl::CompileShader(id);
        }

        match check_shader(logger, id, &data) {
            Ok(_) => {
                logger.close_scope("Compiling ".yellow() + "Successful".green());
            }
            Err(e) => {
                logger.close_scope("Compiling ".yellow() + "Failed".red());
                logger.panic(e.as_str().red());
                panic!("{}", "! Error ! ".red() + e.as_str().red());
            }
        }
        println!();

        Shader { id, data }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}
