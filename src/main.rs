extern crate gl;
extern crate sdl2;
mod image_buffer;
mod raw_model;
mod shader;
mod timer;
mod utils;

use crate::image_buffer::Image2D;
use crate::raw_model::RawModel;
use crate::shader::{ComputeShader, GraphicsShader, ShaderProgram};
use crate::timer::Timer;
use crate::utils::html_logger::HTMLLogger;
use gl::types::GLsizei;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::video::GLProfile;
use std::os::raw;

const START_WIDTH: u32 = 800;
const START_HEIGHT: u32 = 800;

fn main() {
    let sdl = sdl2::init().unwrap();
    let video_sub_system = sdl.video().unwrap();

    let window = video_sub_system
        .window("My Cool Screen", START_WIDTH, START_HEIGHT)
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
        gl::Viewport(0, 0, START_WIDTH as GLsizei, START_HEIGHT as GLsizei);
        gl::ClearColor(0.0, 0.0, 0.0, 1.0);
    }

    let mut html_logger = HTMLLogger::new("Quad Shader");
    let mut ray_tracing_compute =
        ShaderProgram::<ComputeShader>::new(&mut html_logger, "RT Shader", "ray_tracing/main.comp")
            .unwrap();

    let resolution_uniform = ray_tracing_compute
        .get_uniform::<[f32; 2]>("resolution")
        .unwrap();
    let time_uniform = ray_tracing_compute.get_uniform::<f32>("time").unwrap();
    // let screen_image_uniform = ray_tracing_compute
    //     .get_uniform::<Image2D>("screen")
    //     .unwrap();

    // Set resolution uniform
    {
        resolution_uniform
            .borrow_mut()
            .get_bind()
            .set([START_WIDTH as f32, START_HEIGHT as f32]);
    }

    // Necessary images for fragment shader, required for noise reduction
    let mut screen_image = Image2D::new(
        START_WIDTH as i32,
        START_HEIGHT as i32,
        gl::WRITE_ONLY,
        gl::RGBA32F,
        gl::RGBA32F,
    );

    let mut frame_count_image = Image2D::new(
        START_WIDTH as i32,
        START_HEIGHT as i32,
        gl::READ_WRITE,
        gl::R32F,
        gl::R32F,
    );

    // Quad Shader Part
    let quad_model = RawModel::from_vertices(&[-1.0, -1.0, 3.0, -1.0, -1.0, 3.0], &[0, 1, 2]);
    let mut quad_shader = ShaderProgram::<GraphicsShader>::new(
        &mut html_logger,
        "Quad Shader",
        "quad.vert",
        "quad.frag",
    )
    .unwrap();

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

                    // Then update the screen image
                    screen_image =
                        screen_image.clone_with_size(START_WIDTH as i32, START_HEIGHT as i32);
                    frame_count_image =
                        frame_count_image.clone_with_size(START_WIDTH as i32, START_HEIGHT as i32);
                }
                _ => {}
            }
        }

        // Ray Tracing
        {
            ray_tracing_compute.toggle_use();

            screen_image.bind_as_image(0);
            frame_count_image.bind_as_image(1);

            let mut borrow = resolution_uniform.borrow_mut();
            let resolution = borrow.get_bind().get();
            ray_tracing_compute.dispatch_compute(
                (resolution[0] / 8.0).ceil() as u32,
                (resolution[1] / 4.0).ceil() as u32,
                1,
            );

            ray_tracing_compute.toggle_use();
        }

        // Rendering
        {
            quad_shader.toggle_use();

            screen_image.bind_as_sampler(0);
            frame_count_image.bind_as_sampler(1);

            quad_model.render();

            quad_shader.toggle_use();
        }

        window.gl_swap_window();

        // Check shaders for updates, this function updates the shaders if any change is detected
        ray_tracing_compute.check_watchers(&mut html_logger);
        quad_shader.check_watchers(&mut html_logger);

        let mut ref_timer = time_uniform.borrow_mut();
        let timer_bind = ref_timer.get_bind();
        *timer_bind += timer.elapsed() as f32;
    }
}
