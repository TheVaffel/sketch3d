
extern crate glm;


pub fn select_point(mouse_pos : glm::Vec2, points : &Vec<glm::Vec3>,
		    projection_matrix: &glm::Mat4, sensitivity : f32) -> i32 {

    let mut mindi = -1;
    let mut mind = sensitivity;
    
    for i in 0..points.len() {
	let lp = glm::vec4(points[i].x, points[i].y, points[i].z, 1.0);
	let trr = * projection_matrix * lp;
	let pp = glm::vec2(trr.x / trr.w, trr.y / trr.w);


	let ll = glm::builtin::length(pp - mouse_pos);
	if ll < mind {
	    mind = ll;
	    mindi = i as i32;
	}
    }

    // Return -1 if none is within sensitivity
    return mindi; 
}
