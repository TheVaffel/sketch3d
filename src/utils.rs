
extern crate glm;


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
