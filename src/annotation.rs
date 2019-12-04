extern crate glm;
extern crate gl;

use crate::program;
use crate::edit;
use crate::cylinder;
use crate::shaders;

use std::ffi::CString;
use std::collections::HashMap;

pub trait Annotation {
    fn get_str(&self) -> std::string::String;
    fn get_color(&self) -> glm::Vec4;
    
    fn alters_size(&self) -> bool {
	false
    }
    
    // Explicit angle requirements (how to preserve this in laplacian solver?)
    fn alters_angle(&self) -> bool {
	false 
    }
    
    fn get_size(&self) -> f32 {
	0.0
    }

    fn get_render_index(&self) -> usize {
	0
    }

    
    fn apply_size(&self, _sizes: &mut Vec<f32>) { }
}


pub struct SizeAnnotation {
    pub size: f32,
    pub index: usize,
}


impl Annotation for SizeAnnotation {
    fn get_str(&self) -> std::string::String {
	format!("Sets size to {}", self.size.to_string())
    }

    fn get_color(&self) -> glm::Vec4 {
	glm::vec4(0.0, 0.0, 1.0, 1.0)
    }

    fn alters_size(&self) -> bool {
	true
    }

    fn apply_size(&self, sizes: &mut Vec<f32>) {
	sizes[self.index] = self.size;
    }

    fn get_render_index(&self) -> usize {
	self.index
    }
}

pub struct AnnotationState {
    curr_cylinder_index: usize,
    curr_annotation_index: usize,
    
    points_vbo: gl::types::GLuint,
    colors_vbo: gl::types::GLuint,
    visual_vao: gl::types::GLuint,
}

impl Drop for AnnotationState {
    fn drop(&mut self) {
	unsafe {
	    gl::DeleteBuffers(1, &self.points_vbo);
	    gl::DeleteBuffers(1, &self.colors_vbo);
	    gl::DeleteVertexArrays(1, &self.visual_vao);
	}
    }
}

impl AnnotationState {
    pub fn new(edit_state : edit::EditState,
	       session: &program::Session) -> AnnotationState {

	let mut points_vbo: gl::types::GLuint = 0;
	let mut colors_vbo: gl::types::GLuint = 0;
	let mut visual_vao: gl::types::GLuint = 0;
	
	unsafe {
	    gl::GenBuffers(1, &mut points_vbo);
	    gl::GenBuffers(1, &mut colors_vbo);
	    gl::GenVertexArrays(1, &mut visual_vao);

	    gl::BindVertexArray(visual_vao);

	    gl::BindBuffer(gl::ARRAY_BUFFER, points_vbo);
	    gl::VertexAttribPointer(
		0, 3, gl::FLOAT,
		gl::FALSE, std::mem::size_of::<glm::Vec3>() as gl::types::GLint,
		std::ptr::null());

	    gl::BindBuffer(gl::ARRAY_BUFFER, colors_vbo);
	    gl::VertexAttribPointer(
		1, 4, gl::FLOAT,
		gl::FALSE, std::mem::size_of::<glm::Vec4>() as gl::types::GLint,
		std::ptr::null());

	    gl::EnableVertexAttribArray(0);
	    gl::EnableVertexAttribArray(1);
	}
	
	let ass = AnnotationState {
	    curr_cylinder_index: 0,
	    curr_annotation_index: 0,
	    points_vbo,
	    colors_vbo,
	    visual_vao
	};

	update_gpu_annotation_state(&ass, session);

	ass
    }
}

pub fn handle_annotation(proj: &glm::Mat4, input_state: &program::InputState,
			 annotation_state: &mut AnnotationState,
			 session: &mut program::Session) { 
    
}


static ANNOTATION_X_OFFSET: f32 = 0.05;
static ANNOTATION_Y_OFFSET: f32 = 0.04;

