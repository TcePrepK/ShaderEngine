use crate::shader::bindable::Bindable;
use gl::types::{GLdouble, GLfloat, GLint, GLuint};
use std::any::Any;
use std::ffi::CString;

pub struct UniformVariable<T> {
    name: String,
    ty: String,
    location: Option<GLint>,
    bind: Bindable<T>,
}

#[allow(dead_code)]
pub trait Uniform: Any {
    fn load_uniform(&self);
    fn is_dirty(&self) -> bool;
    fn clear_dirty(&mut self);

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn to_string(&self) -> String;
}

impl<T> UniformVariable<T> {
    pub fn new(name: &str, ty: &str, initial: T) -> UniformVariable<T> {
        UniformVariable {
            name: name.to_string(),
            ty: ty.to_string(),
            location: None,
            bind: Bindable::new(initial),
        }
    }

    pub fn bind_program(&mut self, program: GLuint) -> Result<(), ()> {
        let uniform_name = CString::new(self.name.as_str()).unwrap();
        let location = unsafe { gl::GetUniformLocation(program, uniform_name.as_ptr()) };
        if location == -1 {
            return Err(());
        }
        self.location = Some(location);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_location(&self) -> Option<GLint> {
        self.location
    }

    pub fn get_bind(&mut self) -> &mut Bindable<T> {
        &mut self.bind
    }

    pub fn get_value(&self) -> &T {
        self.bind.get()
    }
}

impl<T: 'static> Uniform for UniformVariable<T>
where
    UniformVariable<T>: LoadableUniform,
{
    fn load_uniform(&self) {
        if self.location.is_some() {
            LoadableUniform::load_uniform(self);
        }
    }

    fn is_dirty(&self) -> bool {
        self.bind.is_dirty()
    }

    fn clear_dirty(&mut self) {
        self.bind.clear_dirty();
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn to_string(&self) -> String {
        format!("{}: {}", self.name, self.ty)
    }
}

pub trait LoadableUniform {
    fn load_uniform(&self);
}

macro_rules! impl_uniform_loader {
    ($ty:ty, [1], $gl_func:ident, $gl_type:ty) => {
        impl LoadableUniform for UniformVariable<$ty> {
            fn load_uniform(&self) {
                unsafe {
                    gl::$gl_func(self.location.unwrap(), *self.get_value() as $gl_type);
                }
            }
        }
    };

    ($ty:ty, [$len:expr], $gl_func:ident, $gl_type:ty) => {
        impl LoadableUniform for UniformVariable<[$ty; $len]> {
            fn load_uniform(&self) {
                let converted = self.get_value().map(|v| v as $gl_type);
                unsafe {
                    gl::$gl_func(self.location.unwrap(), 1, converted.as_ptr());
                }
            }
        }
    };
}

// Scalar types
impl_uniform_loader!(bool, [1], Uniform1i, GLint);
impl_uniform_loader!(i32, [1], Uniform1i, GLint);
impl_uniform_loader!(u32, [1], Uniform1ui, GLuint);
impl_uniform_loader!(f32, [1], Uniform1f, GLfloat);
impl_uniform_loader!(f64, [1], Uniform1d, GLdouble);

// Vector types
impl_uniform_loader!(bool, [2], Uniform2iv, GLint);
impl_uniform_loader!(bool, [3], Uniform3iv, GLint);
impl_uniform_loader!(bool, [4], Uniform4iv, GLint);
impl_uniform_loader!(i32, [2], Uniform2iv, GLint);
impl_uniform_loader!(i32, [3], Uniform3iv, GLint);
impl_uniform_loader!(i32, [4], Uniform4iv, GLint);
impl_uniform_loader!(u32, [2], Uniform2uiv, GLuint);
impl_uniform_loader!(u32, [3], Uniform3uiv, GLuint);
impl_uniform_loader!(u32, [4], Uniform4uiv, GLuint);
impl_uniform_loader!(f32, [2], Uniform2fv, GLfloat);
impl_uniform_loader!(f32, [3], Uniform3fv, GLfloat);
impl_uniform_loader!(f32, [4], Uniform4fv, GLfloat);
impl_uniform_loader!(f64, [2], Uniform2dv, GLdouble);
impl_uniform_loader!(f64, [3], Uniform3dv, GLdouble);
impl_uniform_loader!(f64, [4], Uniform4dv, GLdouble);
