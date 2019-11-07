
extern crate glm;

use crate::settings;
use crate::program;
use crate::cyllinder;
use crate::laplacian;

static SELECTION_SENSITIVITY : f32 = 0.03;

pub enum EditEnum {
    Selecting,
    Dragging
}

pub struct EditState {
    pub selected_indices: Vec<usize>,
    pub ref_point: glm::Vec2,
    pub laplacian_system: laplacian::LaplacianEditingSystem,
    pub state : EditEnum,
}

pub fn normalize_point(p : glm::Vec2) -> glm::Vec2 {
    p / (glm::vec2(settings::WINDOW_WIDTH as f32,
		   - (settings::WINDOW_HEIGHT as f32))
	 / 2.0)
	+ glm::vec2(-1.0, 1.0)
}
    
pub fn select_point(mouse_pos : glm::Vec2, points : &Vec<glm::Vec3>,
		    projection_matrix: &glm::Mat4, sensitivity : f32) -> i32 {

    let mut mindi = -1;
    let mut mind = sensitivity;

    let mut ddb = 1e9;
    
    for i in 0..points.len() {
	let lp = glm::vec4(points[i].x, points[i].y, points[i].z, 1.0);
	let trr = * projection_matrix * lp;
	let pp = glm::vec2(trr.x / trr.w, trr.y / trr.w);

	let normalized_mouse_pos = normalize_point(mouse_pos);

	let ll = glm::builtin::length(pp - normalized_mouse_pos);
	if ll < mind {
	    mind = ll;
	    mindi = i as i32;
	}

	ddb = glm::min(ddb, ll);
    }

    // Return -1 if none is within sensitivity
    return mindi; 
}



pub fn handle_edit_operation(cyllinder : &mut cyllinder::GeneralizedCyllinder,
			     proj : &glm::Mat4, input_state: &program::InputState,
			     edit_state : &mut EditState) {
    match edit_state.state {
        EditEnum::Selecting => {
            if input_state.mouse_state.button1_pressed {
                let selected_point_ind = select_point(input_state.mouse_state.pos,
					              &cyllinder.spline.control_points,
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
                    cyllinder.spline.point_colors[selected_point_ind as usize] = glm::vec4(1.0, 0.0, 0.0, 1.0);
                    edit_state.selected_indices.push(selected_point_ind as usize);
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
                    let selected_point_ind = select_point(input_state.mouse_state.pos,
					                  &cyllinder.spline.control_points,
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
                        edit_state.state = EditEnum::Selecting;
                        for i in &edit_state.selected_indices {
		            cyllinder.spline.point_colors[*i] = glm::vec4(0.0, 0.0, 0.0, 1.0);
	                }
	                edit_state.selected_indices.clear();
                    } else {
                        edit_state.ref_point = normalize_point(input_state.mouse_state.pos);

			

			let mut fixed_vec : Vec<usize> = Vec::new();
			
			let mut arr : Vec<i32> = (0..cyllinder.spline.control_points.len() as i32).collect();
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
			
			edit_state.laplacian_system = laplacian::setup_system(&cyllinder.spline.control_points,
									      fixed_vec);
                    }
                } else {
                    let new_mpoint = normalize_point(input_state.mouse_state.pos);
	            for i in &edit_state.selected_indices {
		        cyllinder.spline.control_points[*i] =
			    glm::vec3(new_mpoint.x, -new_mpoint.y, 0.0);
	            }

		    
		    edit_state.laplacian_system.solve(&mut cyllinder.spline.control_points);
                }
                
            }
        }
    }

    cyllinder.update_mesh();

    cyllinder.spline.update_gpu_state();
}