pub fn update_gpu_annotation_state(annotation_state: &AnnotationState,
				   session: &program::Session) {

    let cylinders = &session.cylinders;


    let mut sum = 0;
    for i in &session.annotations {
	sum += i.len();
    }
    
    let mut positions : Vec<glm::Vec3> = Vec::with_capacity(sum);
    let mut colors : Vec<glm::Vec4> = Vec::with_capacity(sum);

    for j in 0..cylinders.len() {
	let annotations = &session.annotations[j];
	let mut hash_map: HashMap<usize, usize> = HashMap::with_capacity(annotations.len());
	let mut nums : Vec<usize> = Vec::with_capacity(cylinders[j].
						       spline.
						       control_points.
						       len());

	for ann in annotations {
	    let ind = ann.get_render_index();
	    let newr;

	    let res = hash_map.get(&ind);
	    
	    match res {
		Some(n) => newr = n + 1,
		None    => newr = 1
	    };

	    hash_map.insert(ind, newr);
	}
	
	for _i in &cylinders[j].spline.control_points {
	    nums.push(0);
	}
	
	
	let mut xoffsets : Vec<f32> = Vec::with_capacity(annotations.len());
	for anni in 0..annotations.len() {
	    let ind = annotations[anni].get_render_index();
	    
	    match hash_map.get(&ind) {
		Some(n) => 
		    xoffsets.push((-(*n as f32 - 1.0) / 2.0 + nums[ind] as f32) * ANNOTATION_X_OFFSET),
		None => {}
	    };
	    
	    nums[ind] += 1;
	}

	for anni in 0..annotations.len() {
	    positions.push(cylinders[j].spline.control_points[annotations[anni].get_render_index()] +
			   glm::vec3(xoffsets[anni], ANNOTATION_Y_OFFSET, 0.0)); // Copy trait?
	    println!("Pushed position {:?}", positions[positions.len() - 1]);
	    colors.push(annotations[anni].get_color());
	}
    }

    println!("Position length: {}", positions.len());

    unsafe {
	gl::BindVertexArray(annotation_state.visual_vao);
	
	gl::BindBuffer(gl::ARRAY_BUFFER, annotation_state.points_vbo);
	gl::BufferData(
	    gl::ARRAY_BUFFER,
	    (positions.len() * std::mem::size_of::<glm::Vec3>()) as gl::types::GLsizeiptr,
	    positions.as_ptr() as *const gl::types::GLvoid,
	    gl::DYNAMIC_DRAW);

	gl::BindBuffer(gl::ARRAY_BUFFER, annotation_state.colors_vbo);
	gl::BufferData(
	    gl::ARRAY_BUFFER,
	    (colors.len() * std::mem::size_of::<glm::Vec4>()) as gl::types::GLsizeiptr,
	    colors.as_ptr() as *const gl::types::GLvoid,
	    gl::DYNAMIC_DRAW);

	gl::EnableVertexAttribArray(0);
	gl::EnableVertexAttribArray(1);
    }
}

pub fn draw_annotations(session: &program::Session,
			annotation_state : &AnnotationState, 
			annotation_program: &shaders::ShaderProgram,
			transform : &glm::Mat4) {
    let annotations = &session.annotations;

    annotation_program.activate();
    
    let transform_location = unsafe {
	gl::GetUniformLocation(annotation_program.id,
			       CString::new("trans").unwrap().as_ptr())
    };

    unsafe {
	gl::UniformMatrix4fv(transform_location,
			     1, gl::FALSE, &transform[0][0]);
    }

    let mut sum = 0;
    for i in annotations {
	sum += i.len();
    }
    
    unsafe {

	gl::Disable(gl::DEPTH_TEST);
	
	gl::BindVertexArray(annotation_state.visual_vao);
	
	gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
	gl::EnableVertexAttribArray(0);
	gl::EnableVertexAttribArray(1);
	
	gl::PointSize(8.0);
	gl::DrawArrays(gl::POINTS, 0, sum as i32);
	gl::Enable(gl::DEPTH_TEST);
    }
}

pub fn push_default_annotations(cylinder_num: usize,
				num_points: usize,
				annotations: &mut Vec<Vec<Box<dyn Annotation>>>) {
    let ll = annotations.len();
    annotations.push(Vec::new());
    annotations[ll].push(Box::<SizeAnnotation>::from( SizeAnnotation { size: 1.0,
								       index: 0 }));
    annotations[ll].push(Box::<SizeAnnotation>::from( SizeAnnotation { size: 1.0,
								       index: num_points - 1}));
						 
}
