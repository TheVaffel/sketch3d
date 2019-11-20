extern crate glm;

use crate::program;
use crate::edit;
use crate::cyllinder;

trait Annotation {
    fn get_str(&self) -> std::string::String;
    fn get_color(&self) -> glm::Vec3;

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

    fn get_color(&self) -> glm::Vec3 {
	glm::vec3(0.0, 0.0, 1.0)
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
    annotations: Vec<Box<dyn Annotation>>,
    pub cyllinders: Vec<cyllinder::GeneralizedCyllinder>,
    pub curr_cyllinder: usize
}

impl AnnotationState {
    pub fn new(edit_state : edit::EditState) -> AnnotationState {
	AnnotationState {
	    annotations: Vec::new(),
	    cyllinders: edit_state.cyllinders,
	    curr_cyllinder: edit_state.curr_cyllinder
	}
    }
}

pub fn handle_annotation(proj: &glm::Mat4, input_state: &program::InputState,
			 annotation_state: &mut AnnotationState) { 
    
}
