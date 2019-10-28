
extern crate glm;

use crate::settings;

pub struct EditState {
    pub selected_indices: Vec<usize>,
    pub ref_point: glm::Vec2,
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
			 proj : &glm::Mat4, mouse_state : &program::MouseState,
			 key_state : &program::KeyState, edit_state : &mut EditState) {
    match edit_state.state {
        EditEnum::Selecting => {
            if mouse_state.button1_pressed {
                let selected_point_ind = select_point(mouse_state.pos,
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
            
            if !mouse_state.button1_pressed &&
                mouse_state.button1_was_pressed &&
                edit_state.selected_indices.len() > 0 {
                    edit_state.state = EditEnum::Dragging;
                }
        },
        EditEnum::Dragging => {
            if mouse_state.button1_pressed {
                if !mouse_state.button1_was_pressed {
                    let selected_point_ind = select_point(mouse_state.pos,
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
                        edit_state.ref_point = normalize_point(mouse_state.pos);
                    }
                } else {
                    let new_mpoint = normalize_point(mouse_state.pos);
	            let diff = new_mpoint - edit_state.ref_point;
	            for i in &edit_state.selected_indices {
		        cyllinder.spline.control_points[*i] =
		            cyllinder.spline.control_points[*i] + glm::vec3(diff.x, -diff.y, 0.0);
	            }
	            edit_state.ref_point = new_mpoint;
                }
                
            }
        }
    }

    if key_state.enter {
        cyllinder.update_mesh();
    }

    cyllinder.spline.update_gpu_state();
}
