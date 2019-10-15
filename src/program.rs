extern crate glfw;
extern crate gl;
extern crate glm;
extern crate num_traits;

use glfw::{Context,Key,Action};
use crate::{ModelerState, GLFWState};
use std::ffi::{CString};
use std::f32;
use num_traits::identities::One;

use crate::settings;
use crate::shaders::{self};
use crate::objects;
use crate::lineobjects;
use crate::splinedraw;
use crate::cyllinder;

pub struct MouseState {
    pub pos: glm::Vec2,
    pub button1_pressed: bool,
    pub in_window: bool,
}


// What keys are pressed?
pub struct KeyState {
    pub enter: bool,
}

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
    // let cyllinder = cyllinder::create_generalized_cyllinder_object(0.5, 2.0, 20, 5);

    ModelerState { objects: vec![/*cyllinder, */ cone, sphere, cube, triangle,],}
}

pub fn run_loop(mut glfw_state: GLFWState, mut modeler_state: ModelerState) {

    let mut mouse_state = MouseState { pos: glm::vec2(0.0, 0.0),
				       button1_pressed: false,
				       in_window: true, };
    let mut key_state = KeyState { enter: false, };

    let mut spline_state = splinedraw::SplineState::new();

    let spline_coefficients = splinedraw::make_spline_coefficients();

    let mut line_objects = Vec::new();
    for i in 0..modeler_state.objects.len() {
	line_objects.push(lineobjects::create_line_object(&modeler_state.objects[i].vertices,
							 &modeler_state.objects[i].indices));
    }
    
    
    unsafe {
	
	gl::Enable(gl::DEPTH_TEST);
	gl::DepthFunc(gl::LEQUAL);

	// Configure miscellaneous OpenGL settings
	gl::Enable(gl::CULL_FACE);
	
	gl::Viewport(0, 0, settings::WINDOW_WIDTH as i32, settings::WINDOW_HEIGHT as i32);
	gl::ClearColor(1.0, 1.0, 1.0, 1.0);
    }

    // Load shaders

    let shader_program = shaders::create_simple_shader(
	&CString::new(include_str!("shaders/simple.vert")).unwrap(),
	&CString::new(include_str!("shaders/simple.frag")).unwrap()
    ).unwrap();

    let line_program = shaders::create_simple_shader(
	&CString::new(include_str!("shaders/screen_line.vert")).unwrap(),
	&CString::new(include_str!("shaders/screen_line.frag")).unwrap()
    ).unwrap();

    
    shader_program.activate();

    let transform_location = unsafe {
        gl::GetUniformLocation(shader_program.id,
                               CString::new("trans").unwrap().as_ptr())
    };

    let displacement_location = unsafe {
	gl::GetUniformLocation(shader_program.id,
			       CString::new("displacement").unwrap().as_ptr())
    };

    let color_location = unsafe {
	gl::GetUniformLocation(shader_program.id,
			       CString::new("uni_color").unwrap().as_ptr())
    };

    let black_color = glm::vec4(0.0, 0.0, 0.0, 1.0);
    let white_color = glm::vec4(1.0, 1.0, 1.0, 1.0);

    let no_translation = glm::vec4(0.0, 0.0, 0.0, 0.0);
    let small_translation = glm::vec4(0.0, 0.0, -0.08, 0.0);

	
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
	    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

	}

	shader_program.activate();
	
	unsafe {
	    for i in 0..modeler_state.objects.len() {
		let model_trans = glm::ext::translate(&glm::Matrix4::one(),
						      glm::vec3(0.0, 0.0, (i as f32 - modeler_state.objects.len() as f32  + 1.0) * 2.5));

		let trans = trans * model_trans;
		
		gl::UniformMatrix4fv(transform_location,
				     1, gl::FALSE, &trans[0][0]);
		gl::Uniform4fv(displacement_location,
			       1, &no_translation[0]);
		gl::Uniform4fv(color_location,
			       1, &black_color[0]);

		gl::LineWidth(4.0);
		gl::BindVertexArray(line_objects[i].all_vao);
		gl::DrawElements(
		    gl::LINES,
		    line_objects[i].all_indices.len() as gl::types::GLsizei,
		    gl::UNSIGNED_INT,
		    std::ptr::null());

		gl::Uniform4fv(displacement_location,
			       1, &small_translation[0]);
		gl::Uniform4fv(color_location,
			       1, &white_color[0]);
		
		gl::BindVertexArray(modeler_state.objects[i].vao);
		gl::DrawElements(
		    gl::TRIANGLES,
		    modeler_state.objects[i].indices.len() as gl::types::GLsizei,
		    gl::UNSIGNED_INT,
		    std::ptr::null());

		gl::Uniform4fv(color_location,
			       1, &black_color[0]);
		gl::LineWidth(2.0);

		gl::BindVertexArray(line_objects[i].vao);
		gl::DrawElements(
		    gl::LINES,
		    line_objects[i].indices.len() as gl::types::GLsizei,
		    gl::UNSIGNED_INT,
		    std::ptr::null());
	    }
	}
	
	// Poll for and process events
	glfw_state.glfw.poll_events();

	let flushed_events = glfw::flush_messages(&glfw_state.events);
	
        for (_, event) in flushed_events {
            // println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    glfw_state.window.set_should_close(true)
                },
		glfw::WindowEvent::Key(Key::Enter, _, action, _) => {
		    key_state.enter =
			match action {
			    Action::Release => false,
			    Action::Press => true,
			    Action::Repeat => key_state.enter
			};
		},
		glfw::WindowEvent::CursorPos(x, y) => {
		    mouse_state.pos = glm::vec2(x as f32, y as f32);
		},
		glfw::WindowEvent::CursorEnter(bb) => {
		    mouse_state.in_window = bb;
		},
		glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1,
					       Action::Press, _) => {
		    mouse_state.button1_pressed = true;
		},
		glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1,
					       Action::Release, _) => {
		    mouse_state.button1_pressed = false;
		},
                _ => {},
            }
        }

	if key_state.enter {
	    if spline_state.spline_points.len() > 0 {
		let cyllinder_object = cyllinder::create_cyllinder(0.5, 2.0, 20, 5, &spline_state);
		
		line_objects.push(lineobjects::create_line_object(&cyllinder_object.vertices,
								  &cyllinder_object.indices));
		modeler_state.objects.push(cyllinder_object);
	    }
	    spline_state = splinedraw::SplineState::new();
	} else {
	    splinedraw::handle_spline_draw(&mouse_state, &mut spline_state);
	}
	
	line_program.activate();
	spline_state.update_gpu_state(&spline_coefficients);
	splinedraw::draw_spline(&spline_state);
        // Swap front and back buffers
        glfw_state.window.swap_buffers();

	std::thread::sleep(std::time::Duration::from_millis(10));
	
    }
}

