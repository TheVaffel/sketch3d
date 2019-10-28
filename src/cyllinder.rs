
use crate::splinedraw;
use crate::objects;
use crate::lineobjects;
use crate::shaders;

use std::ffi::{CString};
use crate::Object;

use std::f32;

pub struct GeneralizedCyllinder {
    radius : f32,
    circ_resolution : usize,
    pub object : Object,
    pub line_object : lineobjects::LineObject,
    pub spline : splinedraw::SplineState,
}

impl GeneralizedCyllinder {
    pub fn update_mesh(self : &mut GeneralizedCyllinder) {
        let (vertices, indices) = get_cyllinder_values(self.radius, self.circ_resolution, &self.spline);

        self.object.vertices = vertices;
        self.object.indices = indices;
        
        self.object.update_gpu_state();
        self.line_object.update(self.object.vertices.clone(), &self.object.indices);
    }
}

pub fn draw_cyllinder(generalized_cyllinder : &GeneralizedCyllinder,
		      body_program : &shaders::ShaderProgram,
		      line_program : &shaders::ShaderProgram,
		      transform : &glm::Mat4) {
    let black_color = glm::vec4(0.0, 0.0, 0.0, 1.0);
    let white_color = glm::vec4(1.0, 1.0, 1.0, 1.0);

    let no_translation = glm::vec4(0.0, 0.0, 0.0, 0.0);
    let small_translation = glm::vec4(0.0, 0.0, -0.08, 0.0);

    body_program.activate();
    
    let transform_location = unsafe {
        gl::GetUniformLocation(body_program.id,
                               CString::new("trans").unwrap().as_ptr())
    };

    let displacement_location = unsafe {
	gl::GetUniformLocation(body_program.id,
			       CString::new("displacement").unwrap().as_ptr())
    };

    let color_location = unsafe {
	gl::GetUniformLocation(body_program.id,
			       CString::new("uni_color").unwrap().as_ptr())
    };
    
    unsafe {
	gl::UniformMatrix4fv(transform_location,
			     1, gl::FALSE, &transform[0][0]);
	gl::Uniform4fv(displacement_location,
		       1, &no_translation[0]);
	gl::Uniform4fv(color_location,
		       1, &black_color[0]);
	
	gl::LineWidth(1.0);
	gl::BindVertexArray(generalized_cyllinder.line_object.all_vao);

	gl::DisableVertexAttribArray(1);
	gl::DrawElements(
	    gl::LINES,
	    generalized_cyllinder.line_object.all_indices.len() as gl::types::GLsizei,
	    gl::UNSIGNED_INT,
	    std::ptr::null());


	gl::Uniform4fv(displacement_location,
		       1, &small_translation[0]);
	gl::Uniform4fv(color_location,
		       1, &white_color[0]);
	
	gl::BindVertexArray(generalized_cyllinder.object.vao);
	gl::DrawElements(
	    gl::TRIANGLES,
	    generalized_cyllinder.object.indices.len() as gl::types::GLsizei,
	    gl::UNSIGNED_INT,
	    std::ptr::null());

	gl::Uniform4fv(color_location,
		       1, &black_color[0]);
	gl::LineWidth(2.0);

	gl::BindVertexArray(generalized_cyllinder.line_object.vao);
	// println!("Number of lines to draw: {}", generalized_cyllinder.line_object.indices.len());
	gl::DrawElements(
	    gl::LINES,
	    generalized_cyllinder.line_object.indices.len() as gl::types::GLsizei,
	    gl::UNSIGNED_INT,
	    std::ptr::null());


	gl::Disable(gl::DEPTH_TEST);
	gl::EnableVertexAttribArray(1);
    }

    line_program.activate();

    let transform_location = unsafe {
	gl::GetUniformLocation(line_program.id,
			       CString::new("trans").unwrap().as_ptr())
    };

    unsafe {
	gl::UniformMatrix4fv(transform_location,
			     1, gl::FALSE, &transform[0][0]);
    }

    
    splinedraw::draw_spline_lines(&generalized_cyllinder.spline);

    splinedraw::draw_control_points(&generalized_cyllinder.spline);

    unsafe {
	gl::Enable(gl::DEPTH_TEST);
    }
}

