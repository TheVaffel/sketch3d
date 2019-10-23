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
use crate::shaders;
use crate::objects;
use crate::lineobjects;
use crate::splinedraw;
use crate::cyllinder;
use crate::utils;
use crate::edit;

pub struct MouseState {
    pub pos: glm::Vec2,
    pub button1_pressed: bool,
    pub button1_was_pressed: bool,
    pub in_window: bool,
}

impl MouseState {
    fn tick(self : &mut MouseState) {
	self.button1_was_pressed = self.button1_pressed;
    }
}

pub enum ProgramState {
    Draw,
    Edit,
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

fn handle_draw_operation(mut spline_state : splinedraw::SplineState,
			 cyllinders : &mut Vec<cyllinder::GeneralizedCyllinder>,
			 mouse_state : &MouseState,
			 key_state : &KeyState) -> (ProgramState, splinedraw::SplineState) {
    if key_state.enter {
	println!("Enter state true");
	if spline_state.control_points.len() > 2 {
	    let cyllinder_object = cyllinder::create_cyllinder(0.3, 5, spline_state);
	    
	    cyllinders.push(cyllinder_object);
	    
	    return (ProgramState::Edit, splinedraw::SplineState::new());
	}
    } else {
	splinedraw::handle_spline_draw(&mouse_state, &mut spline_state);
    }

    
    (ProgramState::Draw, spline_state)
}

fn handle_edit_operation(cyllinder : &mut cyllinder::GeneralizedCyllinder,
			 proj : &glm::Mat4, mouse_state : &MouseState,
			 _key_state : &KeyState, edit_state : &mut edit::EditState) {
    
    if mouse_state.button1_pressed {
	
	
	let selected_point_ind = edit::select_point(mouse_state.pos,
						    &cyllinder.spline.control_points,
						    proj, 0.03);

	if selected_point_ind < 0 {
	    for i in &edit_state.selected_indices {
		cyllinder.spline.point_colors[*i] = glm::vec4(0.0, 0.0, 0.0, 1.0);
	    }
	    edit_state.selected_indices.clear();
	    cyllinder.spline.update_gpu_state();
	    return;
	}
	
	let mut already_chosen = false;
	for i in &edit_state.selected_indices {
	    if *i == selected_point_ind as usize {
		already_chosen = true;
		break;
	    }
	}
	
	if !mouse_state.button1_was_pressed {
	    if already_chosen {
		edit_state.ref_point = edit::normalize_point(mouse_state.pos);
	    }
	} else {
	    let new_mpoint = edit::normalize_point(mouse_state.pos);
	    let diff = new_mpoint - edit_state.ref_point;
	    for i in &edit_state.selected_indices {
		cyllinder.spline.control_points[*i] =
		    cyllinder.spline.control_points[*i] + glm::vec3(diff.x, diff.y, 0.0);
	    }
	    edit_state.ref_point = new_mpoint;
	}
	
	if selected_point_ind >= 0 && !already_chosen {	
	    cyllinder.spline.point_colors[selected_point_ind as usize] = glm::vec4(1.0, 0.0, 0.0, 1.0);
	    edit_state.selected_indices.push(selected_point_ind as usize);
	    cyllinder.spline.update_gpu_state();
	}
    }

    
}

pub fn run_loop(mut glfw_state: GLFWState, modeler_state: ModelerState) {

    let mut mouse_state = MouseState { pos: glm::vec2(0.0, 0.0),
				       button1_pressed: false,
				       button1_was_pressed: false,
				       in_window: true, };
    let mut key_state = KeyState { enter: false, };

    let mut edit_state = edit::EditState { selected_indices : Vec::new(),
					   ref_point : glm::vec2(0.0, 0.0)};

    let mut spline_state = splinedraw::SplineState::new();

    let mut line_objects = Vec::new();
    for i in 0..modeler_state.objects.len() {
	line_objects.push(lineobjects::create_line_object(&modeler_state.objects[i].vertices,
							  &modeler_state.objects[i].indices));
    }
    

    let mut cyllinders : Vec<cyllinder::GeneralizedCyllinder> = Vec::new();
    
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
	&CString::new(include_str!("shaders/body.vert")).unwrap(),
	&CString::new(include_str!("shaders/body.frag")).unwrap()
    ).unwrap();

