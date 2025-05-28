mod error_handler;
mod shader;
mod transpiler;

use crate::quote;
use crate::shader::shader::Shader;
use crate::uniform::{Uniform, UniformVariable};
use crate::utils::colorized_text::Colorize;
use crate::utils::html_logger::HTMLLogger;
use gl::types::{GLchar, GLint, GLuint};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;

macro_rules! match_uniform_type {
    ($self:ident, $logger:ident, $uniform:ident, $($ty_str:literal => $ty:ty = $default:expr),* $(,)?) => {
        match $uniform.ty.as_str() {
            $(
                $ty_str => {
                    $self.add_uniform::<$ty>($logger, &$uniform.name, $default);
                },
            )*
            _ => panic!("Unknown uniform type: {}", $uniform.ty),
        }
    };
}

pub struct ShaderProgram {
    name: String,
    id: GLuint,
    using: bool,
    shaders: Vec<Shader>,
    uniforms: HashMap<String, Rc<RefCell<dyn Uniform>>>,
}

impl ShaderProgram {
    pub fn toggle_use(&mut self) {
        self.using = !self.using;
        if self.using {
            unsafe {
                gl::UseProgram(self.id);
            }

            self.handle_uniforms();
        } else {
            unsafe {
                gl::UseProgram(0);
            }
        }
    }

    pub fn handle_uniforms(&mut self) {
        for ref_uniform in self.uniforms.values_mut() {
            let mut uniform = ref_uniform.borrow_mut();
            if uniform.is_dirty() {
                uniform.clear_dirty();
                uniform.load_uniform();
            }
        }
    }

    #[allow(dead_code)]
    pub fn generate_graphics(
        name: &str,
        vertex_file: &str,
        fragment_file: &str,
    ) -> Result<ShaderProgram, String> {
        let mut logger = HTMLLogger::new(name);
        logger.open_scope("Creating ".yellow() + name.magenta());

        let vertex_shader = Shader::from_file(&mut logger, &vertex_file, gl::VERTEX_SHADER)?;
        let fragment_shader = Shader::from_file(&mut logger, &fragment_file, gl::FRAGMENT_SHADER)?;

        logger.open_scope("Program Linking ".yellow() + "Starting".green());
        logger.info("Attaching ".cyan() + quote!(vertex_file).magenta());
        logger.info("Attaching ".cyan() + quote!(fragment_file).magenta());

        let program = unsafe { gl::CreateProgram() };
        unsafe {
            gl::AttachShader(program, vertex_shader.id);
            gl::AttachShader(program, fragment_shader.id);
            gl::LinkProgram(program);
        }

        check_program(&mut logger, program);

        unsafe {
            gl::DetachShader(program, vertex_shader.id);
            gl::DetachShader(program, fragment_shader.id);
        }

        let mut shader_program = ShaderProgram {
            name: name.to_string(),
            id: program,
            using: false,
            shaders: Vec::from([vertex_shader, fragment_shader]),
            uniforms: HashMap::new(),
        };

        shader_program.link_all_uniforms(&mut logger);

        logger.to_html();
        Ok(shader_program)
    }

    #[allow(dead_code)]
    pub fn generate_compute(name: &str, compute_file: &str) -> Result<ShaderProgram, String> {
        let mut logger = HTMLLogger::new(name);
        logger.open_scope("Creating ".yellow() + name.magenta());

        let compute_shader = Shader::from_file(&mut logger, &compute_file, gl::COMPUTE_SHADER)?;

        logger.open_scope("Program Linking ".yellow() + "Starting".green());
        logger.info("Attaching ".cyan() + quote!(compute_file).magenta());

        let program = unsafe { gl::CreateProgram() };
        unsafe {
            gl::AttachShader(program, compute_shader.id);
            gl::LinkProgram(program);
        }

        check_program(&mut logger, program);

        unsafe {
            gl::DetachShader(program, compute_shader.id);
        }

        let mut shader_program = ShaderProgram {
            name: name.to_string(),
            id: program,
            using: false,
            shaders: Vec::from([compute_shader]),
            uniforms: HashMap::new(),
        };

        shader_program.link_all_uniforms(&mut logger);

        logger.to_html();
        Ok(shader_program)
    }

