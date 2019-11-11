extern crate imgui_glfw_rs;
extern crate gl;
extern crate glm;
extern crate num_traits;

use imgui_glfw_rs::glfw::{self,Context,Key,Action};
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
use crate::gui;

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

pub struct InputState {
    pub mouse_state: MouseState,
    pub key_state: KeyState,
    pub gui_state: gui::GUIState
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

    ModelerState { objects: vec![cone, sphere, cube, triangle,],}
}

fn handle_draw_operation(mut spline_state : splinedraw::SplineState,
			 edit_state : &mut edit::EditState,
			 input_state: &InputState) -> (ProgramState, splinedraw::SplineState) {
    if input_state.key_state.enter {
	if spline_state.control_points.len() >= 2 {
	    let cyllinder_object = cyllinder::create_cyllinder(0.1, 5, spline_state);

	    // cyllinders.push(cyllinder_object);
	    edit_state.cyllinder = Some(cyllinder_object);
	    return (ProgramState::Edit, splinedraw::SplineState::new());
	}
    } else {
	splinedraw::handle_spline_draw(&input_state.mouse_state, &mut spline_state);
    }

    
    (ProgramState::Draw, spline_state)
}

pub fn run_loop(mut glfw_state: GLFWState, modeler_state: ModelerState) {

    let mouse_state = MouseState { pos: glm::vec2(0.0, 0.0),
				       button1_pressed: false,
				       button1_was_pressed: false,
				       in_window: true, };
    let key_state = KeyState { enter: false, };
    let gui_state = gui::GUIState { using_peeling: false,
				    used_mouse: false };

    let mut input_state = InputState { mouse_state, key_state, gui_state };

    let mut edit_state =
	edit::EditState::new();

    let mut spline_state = splinedraw::SplineState::new();

    let mut line_objects = Vec::new();
    for i in 0..modeler_state.objects.len() {
	line_objects.push(lineobjects::create_line_object(&modeler_state.objects[i].vertices,
							  &modeler_state.objects[i].indices));
    }
    

    // let mut cyllinders : Vec<cyllinder::GeneralizedCyllinder> = Vec::new();
    
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

	
    let mut count = 0;

    let mut program_state = ProgramState::Draw;
    
    // Loop until the user closes the window
    while !glfw_state.window.should_close() {

	count += 1;
	
	let r :f32 = 1.0;
	// let phi : f32 = 0.5;
	let phi : f32 = 0.0;
	// let th :f32 = count as f32 / 50.7;
	let th : f32 = 0.0;
	// let proj = glm::ext::perspective(30.0, 4.0 / 3.0,
	// 0.1,
	// 100.0);
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
	// for i in 0..cyllinders.len() {
	if edit_state.has_cyllinder() {
	    let model_trans = glm::ext::translate(&glm::Matrix4::one(),
						  glm::vec3(0.0, 0.0,
							    -1.0 // (i as f32 - cyllinders.len() as f32
							     + 1.0) * 2.5);
	    
	    let trans = trans * model_trans; 

	    
	    cyllinder::draw_cyllinder(edit_state.cyllinder.as_ref().unwrap(),
				      &shader_program,
				      &world_line_program,
				      &trans);
	}

	edit::handle_gui_update(&mut glfw_state, &mut input_state.gui_state, &mut edit_state);
	
	// Poll for and process events
	glfw_state.glfw.poll_events();

	let flushed_events = glfw::flush_messages(&glfw_state.events);

	input_state.mouse_state.tick();
	
        for (_, event) in flushed_events {

	    glfw_state.imgui_glfw_context.handle_event(&mut glfw_state.imgui_context, &event);
	    
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    glfw_state.window.set_should_close(true)
                },
		glfw::WindowEvent::Key(Key::Enter, _, action, _) => {
		    input_state.key_state.enter =
			match action {
			    Action::Release => false,
			    Action::Press => true,
			    Action::Repeat => input_state.key_state.enter
			};
		},
		glfw::WindowEvent::CursorPos(x, y) => {
		    if !input_state.gui_state.used_mouse {
			input_state.mouse_state.pos = glm::vec2(x as f32, y as f32);
		    }
		},
		glfw::WindowEvent::CursorEnter(bb) => {
		    if !input_state.gui_state.used_mouse {
			input_state.mouse_state.in_window = bb;
		    }
		},
		glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1,
					       Action::Press, _) => {
		    if !input_state.gui_state.used_mouse {
			input_state.mouse_state.button1_pressed = true;
		    }
		},
		glfw::WindowEvent::MouseButton(glfw::MouseButton::Button1,
					       Action::Release, _) => {
		    if !input_state.gui_state.used_mouse {
			input_state.mouse_state.button1_pressed = false;
		    }
		},
                _ => {},
            }
        }

	match program_state  {
	    ProgramState::Draw => {
		let (t_program_state, t_spline_state) = handle_draw_operation(spline_state, &mut edit_state,
									      &input_state);

		program_state = t_program_state;
		spline_state = t_spline_state;
		
		screen_line_program.activate();
		spline_state.update_gpu_state();
		splinedraw::draw_spline_lines(&spline_state);
	    },
	    ProgramState::Edit => {
		edit::handle_edit_operation(&proj,
					    &input_state, &mut edit_state);
	    }
	}

	
	
        // Swap front and back buffers
        glfw_state.window.swap_buffers();

	std::thread::sleep(std::time::Duration::from_millis(17));
	
    }
}

