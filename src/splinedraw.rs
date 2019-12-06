extern crate glm;
extern crate gl;

use crate::program;
use crate::edit;
use crate::utils;

use glm::builtin::*;

pub static LINE_LIMIT: f32 = 1.0 / 20.0; // 5% of window width
static MAX_NUM_POINTS: usize = 400;

pub static SPLINE_RESOLUTION: usize = 5; // Points per control point
static SPLINE_DEGREE:     usize = 3;



fn lookup_pad<T: Copy>(vec : &Vec<T>, n : usize) -> T {
    if n >= vec.len() {
	vec[vec.len() - 1]
    } else {
	vec[n]
    }
}


pub struct SplineCoefficients {
    coefficients : Vec<Vec<f32> >
}

pub fn make_spline_coefficients() -> SplineCoefficients {
    let mut coeffs = SplineCoefficients { coefficients: Vec::with_capacity(SPLINE_RESOLUTION) };

    let k = SPLINE_DEGREE;
    let m = SPLINE_RESOLUTION;
    
    let mut bb = vec![vec![0.0; 2 * k + 1]; m];
    for i in 0..m {
	bb[i][k] = 1.0;

	let x = i as f32 / m as f32;
	
	for j in (1..(k + 1)).rev() {
	    for a in 0..(j + k) {
		let pad = k as f32 - a as f32;
		bb[i][a] =
		    bb[i][a] * ((x + pad) / k as f32) +
		    bb[i][a + 1] * (((1 + a) as f32 - x) / k as f32);
	    }
	}

	coeffs.coefficients.push(Vec::with_capacity(k+1));
	
	for j in 0..(k+1) {
	    coeffs.coefficients[i].push(bb[i][j]);
	}
    }

    coeffs
}

lazy_static! {
    pub static ref SPLINE_COEFFICIENTS : SplineCoefficients = make_spline_coefficients();
}



pub struct SplineState {
    pub control_points    : Vec<glm::Vec3>,
    pub point_colors      : Vec<glm::Vec4>, // Colors of control points
    pub spline_points     : Vec<glm::Vec3>,
    spline_lines_vao       : gl::types::GLuint,
    spline_lines_vbo       : gl::types::GLuint,
    control_points_vao     : gl::types::GLuint,
    control_points_vbo     : gl::types::GLuint,
    point_color_vbo        : gl::types::GLuint,
    spline_color_vbo       : gl::types::GLuint
}


impl Drop for SplineState {
    fn drop(&mut self) {
	unsafe {
	    gl::DeleteBuffers(1, &self.spline_lines_vbo);
	    gl::DeleteVertexArrays(1, &self.spline_lines_vao);
	    gl::DeleteBuffers(1, &self.point_color_vbo);
	    gl::DeleteBuffers(1, &self.spline_color_vbo);
	    gl::DeleteBuffers(1, &self.control_points_vbo);
	    gl::DeleteVertexArrays(1, &self.control_points_vao);
	}
    }
}

impl SplineState {
    