pub fn get_cyllinder_values(radius : f32,
                            circ_resolution: usize,
                            spline_state : &splinedraw::SplineState)
                            -> (Vec<f32>, Vec<u32>) {
    let len_resolution = spline_state.spline_points.len() - 1;
    let icirc_resolution = circ_resolution as u32;
    
    // The hemisphere at each end: 2 * resolution panes around its circumference,
    // resolution - 1 of those in height, and then 2 * resolution triangles to close it on top.
    let num_end_triangles =
	(2 * circ_resolution * (circ_resolution - 1)) * 2 + 2 * circ_resolution;

    
    // len_resolution panes in length, 2 * circ_resolution panes in circumference. Two triangles per pane
    let num_base_triangles = len_resolution * 2 * circ_resolution * 2;
    
    let num_total_triangles = 2 * num_end_triangles + num_base_triangles;

    let mut indices = vec![0; 3 * num_total_triangles];

    // Create base
    for i in 0..len_resolution {
	let ii = i as u32;
	for j in 0..(circ_resolution * 2) {
	    let ij = j as u32;
	    indices[6  * (i * 2 * circ_resolution + j) + 0] = ii       * 2 * icirc_resolution + ij;
	    indices[6  * (i * 2 * circ_resolution + j) + 1] = (ii + 1) * 2 * icirc_resolution + ij;
	    indices[6  * (i * 2 * circ_resolution + j) + 2] = (ii + 1) * 2 * icirc_resolution + ((ij + 1) % (2 * icirc_resolution));

	    indices[6  * (i * 2 * circ_resolution + j) + 3] = ii       * 2 * icirc_resolution + ij;
	    indices[6  * (i * 2 * circ_resolution + j) + 4] = (ii + 1) * 2 * icirc_resolution + ((ij + 1) % (2 * icirc_resolution));
	    indices[6  * (i * 2 * circ_resolution + j) + 5] = ii       * 2 * icirc_resolution + ((ij + 1) % (2 * icirc_resolution));
	}
    }

    for k in 0..2 {
	
	let ind_base = 3 * num_base_triangles + k * 3 * num_end_triangles;
	let vert_base = (2 * circ_resolution * (len_resolution + 1) + k * (2 * circ_resolution * (circ_resolution - 1) + 1)) as u32;
	let dir = (if k == 0 {-1} else {1}) as i32;
	let start_j = (if k == 0 {circ_resolution * 2} else {0}) as i32;
			 
	for i in 0..(circ_resolution - 1) {
	    let ii = i as u32;
	    for j in 0..(circ_resolution * 2) {
		let ij = j as i32;
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 0] =
		    vert_base + ii * 2 * icirc_resolution + (start_j + dir * ij      ) as u32 % (2 * icirc_resolution) - 2 * icirc_resolution;
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 1] =
		    vert_base + ii * 2 * icirc_resolution + (start_j + dir * ij      ) as u32 % (2 * icirc_resolution);
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 2] =
		    vert_base + ii * 2 * icirc_resolution + (start_j + dir * (ij + 1)) as u32 % (2 * icirc_resolution);

		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 3] =
		    vert_base + ii * 2 * icirc_resolution + (start_j + dir * ij      ) as u32 % (2 * icirc_resolution) - 2 * icirc_resolution;
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 4] =
		    vert_base + ii * 2 * icirc_resolution + (start_j + dir * (ij + 1)) as u32 % (2 * icirc_resolution);
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 5] =
		    vert_base + ii * 2 * icirc_resolution + (start_j + dir * (ij + 1)) as u32 % (2 * icirc_resolution) - 2 * icirc_resolution;
	    }
	} 

	// Redo bottom of hemisphere - should reuse old vertices
	let prev_base = (if k == 0 {0} else {circ_resolution * 2 * len_resolution}) as u32;
	let start_vert = (if k == 0 {circ_resolution * 2} else {0}) as i32;
	for i in 0..(circ_resolution * 2) {
	    let ii = i as i32;
	    indices[ind_base + 6 * i + 0] = prev_base + (start_vert + dir * ii      ) as u32 % (icirc_resolution * 2);

	    indices[ind_base + 6 * i + 3] = prev_base + (start_vert + dir * ii      ) as u32 % (icirc_resolution * 2);

	    indices[ind_base + 6 * i + 5] = prev_base + (start_vert + dir * (ii + 1)) as u32 % (icirc_resolution * 2);
	} 


	// Top of hemisphere
	for i in 0..(circ_resolution * 2) {
	    let ii = i as i32;
	    indices[ind_base + 6 * (circ_resolution - 1) * 2 * circ_resolution + 3 * i + 0] =
		vert_base + 2 * icirc_resolution * (icirc_resolution - 2) + (start_j + dir * ii      ) as u32 % (icirc_resolution * 2);
	    indices[ind_base + 6 * (circ_resolution - 1) * 2 * circ_resolution + 3 * i + 1] =
		vert_base + 2 * icirc_resolution * (icirc_resolution - 1);
	    indices[ind_base + 6 * (circ_resolution - 1) * 2 * circ_resolution + 3 * i + 2] =
		vert_base + 2 * icirc_resolution * (icirc_resolution - 2) + (start_j + dir * (ii + 1)) as u32 % (icirc_resolution * 2);
	}
    }

    
    // Create vertices

    let num_base_vertices = circ_resolution * 2 * (len_resolution + 1);
    let num_end_vertices = circ_resolution * 2 * (circ_resolution - 1) + 1;

    let num_total_vertices = 2 * num_end_vertices + num_base_vertices;

    let mut vertices : Vec<f32> = vec![0.0; num_total_vertices * 3];

    // Create base
    let base_length = 1.0; // length - 2.0 * radius;
    
    for i in 0..(len_resolution + 1) {
	let ai = if i == len_resolution { i - 1 } else { i };
	let z_dir = glm::builtin::normalize(glm::vec3(spline_state.spline_points[ai + 1].x,
						      spline_state.spline_points[ai + 1].y,
						      0.0) -
					    glm::vec3(spline_state.spline_points[ai].x,
						      spline_state.spline_points[ai].y,
						      0.0));
	let y_dir = glm::vec3(-z_dir.y, z_dir.x, 0.0);
	let x_dir = glm::vec3(0.0, 0.0, 1.0);
	
	for j in 0..(circ_resolution * 2) {
	    let ij = j as u32;
	    let theta = ij as f32 * f32::consts::PI * 2.0 / (circ_resolution * 2) as f32;
	    
	    let vertex = glm::vec3(spline_state.spline_points[ai + 1].x,
				   spline_state.spline_points[ai + 1].y,
				   0.0) * base_length +
		y_dir * theta.sin() * radius +
		x_dir * theta.cos() * radius;

	    vertices[3 * (i * circ_resolution * 2 + j) + 0] = vertex.x;
	    vertices[3 * (i * circ_resolution * 2 + j) + 1] = vertex.y;
	    vertices[3 * (i * circ_resolution * 2 + j) + 2] = vertex.z;
	}
    }

    // Create hemispheres
    for k in 0..2 {
	let factor = if k == 0 {-1.0} else {1.0};

	let vert_base = 3 * (num_base_vertices + k * num_end_vertices);

	let centerxy = if k == 0 { spline_state.spline_points[0] }
	else { spline_state.spline_points[spline_state.spline_points.len() - 1] } ;
	let center = glm::vec3(centerxy.x, centerxy.y, 0.0) * base_length;

	let z_dirxy = if k == 0 { spline_state.spline_points[0] -
				  spline_state.spline_points[6] }
	else {spline_state.spline_points[len_resolution] -
	      spline_state.spline_points[len_resolution - 6] };


	let z_dir = glm::builtin::normalize(glm::vec3(z_dirxy.x, z_dirxy.y, 0.0));
	let y_dir = glm::vec3(- factor * z_dir.y, factor * z_dir.x, 0.0);
	let x_dir = glm::vec3(0.0, 0.0, 1.0);
	
	for i in 0..(circ_resolution - 1) {
	    let phi = (i + 1) as f32 * f32::consts::PI / 2.0 / circ_resolution as f32;
	    let cp = phi.cos();
	    let sp = phi.sin();
	    for j in 0..(circ_resolution * 2) {
		let theta = j as f32 * f32::consts::PI * 2.0 / (circ_resolution * 2) as f32;

		let vertex = center + z_dir * sp * radius +
		    y_dir * radius * cp * theta.sin() +
		    x_dir * radius * cp * theta.cos();
		
		vertices[vert_base + 3 * (i * circ_resolution * 2 + j) + 0] = vertex.x;
		vertices[vert_base + 3 * (i * circ_resolution * 2 + j) + 1] = vertex.y;
		vertices[vert_base + 3 * (i * circ_resolution * 2 + j) + 2] = vertex.z;
		
	    }
	}

	let vertex = center + z_dir * radius;
	
	vertices[vert_base + 3 * (num_end_vertices - 1) + 0] = vertex.x; 
	vertices[vert_base + 3 * (num_end_vertices - 1) + 1] = vertex.y;
	vertices[vert_base + 3 * (num_end_vertices - 1) + 2] = vertex.z;
    }

    (vertices, indices)
}

pub fn create_cyllinder(radius : f32,
			circ_resolution: usize,
			mut spline_state : splinedraw::SplineState) -> GeneralizedCyllinder {
    
    splinedraw::spline_screen_to_world_transform(&mut spline_state);
    
    let (vertices, indices) = get_cyllinder_values(radius, circ_resolution, &spline_state);

    GeneralizedCyllinder {
	line_object: lineobjects::create_line_object(&vertices, &indices),
	object: objects::create_object(vertices, indices),
	spline: spline_state,
        radius,
        circ_resolution}
	
}


