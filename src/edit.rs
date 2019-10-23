
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