    let screen_line_program = shaders::create_simple_shader(
	&CString::new(include_str!("shaders/screen_line.vert")).unwrap(),
	&CString::new(include_str!("shaders/screen_line.frag")).unwrap()
    ).unwrap();

    let world_line_program = shaders::create_simple_shader(
	&CString::new(include_str!("shaders/line.vert")).unwrap(),
	&CString::new(include_str!("shaders/line.frag")).unwrap()
    ).unwrap();

    
    shader_program.activate();

    /* let transform_location = unsafe {
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
    let small_translation = glm::vec4(0.0, 0.0, -0.08, 0.0); */

	
    let mut _count = 0;

    let mut program_state = ProgramState::Draw;
    
    // Loop until the user closes the window
    while !glfw_state.window.should_close() {

	_count += 1;
	
	let r :f32 = 2.0;
	// let phi : f32 = 0.5;
	let phi : f32 = 0.0;
	// let th :f32 = count as f32 / 50.7;
	let th : f32 = 0.0;

	/* let proj = glm::ext::perspective(30.0, 4.0 / 3.0,
					  0.1,
	100.0);*/
	let proj = utils::ortho(-1.0, 1.0, -1.0, 1.0, -100.0, 100.0);
	
	let trans = proj *
	    glm::ext::look_at(glm::vec3(r * th.sin() * phi.cos(),
					-r * phi.sin(),
					r * th.cos() * phi.cos()),
			      glm::vec3(0.0, 0.0, 0.0),
			      glm::vec3(0.0, 1.0, 0.0));

	unsafe {
	    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
	}

	shader_program.activate();
	
	// unsafe {
	    // for i in 0..modeler_state.objects.len() {
	    for i in 0..cyllinders.len() {
		let model_trans = glm::ext::translate(&glm::Matrix4::one(),
						      glm::vec3(0.0, 0.0,
								// (i as f32 - modeler_state.objects.len() as f32
								(i as f32 - cyllinders.len() as f32
								 + 1.0) * 2.5));
		
		let trans = trans * model_trans; 
		
		/* gl::UniformMatrix4fv(transform_location,
				     1, gl::FALSE, &trans[0][0]); */

		
		cyllinder::draw_cyllinder(&cyllinders[i],
					  &shader_program,
					  &world_line_program,
					  &trans);
		
		/* 
		
		gl::Uniform4fv(displacement_location,
			       1, &no_translation[0]);
		gl::Uniform4fv(color_location,
			       1, &black_color[0]);

		gl::LineWidth(1.0);
		gl::BindVertexArray(line_objects[i].all_vao);
		gl::DrawElements(
		    gl::LINES,
		    line_objects[i].all_indices.len() as gl::types::GLsizei,
		    gl::UNSIGNED_INT,
		std::ptr::null());*/


		/* gl::Uniform4fv(displacement_location,
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
		    std::ptr::null()); */
	    // }
	}
	
	// Poll for and process events
	glfw_state.glfw.poll_events();

	let flushed_events = glfw::flush_messages(&glfw_state.events);

	mouse_state.tick();
	
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

	match program_state  {
	    ProgramState::Draw => {
		let (t_program_state, t_spline_state) = handle_draw_operation(spline_state, &mut cyllinders,
									      &mouse_state, &key_state);

		program_state = t_program_state;
		spline_state = t_spline_state;
		
		screen_line_program.activate();
		spline_state.update_gpu_state();
		splinedraw::draw_spline_lines(&spline_state);
	    },
	    ProgramState::Edit => {
		handle_edit_operation(&mut cyllinders[0], &proj, &mouse_state, &key_state, &mut edit_state);
	    }
	}

	
	
        // Swap front and back buffers
        glfw_state.window.swap_buffers();

	std::thread::sleep(std::time::Duration::from_millis(17));
	
    }
}

