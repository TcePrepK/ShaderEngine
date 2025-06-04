use crate::timer::Timer;
use crate::window_manager::{ShaderWindow, ShaderWindowBuilder};
use gl::types::GLsizei;
use sdl2::video::WindowBuildError;
use sdl2::EventPump;

pub mod image_buffer;
pub mod raw_model;
pub mod shader;
pub mod timer;
pub mod utils;
pub mod window_manager;

pub enum MainLoopResult {
    Quit,
    Continue,
    Resize(i32, i32),
}

pub struct ShaderEngine {
    window_created: bool,

    total_running_timer: Timer,
    inner_timer: Timer,

    pub window_manager: Option<ShaderWindow>,
    pub window_builder: ShaderWindowBuilder,
}

impl ShaderEngine {
    pub fn create_window(title: &str, width: i32, height: i32) -> Result<ShaderEngine, String> {
        let builder = ShaderWindow::get_builder(title, width, height)?;
        Ok(Self {
            window_created: false,

            total_running_timer: Timer::new(),
            inner_timer: Timer::new(),

            window_manager: None,
            window_builder: builder,
        })
    }

    pub fn finalize_window(&mut self) -> Result<(), WindowBuildError> {
        let screen_manager = self.window_builder.build_as_manager()?;

        self.window_created = true;
        self.window_manager = Some(screen_manager);

        Ok(())
    }

    pub fn set_loop<F>(&mut self, mut loop_function: F)
    where
        F: FnMut(&mut EventPump, f64) -> MainLoopResult,
    {
        if !self.window_created {
            panic!("Window not created, call `finalize_window` first");
        }

        // Start the timer
        self.total_running_timer.update();
        let mut elapsed_time = 0.0;

        // Start the main loop
        let mut event_pump = self.window_builder.sdl.event_pump().unwrap();
        loop {
            // Start the inner timer
            self.inner_timer.update();

            // Clear color bit
            unsafe {
                gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            }

            // Update the loop function
            let result = loop_function(&mut event_pump, elapsed_time);

            // Handle the result
            match result {
                MainLoopResult::Quit => break,
                MainLoopResult::Continue => self.window_manager.as_mut().unwrap().swap_window(),
                MainLoopResult::Resize(width, height) => {
                    self.resize(width, height);
                }
            }

            // Update the elapsed time
            elapsed_time = self.inner_timer.elapsed();
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        if !self.window_created {
            panic!("Window not created, call `finalize_window` first");
        }

        let window_manager = self.window_manager.as_mut().unwrap();
        window_manager.resize(width, height);

        unsafe {
            gl::Viewport(0, 0, width as GLsizei, height as GLsizei);
        }
    }
}
