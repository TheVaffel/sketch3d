use crate::GLFWState;
use crate::program;
use crate::annotation;
use crate::edit;

use std::mem;

use imgui_glfw_rs::imgui::{Window,im_str,Condition};


pub struct GUIState {
    pub using_peeling: bool,
    pub used_mouse: bool,
}

impl Clone for GUIState {
    fn clone(&self) -> GUIState {
	GUIState { using_peeling: self.using_peeling,
		   used_mouse:    self.used_mouse }
    }
}

/* fn gui_base(ui: &mut UI, glfw_state: &mut GLFWState) {
    
}

fn get_ui(glfw_state: &mut GLFWState) -> UI {
    glfw_state.impui_glfw_context.frame(&mut glfw_state.window, &mut glfw_state.imgui_context);
}

pub fn handle_draw_gui(program_state: &mut program::ProgramState,
		       glfw_state: &mut GLFWState, gui_state: &mut GUIState) {
    let ui = get_ui (glfw_state);

    Window::new(imstr!("Hello"))
	.size([300.0, 300.0]
} */

pub fn run_gui(mut program_state: &mut program::ProgramState,
	       glfw_state: &mut GLFWState, gui_state: &mut GUIState) {
    let ui = glfw_state.imgui_glfw_context.frame(&mut glfw_state.window, &mut glfw_state.imgui_context);
    // ui.show_demo_window(&mut true);

    
    Window::new(im_str!("Hello"))
	.size([300.0, 300.0], Condition::FirstUseEver)
	.position([0.0, 0.0], Condition::Always)
	.build(&ui, || {
	    ui.text(im_str!("sketch3d - Main menu"));
	    ui.separator();
	    
	    let old_prog_num = program_state.to_num();
	    let mut prog_num = program_state.to_num();

	    match program_state {
		program::ProgramState::Edit(_) | program::ProgramState::Annotate(_) => {
		    ui.radio_button(im_str!("Edit"), &mut prog_num, program::PS_EDIT_NUM);
		    ui.radio_button(im_str!("Annotate"), &mut prog_num, program::PS_ANNOTATE_NUM);
		    ui.separator();
		},
		_ => {}
	    }


	    if prog_num != old_prog_num {
		
		
		let mut ps2 = program::ProgramState::Draw;
		mem::swap(&mut ps2, program_state);
		
		if prog_num == program::PS_EDIT_NUM {
		    match ps2 {
			program::ProgramState::Annotate(annotation_state) => {
			    
			    *program_state = program::ProgramState::Edit(
				edit::EditState::from_annotation_state(annotation_state)
			    );
			    /* *program_state = program::ProgramState::Edit(
				edit::EditState::from_annotation_state(*annotation_state)
			    ); */
			},
			_ => {
			    *program_state = program::ProgramState::Edit(
				edit::EditState::new()
			    );
			}
		    }
		} else if prog_num == program::PS_ANNOTATE_NUM {
		    match ps2 {
			program::ProgramState::Edit(edit_state) => {
			    *program_state = program::ProgramState::Annotate(
				annotation::AnnotationState::new(edit_state)
			    );
			    
			},
			
			_ => {
			    std::panic!("Tried to convert to annotate state without having an edit state");
			}
		    }
		}
	    }

	    match program_state {
		program::ProgramState::Edit(_) => {
		    ui.text(im_str!("Use peeling"));
		    ui.radio_button(im_str!("On"), &mut gui_state.using_peeling, true);
		    ui.radio_button(im_str!("Off"), &mut gui_state.using_peeling, false);
		},
		program::ProgramState::Annotate(_) => {
		    ui.text(im_str!("You go annotate!"));
		},
		program::ProgramState::Draw => {
		    ui.text(im_str!("Do some drawing already!"));
		}
	    }
	    
	    
	    let mouse_pos = ui.io().mouse_pos;

	    ui.separator();
	    ui.text(format!(
		"Mouse Position: ({:.1},{:.1})",
		mouse_pos[0], mouse_pos[1]
	    ));
	});

    gui_state.used_mouse = ui.io().want_capture_mouse;
    
    glfw_state.imgui_glfw_context.draw(ui, &mut glfw_state.window);
}
