
extern crate glm;

use crate::program;
use crate::cylinder;
use crate::laplacian;
use crate::splinedraw;
use crate::annotation;
use crate::utils;

pub static SELECTION_SENSITIVITY : f32 = 0.03;

pub enum EditEnum {
    Selecting,
    Dragging
}

pub struct EditState {
    pub selected_indices: Vec<usize>,
    pub ref_point: glm::Vec2,
    pub laplacian_system: laplacian::LaplacianEditingSystem,
    pub state : EditEnum,
    pub curr_cylinder: usize,
}


impl EditState {
    pub fn new() -> EditState {
	EditState { selected_indices : Vec::new(),
		    ref_point : glm::vec2(0.0, 0.0),
                    state : EditEnum::Selecting,
		    laplacian_system : laplacian::LaplacianEditingSystem::empty(),
		    curr_cylinder: usize::max_value() }
    }

    pub fn from_annotation_state(annotation_state : annotation::AnnotationState) -> EditState {
	let mut ee = EditState::new();
	// ee.cylinders = annotation_state.cylinders;
	ee.curr_cylinder = 0;
	ee
    }

    pub fn add_selected_point(&mut self,
			      session: &mut program::Session,
			      ind: usize) {
	let cylinder = &mut session.cylinders[self.curr_cylinder];
	cylinder.
	    spline.point_colors[ind] = glm::vec4(1.0, 0.0, 0.0, 1.0);
        self.selected_indices.push(ind);
    }

    pub fn clear_selected(&mut self,
			  // session: &mut program::Session
			  cylinders: &mut Vec<cylinder::GeneralizedCylinder>) {
	
	let cylinder = &mut cylinders[self.curr_cylinder];
	for i in &self.selected_indices {
	    cylinder.spline.point_colors[*i] = glm::vec4(0.0, 0.0, 0.0, 1.0);
	}
	self.selected_indices.clear();
	
    }
}

pub fn handle_edit_no_peeling(proj : &glm::Mat4, input_state: &program::InputState,
			      edit_state : &mut EditState,
			      session : &mut program::Session) {
    let cylinder = &mut session.cylinders[edit_state.curr_cylinder];
    match edit_state.state {
        EditEnum::Selecting => {
            if input_state.mouse_state.button1_pressed {
                let selected_point_ind = utils::select_point(input_state.mouse_state.pos,
							     &cylinder.spline.control_points,
							     proj, SELECTION_SENSITIVITY);
                
                let mut already_chosen = false;
                if selected_point_ind >= 0 {
	            for i in &edit_state.selected_indices {
	                if *i == selected_point_ind as usize {
		            already_chosen = true;
		            break;
	                }
	            }
                }

                
                if !already_chosen && selected_point_ind >= 0 {
		    edit_state.add_selected_point(session, selected_point_ind as usize);
                }
            }
            
            if !input_state.mouse_state.button1_pressed &&
                input_state.mouse_state.button1_was_pressed &&
                edit_state.selected_indices.len() > 0 {
                    edit_state.state = EditEnum::Dragging;
                }
        },
        EditEnum::Dragging => {

            if input_state.mouse_state.button1_pressed {
                if !input_state.mouse_state.button1_was_pressed {
                    let selected_point_ind = utils::select_point(input_state.mouse_state.pos,
								 &cylinder.spline.control_points,
								 proj, SELECTION_SENSITIVITY);

                    let mut already_chosen = false;
                    if selected_point_ind >= 0 {
	                for i in &edit_state.selected_indices {
	                    if *i == selected_point_ind as usize {
		                already_chosen = true;
		                break;
	                    }
	                }
                    }
                    
                    if selected_point_ind < 0 || !already_chosen {
			edit_state.clear_selected(&mut session.cylinders);
                        edit_state.state = EditEnum::Selecting;
                    } else {
                        edit_state.ref_point = utils::normalize_point(input_state.mouse_state.pos);

			

			let mut fixed_vec : Vec<usize> = Vec::new();
			
			let mut arr : Vec<i32> = (0..cylinder.spline.control_points.len() as i32).collect();
			for i in &edit_state.selected_indices {
			    arr[*i] = -1;
			}
			for i in arr {
			    if i != -1 {
				fixed_vec.push(i as usize);
			    }
			}

			// Fix the position of the currently moving node
			fixed_vec.push(selected_point_ind as usize);
			
			edit_state.laplacian_system = laplacian::setup_system(&cylinder.spline.control_points,
									      fixed_vec);
                    }
                } else {
                    let new_mpoint = utils::normalize_point(input_state.mouse_state.pos);
	            for i in &edit_state.selected_indices {
		        cylinder.spline.control_points[*i] =
			    glm::vec3(new_mpoint.x, -new_mpoint.y, 0.0);
	            }

		    
		    edit_state.laplacian_system.solve(&mut cylinder.spline.control_points);
                }
                
            }
        }
    }
}

