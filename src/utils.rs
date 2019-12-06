
extern crate glm;
extern crate nalgebra as na;

use crate::settings;

pub fn ortho(left : f32, right : f32, bottom : f32, top : f32, back : f32, front : f32) -> glm::Mat4 {
    let dist_hor  = right - left;
    let dist_ver  = top - bottom;
    let dist_view = front - back;

    // Attempt colmajor
    glm::mat4(2.0 / dist_hor, 0.0,             0.0,             0.0,
	      0.0,            -2.0 / dist_ver, 0.0,             0.0,
	      0.0,            0.0,             2.0 / dist_view, 0.0,
	      // 0.0,            0.0,             0.0,             1.0)
	      - (left + right) / dist_hor,          - (bottom + top) / dist_ver,        - (front + back) / dist_view,           1.0)
}



#[allow(dead_code)]
pub fn print_matrix<T: na::Dim,
		    U: na::Dim,
		    S: na::storage::Storage<f32, T, U>>
    (mat : na::Matrix<f32, T, U, S>) {
	for i in 0..mat.nrows() {
	    for j in 0..mat.ncols() {
		print!("{:?}\t", mat[(i,j)]);
	    }
	    print!("\n");
	}
	
    }

pub fn length_mouse_pos_to_point(mouse_pos: glm::Vec2,
				 point: glm::Vec3,
				 projection_matrix: &glm::Mat4) -> f32 {
    let l = *projection_matrix * glm::vec4(point.x, point.y, point.z, 1.0);
    let p = glm::vec2(l.x / l.w, l.y / l.w);
    let nm = normalize_point(mouse_pos);
    glm::builtin::length(p - nm)
}

pub fn select_point(mouse_pos : glm::Vec2, points : &Vec<glm::Vec3>,
		    projection_matrix: &glm::Mat4, sensitivity : f32) -> i32 {

    let mut mindi = -1;
    let mut mind = sensitivity;

    let mut ddb = 1e9;
    
    for i in 0..points.len() {
	let ll = length_mouse_pos_to_point(mouse_pos,
					   points[i],
					   projection_matrix);
	if ll < mind {
	    mind = ll;
	    mindi = i as i32;
	}

	ddb = glm::min(ddb, ll);
    }

    // Return -1 if none is within sensitivity
    return mindi; 
}

pub fn normalize_point(p : glm::Vec2) -> glm::Vec2 {
    p / (glm::vec2(settings::WINDOW_WIDTH as f32,
		   - (settings::WINDOW_HEIGHT as f32))
	 / 2.0)
	+ glm::vec2(-1.0, 1.0)
}
