use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use shader_engine::image_buffer::Image2D;
use shader_engine::raw_model::RawModel;
use shader_engine::shader::{ComputeShader, GraphicsShader, ShaderProgram};
use shader_engine::utils::html_logger::HTMLLogger;
use shader_engine::{MainLoopResult, ShaderEngine};
use std::error;

const START_WIDTH: i32 = 800;
const START_HEIGHT: i32 = 800;

fn main() -> Result<(), Box<dyn error::Error>> {
    let mut shader_engine = ShaderEngine::create_window("Ray Tracing", START_WIDTH, START_HEIGHT)?;

    // Set the window features
    {
        shader_engine.window_builder.resizable();
    }
    shader_engine.finalize_window()?;

    let mut html_logger = HTMLLogger::new("Ray Tracing");

    let mut ray_tracing_compute =
        ShaderProgram::<ComputeShader>::new(&mut html_logger, "RT Shader", "main.comp")?;

    let resolution_uniform = ray_tracing_compute
        .get_uniform::<[f32; 2]>("resolution")
        .unwrap();
    let time_uniform = ray_tracing_compute.get_uniform::<f32>("time").unwrap();

    // Set resolution uniform
    {
        resolution_uniform
            .borrow_mut()
            .get_bind()
            .set([START_WIDTH as f32, START_HEIGHT as f32]);
    }

    // Necessary images for fragment shader, required for noise reduction
    let mut screen_image = Image2D::new(
        START_WIDTH,
        START_HEIGHT,
        gl::WRITE_ONLY,
        gl::RGBA32F,
        gl::RGBA32F,
    );

    let mut frame_count_image = Image2D::new(
        START_WIDTH,
        START_HEIGHT,
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
    )?;

    shader_engine.set_loop(
        move |event_pump: &mut EventPump, elapsed_time: f64| -> MainLoopResult {
            // Check shaders for updates, any change will update the shaders
            ray_tracing_compute.check_watchers(&mut html_logger);
            quad_shader.check_watchers(&mut html_logger);

            // Update the time uniform depending on the elapsed time
            {
                let mut ref_timer = time_uniform.borrow_mut();
                let timer_bind = ref_timer.get_bind();
                *timer_bind += elapsed_time as f32;
            }

            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } => return MainLoopResult::Quit,
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => return MainLoopResult::Quit,
                    Event::Window {
                        win_event: WindowEvent::Resized(width, height),
                        ..
                    } => {
                        // Resize the resolution uniform
                        resolution_uniform
                            .borrow_mut()
                            .get_bind()
                            .set([width as f32, height as f32]);

                        // Resize the images
                        screen_image = screen_image.clone_with_size(width, height);
                        frame_count_image = frame_count_image.clone_with_size(width, height);

                        // Resize the screen manager, this will also update the viewport
                        return MainLoopResult::Resize(width, height);
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

            MainLoopResult::Continue
        },
    );

    Ok(())
}