pub fn handle_edit_with_peeling(proj : &glm::Mat4, input_state: &program::InputState,
				edit_state : &mut EditState,
				session: &mut program::Session) {
    // let cylinder = &mut edit_state.cylinder.as_mut().unwrap();
    
    let cylinder = &mut session.cylinders[edit_state.curr_cylinder];
    match edit_state.state {
	EditEnum::Selecting => {
	    if input_state.mouse_state.button1_pressed {
		
                let selected_point_ind = utils::select_point(input_state.mouse_state.pos,
							     &cylinder.spline.control_points,
							     proj, SELECTION_SENSITIVITY);
		if selected_point_ind >= 0 {
		    
		    edit_state.laplacian_system = laplacian::setup_original_points(&cylinder.spline.control_points);
			
		    edit_state.ref_point = utils::normalize_point(input_state.mouse_state.pos);

		    // Convention: First selected index is always the dragged index
		    edit_state.add_selected_point(session, selected_point_ind as usize);
		    edit_state.state = EditEnum::Dragging;
		}
	    }
	},
	EditEnum::Dragging => {
	    if !input_state.mouse_state.button1_pressed {
		edit_state.state = EditEnum::Selecting;
		edit_state.clear_selected(&mut session.cylinders);
	    } else {
		let new_point = utils::normalize_point(input_state.mouse_state.pos);

		let area_of_effect = (glm::length(new_point - edit_state.ref_point) / splinedraw::LINE_LIMIT) as i32;

		let mut fixed_points : Vec<usize> = Vec::new();

		let len = cylinder.spline.control_points.len();
		
		let s1 = edit_state.selected_indices[0];
		
		cylinder.spline.control_points[s1 as usize] =
		    glm::vec3(new_point.x, -new_point.y, 0.0);

		edit_state.clear_selected(&mut session.cylinders);
		
		// Preserve dragged point as first in selected-point-list
		edit_state.add_selected_point(session, s1);
		
		for i in 0..len {
		    if (i as i32 - s1 as i32).abs() > area_of_effect {
			fixed_points.push(i as usize);
		    } else if i != s1 {
			edit_state.add_selected_point(session, i as usize);
		    }
		}

		fixed_points.push(s1 as usize);

		// Must redeclare to release mutable borrow for above section
		let cylinder = &mut session.cylinders[edit_state.curr_cylinder];
		
		edit_state.laplacian_system.setup_fixed_points(fixed_points);

		edit_state.laplacian_system.solve(&mut cylinder.spline.control_points);
	    }
	}
    }
}

pub fn handle_edit_operation(proj : &glm::Mat4, input_state: &program::InputState,
			     mut edit_state : &mut EditState,
			     session: &mut program::Session) {

    if input_state.gui_state.using_peeling {
	handle_edit_with_peeling(&proj, &input_state,
				 &mut edit_state,
				 session);
    } else {
	handle_edit_no_peeling(&proj, &input_state,
			       &mut edit_state,
			       session);
    }

    
    let cylinder = &mut session.cylinders[edit_state.curr_cylinder];
    cylinder.update_mesh(&session.annotations[edit_state.curr_cylinder]);

    cylinder.spline.update_gpu_state();

}
