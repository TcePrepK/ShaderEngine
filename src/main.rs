extern crate gl;
extern crate sdl2;
mod raw_model;
mod shader;
mod timer;
mod utils;

use crate::raw_model::RawModel;
use crate::shader::ShaderProgram;
use crate::timer::Timer;
use crate::utils::html_logger::HTMLLogger;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::video::GLProfile;
use std::os::raw;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_sub_system = sdl.video().unwrap();

    let window = video_sub_system
        .window("My Cool Screen", 800, 800)
        .resizable()
        .opengl()
        .build()
        .unwrap();
    let _gl_context = window.gl_create_context().unwrap();
    gl::load_with(|s| video_sub_system.gl_get_proc_address(s) as *const raw::c_void);

    let gl_attribute = video_sub_system.gl_attr();
    gl_attribute.set_context_profile(GLProfile::Core);
    gl_attribute.set_context_version(4, 6);

    unsafe {
        gl::Viewport(0, 0, 800, 800);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    let mut html_logger = HTMLLogger::new("Quad Shader");
    let mut quad_shader =
        ShaderProgram::generate_graphics(&mut html_logger, "Quad Shader", "quad.vert", "quad.frag")
            .unwrap();
    let quad_model = RawModel::from_vertices(&[-1.0, -1.0, 3.0, -1.0, -1.0, 3.0], &[0, 1, 2]);

    let resolution_uniform = quad_shader.get_uniform::<[f32; 2]>("resolution").unwrap();
    let time_uniform = quad_shader.get_uniform::<f32>("time").unwrap();

    resolution_uniform
        .borrow_mut()
        .get_bind()
        .set([800.0, 800.0]);

    let mut event_pump = sdl.event_pump().unwrap();
    'main: loop {
        let timer = Timer::new();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main,
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'main,
                Event::Window {
                    win_event: WindowEvent::Resized(width, height),
                    ..
                } => {
                    // When resize happens, update the viewport and resolution uniform
                    resolution_uniform
                        .borrow_mut()
                        .get_bind()
                        .set([width as f32, height as f32]);

                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
                }
                Event::KeyDown {
                    keycode: Some(Keycode::R),
                    keymod,
                    ..
                } => {
                    if keymod.contains(Mod::LSHIFTMOD) {
                        quad_shader.try_reload(&mut html_logger);
                    }
                }
                _ => {}
            }
        }

        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        quad_shader.toggle_use();
        quad_model.render();
        quad_shader.toggle_use();

        window.gl_swap_window();

        let mut ref_timer = time_uniform.borrow_mut();
        let timer_bind = ref_timer.get_bind();
        *timer_bind += timer.elapsed() as f32;
    }
}
