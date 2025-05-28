use crate::quote;
use crate::shader::error_handler::check_shader;
use crate::shader::transpiler::{transpile_shader, TranspiledData};
use crate::utils::colorized_text::Colorize;
use crate::utils::html_logger::HTMLLogger;
use gl::types::GLuint;
use std::ffi::CString;
use std::ptr;

pub struct Shader {
    pub(crate) id: GLuint,
    pub(crate) data: TranspiledData,
}

impl Shader {
    pub fn from_file(
        logger: &mut HTMLLogger,
        file_name: &str,
        shader_type: GLuint,
    ) -> Result<Shader, String> {
        logger.open_scope("Compiling ".yellow() + quote!(file_name).magenta());

        let data = match transpile_shader(logger, file_name) {
            Ok(data) => data,
            Err(e) => {
                logger.error("! Error ! ".red() + e.as_str().red());
                return Err(e);
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
                logger.close_scope();
            }
            Err(e) => {
                logger.panic();
                return Err(e);
            }
        }

        Ok(Shader { id, data })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}
