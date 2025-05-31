use crate::quote;
use crate::shader::error_handler::check_shader;
use crate::shader::transpiler::{transpile_shader, TranspiledData, SHADER_FILE_PREFIX};
use crate::utils::colorized_text::Colorize;
use crate::utils::file_watcher::FileWatcher;
use crate::utils::html_logger::HTMLLogger;
use gl::types::GLuint;
use std::ffi::CString;
use std::ptr;

pub struct Shader {
    pub(crate) id: GLuint,
    pub(crate) data: TranspiledData,
    pub(crate) watchers: Vec<FileWatcher>,
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
                logger.close_scope();
                logger.error(e.as_str().red());
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

        logger.close_scope();
        check_shader(logger, id, &data)?;

        let watchers = get_file_watchers(&data.included_files);

        Ok(Shader { id, data, watchers })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.id);
        }
    }
}

fn get_file_watchers(files: &Vec<String>) -> Vec<FileWatcher> {
    let mut watchers = Vec::new();
    for file in files.iter() {
        watchers.push(FileWatcher::new(SHADER_FILE_PREFIX.to_owned() + file));
    }
    watchers
}
