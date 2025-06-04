mod bindable;
mod error_handler;
mod shader_gen;
mod transpiler;
mod uniform;

use crate::image_buffer::{Image1D, Image2D, Image3D};
use crate::quote;
use crate::shader::shader_gen::Shader;
use crate::shader::uniform::{Uniform, UniformVariable};
use crate::utils::colorized_text::Colorize;
use crate::utils::html_logger::{Details, HTMLLogger, Summary};
use gl::types::{GLchar, GLint, GLuint};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::ptr;
use std::rc::Rc;

pub trait ShaderType {}

pub struct ComputeShader {
    compute_file: String,
}

pub struct GraphicsShader {
    vertex_file: String,
    fragment_file: String,
}

impl ShaderType for ComputeShader {}
impl ShaderType for GraphicsShader {}

pub struct ShaderProgram<T> {
    name: String,
    type_data: T,
    pub(crate) id: GLuint,
    using: bool,
    shaders: Vec<Shader>,
    uniforms: HashMap<String, Rc<RefCell<dyn Uniform>>>,
    images: HashMap<String, GLint>,
}

impl ShaderProgram<GraphicsShader> {
    pub fn new(
        logger: &mut HTMLLogger,
        name: &str,
        vertex_file: &str,
        fragment_file: &str,
    ) -> Result<ShaderProgram<GraphicsShader>, String> {
        let main_scope = logger.open_scope("Creating ".yellow() + name.magenta());

        let (vertex_shader, fragment_shader) =
            match ShaderProgram::<GraphicsShader>::generate_shaders(logger, vec![
                (vertex_file, gl::VERTEX_SHADER),
                (fragment_file, gl::FRAGMENT_SHADER),
            ]) {
                Ok(shaders) => {
                    let mut reversed = shaders.into_iter().rev().collect::<Vec<_>>();
                    (reversed.pop().unwrap(), reversed.pop().unwrap())
                }
                Err(e) => {
                    main_scope
                        .borrow_mut()
                        .summary
                        .text
                        .push_str(" Failed".red().as_str());
                    logger.panic();
                    return Err(e);
                }
            };

        logger.open_scope("Program Linking ".yellow());
        logger.info("Attaching ".cyan() + quote!(vertex_file).magenta());
        logger.info("Attaching ".cyan() + quote!(fragment_file).magenta());

        let program = unsafe { gl::CreateProgram() };
        unsafe {
            gl::AttachShader(program, vertex_shader.id);
            gl::AttachShader(program, fragment_shader.id);
            gl::LinkProgram(program);
        }

        {
            let summary = &mut main_scope.borrow_mut().summary;
            check_program(logger, summary, program)?;
        }

        unsafe {
            gl::DetachShader(program, vertex_shader.id);
            gl::DetachShader(program, fragment_shader.id);
        }

        let mut shader_program = ShaderProgram {
            name: name.to_string(),
            type_data: GraphicsShader {
                vertex_file: vertex_file.to_owned(),
                fragment_file: fragment_file.to_owned(),
            },
            id: program,
            using: false,
            shaders: Vec::from([vertex_shader, fragment_shader]),
            uniforms: HashMap::new(),
            images: HashMap::new(),
        };

        shader_program.link_all_uniforms(logger);

        logger.close_scope();
        logger.to_html();
        Ok(shader_program)
    }
}

impl ShaderProgram<ComputeShader> {
    pub fn new(
        logger: &mut HTMLLogger,
        name: &str,
        compute_file: &str,
    ) -> Result<ShaderProgram<ComputeShader>, String> {
        let main_scope = logger.open_scope("Creating ".yellow() + name.magenta());

        let compute_shader = match ShaderProgram::<ComputeShader>::generate_shaders(logger, vec![(
            compute_file,
            gl::COMPUTE_SHADER,
        )]) {
            Ok(mut shaders) => shaders.pop().unwrap(),
            Err(e) => {
                main_scope
                    .borrow_mut()
                    .summary
                    .text
                    .push_str(" Failed".red().as_str());
                logger.panic();
                return Err(e);
            }
        };

        logger.open_scope("Program Linking ".yellow() + "Starting".green());
        logger.info("Attaching ".cyan() + quote!(compute_file).magenta());

        let program = unsafe { gl::CreateProgram() };
        unsafe {
            gl::AttachShader(program, compute_shader.id);
            gl::LinkProgram(program);
        }

        {
            let summary = &mut main_scope.borrow_mut().summary;
            check_program(logger, summary, program)?;
        }

        unsafe {
            gl::DetachShader(program, compute_shader.id);
        }

        let mut shader_program = ShaderProgram {
            name: name.to_string(),
            type_data: ComputeShader {
                compute_file: compute_file.to_owned(),
            },
            id: program,
            using: false,
            shaders: Vec::from([compute_shader]),
            uniforms: HashMap::new(),
            images: HashMap::new(),
        };

        shader_program.link_all_uniforms(logger);

        logger.close_scope();
        logger.to_html();
        Ok(shader_program)
    }
}