    fn add_control_point(self : &mut SplineState,
			 vec : glm::Vec3) {
	self.control_points.push(vec);
	self.point_colors.push(glm::vec4(0.0, 0.0, 0.0, 1.0));
    }

    
    pub fn new() -> SplineState {
	let mut spline_state = SplineState {control_points: Vec::new(),
					    point_colors: Vec::new(),
					    spline_points: Vec::new(),
					    spline_lines_vao: 0, spline_lines_vbo: 0,
					    control_points_vao: 0, control_points_vbo: 0,
					    point_color_vbo: 0, spline_color_vbo: 0};

	unsafe {
	    // Spline lines
	    gl::GenBuffers(1, &mut spline_state.spline_lines_vbo);
	    gl::GenVertexArrays(1, &mut spline_state.spline_lines_vao);

	    gl::BindBuffer(gl::ARRAY_BUFFER, spline_state.spline_lines_vbo);
	    gl::BindVertexArray(spline_state.spline_lines_vao);
	    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

	    
	    gl::EnableVertexAttribArray(0);
	    gl::VertexAttribPointer(
		0, 3, gl::FLOAT,
		gl::FALSE, (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
		std::ptr::null());

	    
	    gl::EnableVertexAttribArray(1);
	    gl::VertexAttribPointer(
		1, 4, gl::FLOAT,
		gl::FALSE, (4 * std::mem::size_of::<f32>()) as gl::types::GLint,
		std::ptr::null());

	    // Control points
	    gl::GenBuffers(1, &mut spline_state.control_points_vbo);
	    gl::GenBuffers(1, &mut spline_state.point_color_vbo);
	    gl::GenVertexArrays(1, &mut spline_state.control_points_vao);

	    gl::BindBuffer(gl::ARRAY_BUFFER, spline_state.control_points_vbo);
	    gl::BindVertexArray(spline_state.control_points_vao);
	    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

	    gl::EnableVertexAttribArray(0);
	    gl::VertexAttribPointer(
		0, 3, gl::FLOAT,
		gl::FALSE, (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
		std::ptr::null());

	    gl::EnableVertexAttribArray(1);
	    gl::BindBuffer(gl::ARRAY_BUFFER, spline_state.point_color_vbo);
	    gl::VertexAttribPointer(
		1, 4, gl::FLOAT,
		gl::FALSE, (4 * std::mem::size_of::<f32>()) as gl::types::GLint,
		std::ptr::null());
	}

	spline_state
    }

    pub fn update_gpu_state(self: &mut SplineState) {

	if self.control_points.len() >= 2 {
	    /* for _i in 0..SPLINE_DEGREE {
		// self.control_points.push(self.control_points[self.control_points.len() - 1]);
		self.add_control_point(self.control_points[self.control_points.len() - 1]);
	    } */
	    
	    self.spline_points.clear();
	    // let mut spline_points : Vec<glm::Vec3> = Vec::new();

	    let mut spline_colors : Vec<glm::Vec4> = Vec::new();
	    
	    self.spline_points.push(self.control_points[0]);
	    spline_colors.push(self.point_colors[0]);

	    // Build from bottom up
	    let num_cp = self.control_points.len() - 1;
	    let num_points = num_cp * SPLINE_RESOLUTION;

	    for i in 0..num_points {
		let mut pp = glm::vec3(0.0, 0.0, 0.0);
		let mut cp = glm::vec4(0.0, 0.0, 0.0, 1.0);
		let curr_point =  (i + 1) / SPLINE_RESOLUTION; 

		let b_ind = (i + 1) % SPLINE_RESOLUTION;
		
		for j in 0..(SPLINE_DEGREE+1) {
		    pp = pp + lookup_pad(&self.control_points, curr_point + j) *
			SPLINE_COEFFICIENTS.coefficients[b_ind][j];
		    cp = cp + lookup_pad(&self.point_colors, curr_point + j) *
			SPLINE_COEFFICIENTS.coefficients[b_ind][j];
		}

		cp[3] = 1.0;

		self.spline_points.push(pp);
		spline_colors.push(cp);

		if cp[0] != 0.0 || cp[1] != 0.0 || cp[2] != 0.0 {
		    println!("{:?}", cp);
		}
	    }

	    /* for _i in 0..SPLINE_DEGREE {
		self.control_points.pop();
		self.point_colors.pop();
	    } */
	    
	    unsafe {
		gl::BindBuffer(gl::ARRAY_BUFFER, self.spline_lines_vbo);
		gl::BufferData(
		    gl::ARRAY_BUFFER,
		    (self.spline_points.len() * std::mem::size_of::<glm::Vec3>()) as gl::types::GLsizeiptr,
		    self.spline_points.as_ptr() as *const gl::types::GLvoid,
		    gl::STREAM_DRAW);

		gl::BindBuffer(gl::ARRAY_BUFFER, self.spline_color_vbo);
		gl::BufferData(
		    gl::ARRAY_BUFFER,
		    (spline_colors.len() * std::mem::size_of::<glm::Vec4>()) as gl::types::GLsizeiptr,
		    spline_colors.as_ptr() as *const gl::types::GLvoid,
		    gl::STREAM_DRAW);
	    }

	}
	
	unsafe {
	    gl::BindBuffer(gl::ARRAY_BUFFER, self.control_points_vbo);
	    gl::BufferData(
		gl::ARRAY_BUFFER,
		(self.control_points.len() * std::mem::size_of::<glm::Vec3>()) as gl::types::GLsizeiptr,
		self.control_points.as_ptr() as *const gl::types::GLvoid,
		gl::STREAM_DRAW);

	    gl::BindBuffer(gl::ARRAY_BUFFER, self.point_color_vbo);
	    gl::BufferData(
		gl::ARRAY_BUFFER,
		(self.point_colors.len() * std::mem::size_of::<glm::Vec4>()) as gl::types::GLsizeiptr,
		self.point_colors.as_ptr() as *const gl::types::GLvoid,
		gl::STREAM_DRAW);
	}
    }
}


pub fn spline_screen_to_world_transform(spline: &mut SplineState) {
    for i in 0..spline.control_points.len() {
	spline.control_points[i][1] = - spline.control_points[i][1];
    }

    spline.update_gpu_state();
}

pub fn draw_spline_lines(spline_state: &SplineState ) {
    unsafe {
	gl::BindVertexArray(spline_state.spline_lines_vao);
	gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
	gl::EnableVertexAttribArray(0);
	gl::EnableVertexAttribArray(1);
	gl::LineWidth(2.0);

	gl::DrawArrays(gl::LINE_STRIP, 0, spline_state.spline_points.len() as i32);
    }
}

pub fn draw_control_points(spline_state: &SplineState ) {
    unsafe {
	gl::BindVertexArray(spline_state.control_points_vao);
	gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
	gl::EnableVertexAttribArray(0);
	gl::EnableVertexAttribArray(1);
	gl::PointSize(6.0);

	gl::DrawArrays(gl::POINTS, 0, spline_state.control_points.len() as i32);
    }
}

pub fn handle_spline_draw(mouse_state: &program::MouseState, spline_state: & mut SplineState) {
    
    if mouse_state.button1_pressed && mouse_state.in_window {
	let clone = mouse_state.pos.clone();
	let new_point = utils::normalize_point(clone);
	let new_point  = glm::vec3(new_point.x, new_point.y, 0.0);
	
	if spline_state.control_points.len() == 0 {

	    spline_state.add_control_point(new_point);
	} else if length(new_point - spline_state.control_points[spline_state.control_points.len() - 1] )
	    >= LINE_LIMIT * 0.9 &&
	    spline_state.control_points.len() < MAX_NUM_POINTS {
		// Enforce edge length to be LINE_LIMIT:

		let vv = new_point - spline_state.control_points[spline_state.control_points.len() - 1];
		let v2 = vv / length(vv) * LINE_LIMIT;
		let new_point = spline_state.control_points[spline_state.control_points.len() - 1] + v2;
		
		
		spline_state.add_control_point(new_point);
	    }
	
    }
}
