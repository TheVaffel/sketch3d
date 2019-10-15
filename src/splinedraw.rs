extern crate glfw;
extern crate glm;
extern crate gl;

use crate::settings;
use crate::program;

use glm::builtin::*;

static LINE_LIMIT: f32 = 1.0 / 20.0; // 5% of window width
static MAX_NUM_POINTS: usize = 400;

static SPLINE_RESOLUTION: usize = 2; // Points per control point
static SPLINE_DEGREE:     usize = 3;


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

    println!("Coeffs = {:?}", coeffs.coefficients);
    
    coeffs
}


pub struct SplineState {
    control_points    : Vec<glm::Vec2>,
    pub spline_points     : Vec<glm::Vec2>,
    vao       : gl::types::GLuint,
    vbo       : gl::types::GLuint,
}


impl Drop for SplineState {
    fn drop(&mut self) {
	unsafe {
	    gl::DeleteBuffers(1, &self.vbo);
	    gl::DeleteVertexArrays(1, &self.vao);
	}
    }
}

impl SplineState {
    pub fn new() -> SplineState {
	let mut spline_state = SplineState {control_points: Vec::new(),
					    spline_points: Vec::new(),
					    vao: 0, vbo: 0 };

	unsafe {
	    gl::GenBuffers(1, &mut spline_state.vbo);
	    gl::GenVertexArrays(1, &mut spline_state.vao);

	    gl::BindBuffer(gl::ARRAY_BUFFER, spline_state.vbo);
	    gl::BindVertexArray(spline_state.vao);
	    gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);

	    gl::VertexAttribPointer(
		0, 2, gl::FLOAT,
		gl::FALSE, (2 * std::mem::size_of::<f32>()) as gl::types::GLint,
		std::ptr::null());
	    gl::EnableVertexAttribArray(0);
	}

	spline_state
    }

    pub fn update_gpu_state(self: &mut SplineState, coeffs: &SplineCoefficients) {

	if (self.spline_points.len() as i32 - 1) / (SPLINE_RESOLUTION as i32) <
	    self.control_points.len() as i32 - SPLINE_DEGREE as i32 &&
	    self.control_points.len() >= SPLINE_DEGREE + 2 {
	
		if self.spline_points.len() == 0 {
		    self.spline_points.push(self.control_points[0]);
		}

		// let dt = 1.0 / SPLINE_RESOLUTION as f32;
		
		// Build from start_cp + dt, start_cp + 2 * dt ... start_cp + 1
		// NB: Assumes start_cp is multiple of 
		let start_cp = self.spline_points.len() / SPLINE_RESOLUTION;

		let num_cp = self.control_points.len() - SPLINE_DEGREE - start_cp - 1;
		let num_points = num_cp * SPLINE_RESOLUTION;

		for i in 0..num_points {
		    let mut pp = glm::vec2(0.0, 0.0);
		    let curr_point = start_cp + (i + 1) / SPLINE_RESOLUTION; 

		    let b_ind = (i + 1) % SPLINE_RESOLUTION;
		    
		    for j in 0..(SPLINE_DEGREE+1) {
			pp = pp + self.control_points[curr_point + j] *
			    coeffs.coefficients[b_ind][j];
		    }

		    self.spline_points.push(pp);
		}
		
		unsafe {
		    gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
		    gl::BufferData(
			gl::ARRAY_BUFFER,
			(self.spline_points.len() * std::mem::size_of::<glm::Vec2>()) as gl::types::GLsizeiptr,
			self.spline_points.as_ptr() as *const gl::types::GLvoid,
			gl::STREAM_DRAW);
		}
	    }
    }
}

pub fn draw_spline(spline_state: &SplineState ) {
    unsafe {
	gl::BindVertexArray(spline_state.vao);
	gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
	gl::EnableVertexAttribArray(0);
	gl::LineWidth(2.0);

	gl::DrawArrays(gl::LINE_STRIP, 0, spline_state.spline_points.len() as i32);
    }
    
}


pub fn handle_spline_draw(mouse_state: &program::MouseState, spline_state: & mut SplineState) {
    if mouse_state.button1_pressed && mouse_state.in_window {
	let new_point = mouse_state.pos.clone() /
	    (glm::vec2(settings::WINDOW_WIDTH as f32,
		       - (settings::WINDOW_HEIGHT as f32)) / 2.0)
	    + glm::vec2(- 1.0, 1.0);
	
	if spline_state.control_points.len() == 0 {
	    
		if spline_state.control_points.len() == 0 {
		    for _ in 0..(SPLINE_DEGREE+1) {
			spline_state.control_points.push(new_point);
		    }
		}
	    
	} else if length(new_point - spline_state.control_points[spline_state.control_points.len() - 1] )
	    >= LINE_LIMIT * 0.9 &&
	    spline_state.control_points.len() < MAX_NUM_POINTS {
		// Enforce edge length to be LINE_LIMIT:

		let vv = new_point - spline_state.control_points[spline_state.control_points.len() - 1];
		let v2 = vv / length(vv) * LINE_LIMIT;
		let new_point = spline_state.control_points[spline_state.control_points.len() - 1] + v2;
		
		println!("Adding a new point {:?}", new_point);
		// println!("Previous point was {:?}",
		// 	 spline_state.points[spline_state.points.len() - 1]);

		
		spline_state.control_points.push(new_point);
	    }
	
    }
}
