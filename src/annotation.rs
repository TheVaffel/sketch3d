extern crate glm;
extern crate gl;

use crate::program;
use crate::edit;
use crate::cylinder;
use crate::shaders;

use std::cell::Cell;
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


struct SizeAnnotation {
    size: f32,
    index: usize,
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
    pub annotations: Vec<Box<dyn Annotation>>,
    pub cylinders: Vec<cylinder::GeneralizedCylinder>,
    pub curr_cylinder: usize,
    
    points_vbo: gl::types::GLuint,
    colors_vbo: gl::types::GLuint,
    visual_vao: gl::types::GLuint,
}

impl AnnotationState {
    pub fn new(edit_state : edit::EditState) -> AnnotationState {

	let mut points_vbo: gl::types::GLuint = 0;
	let mut colors_vbo: gl::types::GLuint = 0;
	let mut visual_vao: gl::types::GLuint = 0;
	
	unsafe {
	    gl::GenBuffers(1, &mut points_vbo);
	    gl::GenBuffers(1, &mut colors_vbo);
	    gl::GenVertexArrays(1, &mut visual_vao);

	    gl::BindVertexArray(visual_vao);
	    gl::VertexAttribPointer(
		0, 3, gl::FLOAT,
		gl::FALSE, std::mem::size_of::<glm::Vec3>() as gl::types::GLint,
		std::ptr::null());
	    gl::VertexAttribPointer(
		1, 4, gl::FLOAT,
		gl::FALSE, std::mem::size_of::<glm::Vec4>() as gl::types::GLint,
		std::ptr::null());

	    gl::EnableVertexAttribArray(0);
	    gl::EnableVertexAttribArray(0);
	}
	
	let ass = AnnotationState {
	    annotations: vec![Box::<SizeAnnotation>::from(SizeAnnotation { size: 1.0, index: 0 }),
			      Box::<SizeAnnotation>::from(SizeAnnotation { size: 1.0, index: 0 })], // Vec::new(),
	    cylinders: edit_state.cylinders,
	    curr_cylinder: edit_state.curr_cylinder,
	    points_vbo,
	    colors_vbo,
	    visual_vao
	};

	update_gpu_annotation_state(&ass, &ass.cylinders[ass.curr_cylinder]);

	ass
    }
}

pub fn handle_annotation(proj: &glm::Mat4, input_state: &program::InputState,
			 annotation_state: &mut AnnotationState) { 
    
}


static ANNOTATION_X_OFFSET: f32 = 0.02;
static ANNOTATION_Y_OFFSET: f32 = 0.04;

pub fn update_gpu_annotation_state(annotation_state: &AnnotationState,
				   cylinder: &cylinder::GeneralizedCylinder) {

    let annotations = &annotation_state.annotations;
    let mut hash_map: HashMap<usize, usize> = HashMap::with_capacity(annotations.len());
    let mut nums : Vec<usize> = Vec::with_capacity(annotations.len());
    
    for ann in annotations {
	let ind = ann.get_render_index();
	let newr;

	let res = hash_map.get(&ind);
	
	match res {
	    Some(n) => newr = n + 1,
	    None    => newr = 1
	};

	hash_map.insert(ind, newr);

	nums.push(0);
    }
    
    let mut xoffsets : Vec<f32> = Vec::with_capacity(annotations.len());
    for anni in 0..annotations.len() {
	let ind = annotations[anni].get_render_index();
	
	match hash_map.get(&ind) {
	    Some(n) => 
		xoffsets.push((-(*n as f32) / 2.0 + nums[ind] as f32) * ANNOTATION_X_OFFSET),
	    None => {}
	};
	
	nums[ind] += 1;
    }
    
    let mut positions : Vec<glm::Vec3> = Vec::with_capacity(annotations.len());
    let mut colors : Vec<glm::Vec4> = Vec::with_capacity(annotations.len());

    for anni in 0..annotations.len() {
	positions.push(cylinder.spline.control_points[annotations[anni].get_render_index()] +
		       glm::vec3(xoffsets[anni], ANNOTATION_Y_OFFSET, 0.0)); // Copy trait?
	colors.push(annotations[anni].get_color());
    }

    println!("All positions: {:?}", positions);
    
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

pub fn draw_annotations(annotation_state : &AnnotationState, 
			annotation_program: &shaders::ShaderProgram) {
    let annotations = &annotation_state.annotations;
    
    unsafe {
	annotation_program.activate();
	
	gl::PointSize(8.0);
	gl::BindVertexArray(annotation_state.visual_vao);
	
	gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
	gl::EnableVertexAttribArray(0);
	gl::EnableVertexAttribArray(1);
	
	// gl::DrawArrays(gl::POINTS, 0, annotations.len() as i32);
    }
}
