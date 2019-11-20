#[macro_use]
extern crate lazy_static;

// extern crate glfw;
extern crate gl;
extern crate nalgebra as na;
extern crate imgui_glfw_rs;

use imgui_glfw_rs::imgui::Context as ImContext;
use imgui_glfw_rs::glfw::{self,Context};

mod program;
mod settings;
mod shaders;
mod objects;
mod lineobjects;
mod splinedraw;
mod cyllinder;
mod edit;
mod utils;
mod laplacian;
mod gui;
mod annotation;

pub struct Object {
    vao: gl::types::GLuint, 
    vbo: gl::types::GLuint,
    ebo: gl::types::GLuint,
    vertices: Vec<f32>,
    indices: Vec<u32>,
}

pub struct ModelerState {
    objects : Vec<Object>,
}

pub struct GLFWState {
    glfw:   glfw::Glfw,
    window: imgui_glfw_rs::glfw::Window,
    events: std::sync::mpsc::Receiver<(f64, glfw::WindowEvent)>,
    imgui_context: imgui_glfw_rs::imgui::Context,
    imgui_glfw_context: imgui_glfw_rs::ImguiGLFW,
}

fn init_glfw() -> GLFWState  {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(settings::OPENGL_MAJOR_VERSION));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(settings::OPENGL_MINOR_VERSION));
    glfw.window_hint(glfw::WindowHint::OpenGlProfile(glfw::OpenGlProfileHint::Core));

    glfw.window_hint(glfw::WindowHint::Resizable(false));
    glfw.window_hint(glfw::WindowHint::Samples(Some(4)));

    
    
    // Create a windowed mode window and its OpenGL context
    let (mut window, events) = glfw.create_window(settings::WINDOW_WIDTH, settings::WINDOW_HEIGHT, "Hello this is window", glfw::WindowMode::Windowed)
		     .expect("Failed to create GLFW window.");

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);
    
    
    // Make the window's context current
    window.make_current();
    window.set_key_polling(true);
    window.set_cursor_pos_polling(true);
    window.set_mouse_button_polling(true);
    window.set_cursor_enter_polling(true);

    let mut imgui_context = ImContext::create();
    let imgui_glfw_context = imgui_glfw_rs::ImguiGLFW::new(&mut imgui_context, &mut window);

    
    GLFWState{ window, events, glfw, imgui_context, imgui_glfw_context }
}

fn main() {  

    let glfw_state = init_glfw();
    
    let models = program::setup_objects();
    
    program::run_loop(glfw_state, models);
}
