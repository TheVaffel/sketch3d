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

pub fn run_gui(session: &mut program::Session,
	       mut program_state: &mut program::ProgramState,
	       glfw_state: &mut GLFWState,
	       gui_state: &mut GUIState) {
    let ui = glfw_state.imgui_glfw_context.frame(&mut glfw_state.window, &mut glfw_state.imgui_context);
    // ui.show_demo_window(&mut true);

    
    Window::new(im_str!("Menu"))
	.size([300.0, 300.0], Condition::FirstUseEver)
	.position([0.0, 0.0], Condition::Always)
	.build(&ui, || {
	    ui.text(im_str!("ParaGem - Main menu"));
	    ui.separator();
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
		mem::swap(&mut ps2, &mut program_state);
		
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
				annotation::AnnotationState::new(edit_state, session)
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
		program::ProgramState::Annotate(ref annotation_state) => {
		    ui.text(im_str!("You go annotate!"));
		    /* for annl in session.annotations.iter_mut() {
			for ann in annl.iter_mut() {
			    
			    let mut f = ann.get_size();
			    ui.input_float(im_str!("Some size annotation"), &mut f)
                            // .min(0.05).max(4.0).speed(0.1)
                                .build();
			    ann.set_size(f);
			}
		} */
                    if annotation_state.curr_cylinder_index >= 0 {
                        let mut already = -1 as i32;
                        for anni in 0..session.annotations[annotation_state.curr_cylinder_index as usize].len() {
                            let ann = &session.annotations[annotation_state.curr_cylinder_index as usize][anni];
                            let i = ann.as_ref().get_render_index();
                            if annotation_state.curr_render_index == i as i32 {
                                already = anni as i32;
                            }
                        }

                        if already >= 0 {
                            let mut f = session.annotations[annotation_state.curr_cylinder_index as usize][already as usize].as_ref().get_size();

                        ui.drag_float(im_str!("Some size annotation"), &mut f)
                            .min(0.05).max(2.0).speed(0.03)
                            .build();
                        
                        session.annotations[annotation_state.curr_cylinder_index as usize][already as usize].as_mut().set_size(f);
                        } else {
                            if ui.button(im_str!("Create size annotation"), [200.0, 30.0]) {
                                let ann = annotation::SizeAnnotation { size: 1.0,
                                                                       position: glm::vec3 (0.0, 0.0, 0.0),
                                                                       index: annotation_state.curr_render_index as usize };
                                session.annotations[annotation_state.curr_cylinder_index as usize].push(Box::<annotation::SizeAnnotation>::from(ann));
                            }
                        }
                    }
                    
                    /* if annotation_state.curr_cylinder_index >= 0 {
                        let mut f = session.annotations[annotation_state.curr_cylinder_index as usize][annotation_state.curr_render_index as usize].as_ref().get_size();

                        ui.drag_float(im_str!("Some size annotation"), &mut f)
                            .min(0.05).max(2.0).speed(0.03)
                            .build();
                        
                        session.annotations[annotation_state.curr_cylinder_index as usize][annotation_state.curr_render_index as usize].as_mut().set_size(f);
                    } */

		},
		program::ProgramState::Draw => {
		    ui.text(im_str!("Do some drawing already!"));
		}
	    }
	    
	});

    gui_state.used_mouse = ui.io().want_capture_mouse;
    
    glfw_state.imgui_glfw_context.draw(ui, &mut glfw_state.window);
}

