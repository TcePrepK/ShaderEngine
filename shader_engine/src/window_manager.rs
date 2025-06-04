use gl::types::GLsizei;
use sdl2::video::{GLContext, GLProfile, Window, WindowBuildError, WindowBuilder};
use sdl2::{Sdl, VideoSubsystem};
use std::ops::{Deref, DerefMut};
use std::os::raw;
// pub trait ScreenManagerBuilder {
//     fn build_screen_manager(&mut self) -> Result<ScreenManager, String>;
// }
//
// pub struct ScreenManager {
//     window: Window,
//     width: u32,
//     height: u32,
// }
//
// impl ScreenManager {
//     pub fn new(name: &str, initial_width: u32, initial_height: u32) -> WindowBuilder {
//         let sdl = sdl2::init().unwrap();
//         let video_sub_system = sdl.video().unwrap();
//
//         video_sub_system.window(name, initial_width, initial_height)
//     }
// }
//
// impl ScreenManagerBuilder for WindowBuilder {
//     fn build_screen_manager(&mut self) -> Result<ScreenManager, WindowBuildError> {
//         let window = self.build()?;
//
//         let _gl_context = window.gl_create_context().unwrap();
//         gl::load_with(|s| video_sub_system.gl_get_proc_address(s) as *const raw::c_void);
//
//         let gl_attribute = video_sub_system.gl_attr();
//         gl_attribute.set_context_profile(GLProfile::Core);
//         gl_attribute.set_context_version(4, 6);
//
//         unsafe {
//             gl::Viewport(0, 0, START_WIDTH as GLsizei, START_HEIGHT as GLsizei);
//             gl::ClearColor(0.0, 0.0, 0.0, 1.0);
//         }
//
//         let manager = ScreenManager {
//             window: self.build().unwrap(),
//             width: initial_width,
//             height: initial_height,
//         };
//
//         Ok(manager)
//     }
// }

pub struct ShaderWindow {
    pub window: Window,
    pub width: i32,
    pub height: i32,

    #[allow(dead_code)]
    gl_context: GLContext,
}

pub struct ShaderWindowBuilder {
    pub(crate) sdl: Sdl,
    video_subsystem: VideoSubsystem,
    window_builder: WindowBuilder,
    width: i32,
    height: i32,
}

impl ShaderWindow {
    pub(crate) fn get_builder(
        title: &str,
        width: i32,
        height: i32,
    ) -> Result<ShaderWindowBuilder, String> {
        let sdl = sdl2::init()?;
        let video_subsystem = sdl.video()?;
        let window_builder = video_subsystem.window(title, width as u32, height as u32);

        Ok(ShaderWindowBuilder {
            sdl,
            video_subsystem,
            window_builder,
            width,
            height,
        })
    }

    pub(crate) fn swap_window(&mut self) {
        self.window.gl_swap_window();
    }

    pub(crate) fn resize(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;

        unsafe {
            gl::Viewport(0, 0, width as GLsizei, height as GLsizei);
        }
    }
}

impl ShaderWindowBuilder {
    pub fn build_as_manager(&mut self) -> Result<ShaderWindow, WindowBuildError> {
        let window = self.window_builder.opengl().build()?;

        let gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| self.video_subsystem.gl_get_proc_address(s) as *const raw::c_void);

        let gl_attr = self.video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(4, 6);

        unsafe {
            gl::Viewport(0, 0, self.width as GLsizei, self.height as GLsizei);
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
        }

        Ok(ShaderWindow {
            window,
            width: self.width,
            height: self.height,

            gl_context,
        })
    }
}

impl Deref for ShaderWindowBuilder {
    type Target = WindowBuilder;
    fn deref(&self) -> &Self::Target {
        &self.window_builder
    }
}

impl DerefMut for ShaderWindowBuilder {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.window_builder
    }
}
