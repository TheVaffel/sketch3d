extern crate imgui_glfw_rs;
extern crate gl;
extern crate glm;
extern crate num_traits;

use imgui_glfw_rs::glfw::{self,Context,Key,Action};
use crate::{ModelerState, GLFWState};
use std::ffi::{CString};
use std::f32;
use std::mem;
use std::cell::Cell;

use crate::settings;
use crate::shaders;
use crate::objects;
use crate::lineobjects;
use crate::splinedraw;
use crate::cylinder;
use crate::utils;
use crate::edit;
use crate::gui;
use crate::annotation;

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
    Edit(edit::EditState),
    Annotate(annotation::AnnotationState),
}

pub struct Session {
    pub cylinders: Vec<cylinder::GeneralizedCylinder>,
    pub annotations: Vec<Vec<Box<dyn annotation::Annotation>>>, // One vector per cylinder
}

pub static PS_DRAW_NUM : usize = 0;
pub static PS_EDIT_NUM : usize = 1;
pub static PS_ANNOTATE_NUM : usize = 2;

impl ProgramState {
    pub fn to_num(&self) -> usize {
	match self {
	    ProgramState::Draw => PS_DRAW_NUM,
	    ProgramState::Edit(_) => PS_EDIT_NUM,
	    ProgramState::Annotate(_) => PS_ANNOTATE_NUM,
	}
    }
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


pub fn handle_gui_update(session: &mut Session,
			 program_state: &mut ProgramState,
			 mut glfw_state: &mut crate::GLFWState,
			 mut gui_state: &mut gui::GUIState) {
    let old_gui_state = gui_state.clone();
    let old_program_num = program_state.to_num();
    gui::run_gui(session, program_state, &mut glfw_state, &mut gui_state);

    match program_state {
	ProgramState::Edit(ref mut edit_state) => {
	    if gui_state.using_peeling != old_gui_state.using_peeling ||
		PS_EDIT_NUM != old_program_num {
		    edit_state.clear_selected(&mut session.cylinders);
		}
	},
	_ => {}
    }
}


fn handle_draw_operation(mut spline_state : &mut splinedraw::SplineState,
			 input_state: &InputState,
			 session: &Session) -> Option<cylinder::GeneralizedCylinder> {
    if input_state.key_state.enter {
	if spline_state.control_points.len() >= 2 {
	    let mut tmp_spline = splinedraw::SplineState::new();
	    mem::swap(&mut tmp_spline, spline_state);
	    let cylinder_object = cylinder::create_cylinder(0.1, 5, tmp_spline);
	    // *spline_state = splinedraw::SplineState::new();

	    return Some(cylinder_object);
	}
    } else {
	splinedraw::handle_spline_draw(&input_state.mouse_state, &mut spline_state);
    }

    None
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

    let mut spline_state = splinedraw::SplineState::new();

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

	
    let mut _count = 0;

    let mut program_state = ProgramState::Draw;

    let mut session = Session { cylinders: Vec::new(),
				annotations: Vec::new(), };
        
    // Loop until the user closes the window
    while !glfw_state.window.should_close() {

	_count += 1;
	
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
	match program_state {
	    ProgramState::Edit(ref edit_state) => {
		cylinder::draw_cylinder(&session.cylinders[edit_state.curr_cylinder],
					&shader_program,
					&world_line_program,
					&trans);
		
	    },
	    ProgramState::Annotate(ref annotation_state) => {

		for c in &session.cylinders {
		    cylinder::draw_cylinder(&c,
					    &shader_program,
					    &world_line_program,
					    &trans);
		}
		
		annotation::draw_annotations(&session,
					     &annotation_state,
					     &world_line_program,
					     &trans);
	    },
	    _ => {}
	}

	// Handle GUI

	handle_gui_update(&mut session,
			  &mut program_state,
			  &mut glfw_state,
			  &mut input_state.gui_state);
	
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
		let o_cylinder = handle_draw_operation(&mut spline_state,
						       &input_state,
						       &session);
		match o_cylinder {
		    Some(cylinder) => {
			let mut edit = edit::EditState::new();
			
			annotation::push_default_annotations(session.cylinders.len(),
							     cylinder.spline.control_points.len(),
							     &mut session.annotations);
			session.cylinders.push(cylinder);
			edit.curr_cylinder = 0;
			program_state = ProgramState::Edit(edit);

		    },
		    _ => { }
		}
		
		screen_line_program.activate();
		spline_state.update_gpu_state();
		splinedraw::draw_spline_lines(&spline_state);
	    },
	    ProgramState::Edit(ref mut edit_state) => {
		edit::handle_edit_operation(&proj,
					    &input_state,
					    edit_state,
					    &mut session);
	    },
	    ProgramState::Annotate(ref mut annotation_state) => {
		annotation::handle_annotation(&proj,
					      &input_state, annotation_state,
					      &mut session);

		annotation::update_gpu_annotation_state(&annotation_state,
							&mut session);
	    },
	}

	
	
        // Swap front and back buffers
        glfw_state.window.swap_buffers();

	std::thread::sleep(std::time::Duration::from_millis(17));
	
    }
}

