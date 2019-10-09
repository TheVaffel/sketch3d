extern crate glfw;
extern crate gl;
extern crate glm;

use glfw::{Context,Key,Action};
use crate::{ModelerState, Object, GLFWState};
use std::ffi::{CString};
use std::f32;

use crate::settings;
use crate::shaders::{self};
use crate::objects;

pub fn setup_objects() -> ModelerState {

    let vertices: Vec<f32> = vec![
	-0.5, -0.5, 0.0,
	0.5, -0.5, 0.0,
	0.0, 0.5, 0.0
    ];
    
    let indices: Vec<u32> = vec![
	0, 1, 2
    ];

    let triangle = objects::create_object(vertices, indices);

    let cube = objects::create_cube_object(0.5);
    let sphere = objects::create_sphere_object(0.5, 5);
    let cone = objects::create_cone_object(0.5, 2.0, 15);
    let cyllinder = objects::create_generalized_cyllinder_object(0.5, 2.0, 5, 5);

    ModelerState { objects: vec![cyllinder, cone, sphere, cube, triangle,],}
} 

pub fn run_loop(mut glfw_state: GLFWState, modeler_state: ModelerState) {
    
    unsafe {
	
	gl::Enable(gl::DEPTH_TEST);
	gl::DepthFunc(gl::LEQUAL);

	// Configure miscellaneous OpenGL settings
	gl::Enable(gl::CULL_FACE);
	
	gl::Viewport(0, 0, settings::WINDOW_WIDTH as i32, settings::WINDOW_HEIGHT as i32);
	gl::ClearColor(0.3, 0.3, 0.5, 1.0);
    }

    // Load shaders

    let shader_program = shaders::create_simple_shader(
	&CString::new(include_str!("shaders/simple.vert")).unwrap(),
	&CString::new(include_str!("shaders/simple.frag")).unwrap()
    ).unwrap();

    
    shader_program.activate();

    let transform_location = unsafe {
        gl::GetUniformLocation(shader_program.id,
                               CString::new("trans").unwrap().as_ptr())
    };

	
    let mut count = 0;
    
    // Loop until the user closes the window
    while !glfw_state.window.should_close() {

	count += 1;
	
	let r :f32 = 2.0;
	let phi :f32 = 0.5;
	let th :f32 = count as f32 / 50.7;
	
	let trans = glm::ext::perspective(30.0, 4.0 / 3.0,
					  0.1,
					  100.0) *
	    glm::ext::look_at(glm::vec3(r * th.sin() * phi.cos(),
					-r * phi.sin(),
					r * th.cos() * phi.cos()),
			      glm::vec3(0.0, 0.0, 0.0),
			      glm::vec3(0.0, 1.0, 0.0));

	unsafe {
	    gl::UniformMatrix4fv(transform_location,
				 1, gl::FALSE, &trans[0][0]);
	}
	
	unsafe {
	    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

	}

	unsafe {
	    gl::BindVertexArray(modeler_state.objects[0].vao);
	    gl::DrawElements(
		gl::TRIANGLES,
		modeler_state.objects[0].indices.len() as gl::types::GLsizei,
		gl::UNSIGNED_INT,
		std::ptr::null());
	}
	
	// Poll for and process events
	glfw_state.glfw.poll_events();
	
        for (_, event) in glfw::flush_messages(&glfw_state.events) {
            println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    glfw_state.window.set_should_close(true)
                },
                _ => {},
            }
        }
	
        // Swap front and back buffers
        glfw_state.window.swap_buffers();

	std::thread::sleep(std::time::Duration::from_millis(40));
	
    }
}