// Program related functions
impl<ST: ShaderType> ShaderProgram<ST> {
    fn generate_shaders(
        logger: &mut HTMLLogger,
        files_and_type: Vec<(&str, GLuint)>,
    ) -> Result<Vec<Shader>, String> {
        files_and_type
            .iter()
            .map(|(file, ty)| Shader::from_file(logger, file, *ty))
            .collect()
    }

    /// Toggles between on and off and when it's on, it also handles/loads the uniforms
    pub fn toggle_use(&mut self) {
        self.using = !self.using;
        if self.using {
            unsafe {
                gl::UseProgram(self.id);
            }

            self.handle_uniforms(false);
        } else {
            unsafe {
                gl::UseProgram(0);
            }
        }
    }

    /// Forcefully sets the used program to the current program
    pub fn force_set_use(&mut self) {
        self.using = true;
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    /// Drops the shader program
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

// For compute shaders, we need a dispatch function
impl ShaderProgram<ComputeShader> {
    pub fn dispatch_compute(&mut self, x: u32, y: u32, z: u32) {
        unsafe {
            gl::DispatchCompute(x, y, z);
            gl::MemoryBarrier(gl::ALL_BARRIER_BITS);
        }
    }
}

// Uniform related functions
impl<ST: ShaderType> ShaderProgram<ST> {
    fn handle_uniforms(&mut self, force: bool) {
        for ref_uniform in self.uniforms.values_mut() {
            let mut uniform = ref_uniform.borrow_mut();
            if force || uniform.is_dirty() {
                uniform.clear_dirty();
                uniform.load_uniform();
            }
        }
    }

    fn link_all_uniforms(&mut self, logger: &mut HTMLLogger) {
        logger.open_scope("Uniform Linking ".yellow());

        let all_uniforms = self
            .shaders
            .iter()
            .flat_map(|shader| shader.data.uniforms.iter())
            .cloned()
            .collect::<Vec<_>>();

        for uniform in all_uniforms {
            match uniform.ty.as_str() {
                // Literals
                "bool" => self.add_uniform::<bool>(logger, &uniform.name, "bool", false),
                "int" => self.add_uniform::<i32>(logger, &uniform.name, "int", 0),
                "uint" => self.add_uniform::<u32>(logger, &uniform.name, "uint", 0),
                "float" => self.add_uniform::<f32>(logger, &uniform.name, "float", 0.0),
                "double" => self.add_uniform::<f64>(logger, &uniform.name, "double", 0.0),

                // Vectors
                "bvec2" => {
                    self.add_uniform::<[bool; 2]>(logger, &uniform.name, "bvec2", [false; 2])
                }
                "bvec3" => {
                    self.add_uniform::<[bool; 3]>(logger, &uniform.name, "bvec3", [false; 3])
                }
                "bvec4" => {
                    self.add_uniform::<[bool; 4]>(logger, &uniform.name, "bvec4", [false; 4])
                }
                "ivec2" => self.add_uniform::<[i32; 2]>(logger, &uniform.name, "ivec2", [0; 2]),
                "ivec3" => self.add_uniform::<[i32; 3]>(logger, &uniform.name, "ivec3", [0; 3]),
                "ivec4" => self.add_uniform::<[i32; 4]>(logger, &uniform.name, "ivec4", [0; 4]),
                "uvec2" => self.add_uniform::<[u32; 2]>(logger, &uniform.name, "uvec2", [0; 2]),
                "uvec3" => self.add_uniform::<[u32; 3]>(logger, &uniform.name, "uvec3", [0; 3]),
                "uvec4" => self.add_uniform::<[u32; 4]>(logger, &uniform.name, "uvec4", [0; 4]),
                "vec2" => self.add_uniform::<[f32; 2]>(logger, &uniform.name, "vec2", [0.0; 2]),
                "vec3" => self.add_uniform::<[f32; 3]>(logger, &uniform.name, "vec3", [0.0; 3]),
                "vec4" => self.add_uniform::<[f32; 4]>(logger, &uniform.name, "vec4", [0.0; 4]),
                "dvec2" => self.add_uniform::<[f64; 2]>(logger, &uniform.name, "dvec2", [0.0; 2]),
                "dvec3" => self.add_uniform::<[f64; 3]>(logger, &uniform.name, "dvec3", [0.0; 3]),
                "dvec4" => self.add_uniform::<[f64; 4]>(logger, &uniform.name, "dvec4", [0.0; 4]),

                // Images and samplers
                "image1D" => self.add_image::<Image1D>(logger, &uniform.name),
                "image2D" => self.add_image::<Image2D>(logger, &uniform.name),
                "image3D" => self.add_image::<Image3D>(logger, &uniform.name),
                "sampler1D" => self.add_image::<Image1D>(logger, &uniform.name),
                "sampler2D" => self.add_image::<Image2D>(logger, &uniform.name),
                "sampler3D" => self.add_image::<Image3D>(logger, &uniform.name),
                _ => panic!("Unknown uniform type: {}", uniform.ty),
            }
        }

        logger.close_scope();
    }

    fn add_uniform<T: 'static>(&mut self, logger: &mut HTMLLogger, name: &str, ty: &str, initial: T)
    where
        UniformVariable<T>: Uniform,
    {
        let uniform = UniformVariable::new(name, ty, initial);
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

    pub fn get_uniform<T: 'static>(&self, name: &str) -> Option<Rc<RefCell<UniformVariable<T>>>>
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

    fn add_image<T: 'static>(&mut self, logger: &mut HTMLLogger, name: &str) {
        let uniform_name = CString::new(name).unwrap();
        let location = unsafe { gl::GetUniformLocation(self.id, uniform_name.as_ptr()) };
        self.images.insert(name.to_string(), location);
        println!("{}", location);
    }

    pub fn get_image_location(&self, name: &str) -> Option<GLint> {
        self.images.get(name).cloned()
    }
}

macro_rules! generate_shader_reload_functions {
    ($shader_type:ty, [$($field:ident),+]) => {
        #[allow(dead_code)]
        impl ShaderProgram<$shader_type> {
            /// Checks every watchers for the shaders, so whenever if a file is updated, the shaders are reloaded
            pub fn check_watchers(&mut self, logger: &mut HTMLLogger) {
                let mut any_updated = false;
                for shader in self.shaders.iter_mut() {
                    for watcher in shader.watchers.iter_mut() {
                        let updated = watcher.update();
                        if !updated {
                            continue;
                        }

                        any_updated = true;
                        logger.info("Update on ".cyan() + quote!(watcher.path).magenta());
                    }
                }

                if any_updated {
                    let main_scope = logger.open_scope("Reloading ".yellow() + self.name.magenta());
                    let shader_result =
                        ShaderProgram::<$shader_type>::new(logger, &self.name, $(&self.type_data.$field),+);
                    self.try_reload(logger, shader_result, main_scope);
                }
            }
        }
    };
}

generate_shader_reload_functions!(GraphicsShader, [vertex_file, fragment_file]);
generate_shader_reload_functions!(ComputeShader, [compute_file]);

impl<ST: ShaderType> ShaderProgram<ST> {
    /// Tries to reload the shaders
    fn try_reload(
        &mut self,
        logger: &mut HTMLLogger,
        new_shader: Result<ShaderProgram<ST>, String>,
        main_scope: Rc<RefCell<Details>>,
    ) {
        match new_shader {
            Ok(mut new_shader_program) => {
                // In case the shader loaded correctly, check if the uniforms are the same
                // For each old uniform, we find the corresponding new uniform

                logger.open_scope("Reloading Uniforms".yellow());
                for uniform in self.uniforms.values() {
                    let uniform_str = uniform.borrow().to_string();
                    let (name, _) = uniform_str.split_once(": ").unwrap();

                    // If the new uniform is found, then, if the types are the same, we exchange it
                    if let Some(new_uniform) = new_shader_program.uniforms.get(name) {
                        let new_uniform_str = new_uniform.borrow().to_string();
                        if new_uniform_str == uniform_str {
                            new_shader_program
                                .uniforms
                                .insert(name.to_string(), uniform.clone());
                            logger.info("Uniform ".cyan() + name.magenta() + " reloaded".green());
                        } else {
                            logger.info("Uniform ".cyan() + name.magenta() + " type changed".red());
                        }
                    } else {
                        logger.info("Uniform ".cyan() + name.magenta() + " not found".red());
                    }
                }
                logger.close_scope();

                self.drop();
                let ShaderProgram {
                    id,
                    shaders,
                    uniforms,
                    ..
                } = new_shader_program;
                self.id = id;
                self.using = false;
                self.shaders = shaders;
                self.uniforms = uniforms;

                self.force_set_use();
                self.handle_uniforms(true);
                self.toggle_use();

                main_scope
                    .borrow_mut()
                    .summary
                    .text
                    .push_str(" Success".green().as_str());
            }
            Err(_) => {
                main_scope
                    .borrow_mut()
                    .summary
                    .text
                    .push_str(" Failed".red().as_str());
            }
        };

        logger.close_scope();
        logger.to_html();
    }
}

/// Checks if a program has compiled successfully or not
fn check_program(
    logger: &mut HTMLLogger,
    summary: &mut Summary,
    program: GLuint,
) -> Result<(), String> {
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

        summary.text.push_str(" Failed".red().as_str());

        let error_message = error_message.to_string_lossy().into_owned();
        return Err(error_message);
    }

    summary.text.push_str(" Success".green().as_str());
    logger.close_scope();
    Ok(())
}