    pub fn get_uniform<T: 'static>(&mut self, name: &str) -> Option<Rc<RefCell<UniformVariable<T>>>>
    where
        UniformVariable<T>: Uniform,
    {
        let uniform = self.uniforms.get(name)?.clone();

        // Try to borrow and check if the inner type is UniformVariable<T>
        if uniform.borrow().as_any().is::<UniformVariable<T>>() {
            // SAFETY: We confirmed the inner type, now cast the Rc to the correct type
            // But we cannot cast Rc<RefCell<dyn Uniform>> to Rc<RefCell<UniformVariable<T>>> directly
            // So, here's the workaround:

            let raw = Rc::into_raw(uniform.clone()) as *const RefCell<UniformVariable<T>>;
            let converted = unsafe { Rc::from_raw(raw) };

            Some(converted)
        } else {
            None
        }
    }
}

impl ShaderProgram {
    fn link_all_uniforms(&mut self, logger: &mut HTMLLogger) {
        logger.open_scope("Uniform Linking ".yellow() + "Starting".green());

        let all_uniforms = self
            .shaders
            .iter()
            .flat_map(|shader| shader.data.uniforms.iter())
            .cloned()
            .collect::<Vec<_>>();

        for uniform in all_uniforms {
            match_uniform_type!(self, logger, uniform,
                "bool" => bool = false,
                "int" => i32 = 0,
                "uint" => u32 = 0,
                "float" => f32 = 0.0,
                "double" => f64 = 0.0,

                "bvec2" => [bool; 2] = [false; 2],
                "bvec3" => [bool; 3] = [false; 3],
                "bvec4" => [bool; 4] = [false; 4],
                "ivec2" => [i32; 2] = [0; 2],
                "ivec3" => [i32; 3] = [0; 3],
                "ivec4" => [i32; 4] = [0; 4],
                "uvec2" => [u32; 2] = [0; 2],
                "uvec3" => [u32; 3] = [0; 3],
                "uvec4" => [u32; 4] = [0; 4],
                "vec2" => [f32; 2] = [0.0; 2],
                "vec3" => [f32; 3] = [0.0; 3],
                "vec4" => [f32; 4] = [0.0; 4],
                "dvec2" => [f64; 2] = [0.0; 2],
                "dvec3" => [f64; 3] = [0.0; 3],
                "dvec4" => [f64; 4] = [0.0; 4],
            );
        }

        logger.close_scope();
    }

    fn add_uniform<T: 'static>(&mut self, logger: &mut HTMLLogger, name: &str, initial: T)
    where
        UniformVariable<T>: Uniform,
    {
        let uniform = UniformVariable::new(name, initial);
        let ref_uniform = Rc::new(RefCell::new(uniform));
        let successful = ref_uniform.borrow_mut().bind_program(self.id);
        self.uniforms.insert(name.to_string(), ref_uniform.clone());
        match successful {
            Ok(_) => {
                logger.info("Uniform ".cyan() + quote!(name).magenta() + " found".green());
            }
            Err(_) => {
                logger.info("Uniform ".cyan() + quote!(name).magenta() + " not found".red());
            }
        }
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

/// Checks if a program has compiled successfully or not
fn check_program(logger: &mut HTMLLogger, program: GLuint) {
    // Success flag determines if the program compiled successfully
    let mut success: GLint = 0;
    unsafe {
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    }

    // If the shader failed to compile, success will be 0
    if success == 0 {
        // Get the length of the error message using `gl::GetProgramiv`
        let mut len: GLint = 0;
        unsafe {
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        }

        // Create a buffer out of the length of the error message, gl functions require null terminated strings
        // This is why we create a CString from the buffer
        let mut buffer: Vec<u8> = Vec::with_capacity(len as usize + 1);
        buffer.extend([b' '].iter().cycle().take(len as usize));
        let error_message = unsafe { CString::from_vec_unchecked(buffer) };

        unsafe {
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                error_message.as_ptr() as *mut GLchar,
            );
        }

        logger.close_scope();

        let error_message = error_message.to_string_lossy().into_owned();
        panic!("{}", "! Error ! ".red() + error_message.as_str().red());
    }

    logger.close_scope();
}
