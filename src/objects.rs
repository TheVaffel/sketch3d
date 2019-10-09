use std::f32;
use std::usize;

use crate::Object;

pub fn create_object(vertices: Vec<f32>,
		     indices: Vec<u32>) -> Object {
    let mut vbo: gl::types::GLuint = 0;
    unsafe {
	gl::GenBuffers(1, &mut vbo);
	gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
	gl::BufferData(
	    gl::ARRAY_BUFFER,
	    (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
	    vertices.as_ptr() as *const gl::types::GLvoid,
	    gl::STATIC_DRAW
	);
    }

    let mut vao: gl::types::GLuint = 0;
    unsafe {
	gl::GenVertexArrays(1, &mut vao);
	gl::BindVertexArray(vao);
	gl::VertexAttribPointer(
	    0, 3, gl::FLOAT,
	    gl::FALSE, (3 * std::mem::size_of::<f32>()) as gl::types::GLint,
	    std::ptr::null());
	gl::EnableVertexAttribArray(0);
    }
    

    let mut ebo: gl::types::GLuint = 0;
    unsafe {
	gl::GenBuffers(1, &mut ebo);
	gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
	gl::BufferData(
	    gl::ELEMENT_ARRAY_BUFFER,
	    (indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
	    indices.as_ptr() as *const gl::types::GLvoid,
	    gl::STATIC_DRAW
	);
    }

    unsafe {
	// gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
	gl::BindBuffer(gl::ARRAY_BUFFER, 0);
	gl::BindVertexArray(0);
    }

    Object { vbo, ebo, vao, vertices: vertices, indices: indices } 
}


pub fn create_cube_object(side: f32) -> Object {
    let cube_numbers = [0, 1, 3, 2, 6, 4, 5, 7];

    let mut cube_vertices : Vec<f32> = vec![0.0; 3 * 8];

    for i in 0..8 {
	for j  in 0..3 {
	    if (cube_numbers[i] & (1 << j)) != 0 {
		cube_vertices[3 * i + j] =   side / 2.0;
	    } else {
		cube_vertices[3 * i + j] = - side / 2.0;
	    }
	}
    }

    let mut cube_indices : Vec<u32>  = vec![0; 3 * 12];
    let pivots = [0, 7];

    for i in 0..2 {
	for j in 0..6 {
	    cube_indices[3 * (i * 6 + j)] = pivots[i];
	    cube_indices[3 * (i * 6 + j) + (if i > 0 {1} else {2})] = 1 + j as u32;
	    cube_indices[3 * (i * 6 + j) + (if i > 0 {2} else {1})] = 1 + ((j + 1) % 6) as u32;
	}
    }

    create_object(cube_vertices,
		  cube_indices)
}

pub fn create_sphere_object(radius: f32,
			    resolution: usize) -> Object {
    let mut data : Vec<f32> = vec![0.0; 3 * (resolution * 2 * (resolution - 1) + 2)];

    let invres = 1.0 / resolution as f32;

    let iresolution :u32 = resolution as u32;
    println!("Resolutions: {} {}", iresolution, resolution);

    for i in 0..(resolution - 1) {
	let phi = f32::consts::PI * (invres * (i + 1) as f32 - 1.0 / 2.0);
	let cphi = phi.cos();
	let sphi = phi.sin();

	for j in 0..(resolution * 2) {
	    let theta = f32::consts::PI * invres * j as f32;
	    let ctheta = theta.cos();
	    let stheta = theta.sin();
	    
	    data[3 * (i * resolution * 2 + j) + 0] = cphi * ctheta * radius;
	    data[3 * (i * resolution * 2 + j) + 1] = sphi * radius;
	    data[3 * (i * resolution * 2 + j) + 2] = cphi * stheta * radius;
	}
    }
    
    data[3 * (resolution * 2 * (resolution - 1) + 0) + 0] = 0.0;
    data[3 * (resolution * 2 * (resolution - 1) + 0) + 1] = - radius;
    data[3 * (resolution * 2 * (resolution - 1) + 0) + 2] = 0.0;

    data[3 * (resolution * 2 * (resolution - 1) + 1) + 0] = 0.0;
    data[3 * (resolution * 2 * (resolution - 1) + 1) + 1] = radius;
    data[3 * (resolution * 2 * (resolution - 1) + 1) + 2] = 0.0;

    let mut indices : Vec<u32> =
	vec![0; 6 * resolution * 2 * (resolution - 2) + 2 * 3 * 2 * resolution];

    println!("Index vector: {:?}", indices);
    
    for i in 0..(resolution - 2) {
	let ii :u32 = i as u32;
	for j in 0..(resolution * 2) {
	    let ij :u32 = j as u32;
	    indices[6 * (resolution * 2 * i + j) + 0] =
		(ii + 0) * 2 * iresolution + ((ij + 0) % (2 * iresolution));
            indices[6 * (resolution * 2 * i + j) + 1] =
		(ii + 0) * 2 * iresolution + ((ij + 1) % (2 * iresolution));
	    indices[6 * (resolution * 2 * i + j) + 2] =
		(ii + 1) * 2 * iresolution + ((ij + 0) % (2 * iresolution));

            indices[6 * (resolution * 2 * i + j) + 3] =
		(ii + 1) * 2 * iresolution + ((ij + 0) % (2 * iresolution));
            indices[6 * (resolution * 2 * i + j) + 4] =
		(ii + 0) * 2 * iresolution + ((ij + 1) % (2 * iresolution));
            indices[6 * (resolution * 2 * i + j) + 5] =
		(ii + 1) * 2 * iresolution + ((ij + 1) % (2 * iresolution)); 
	}
    }

    for i in 0..(resolution * 2) {
	let ii : u32 = i as u32;
	indices[6 * (resolution * 2 * (resolution - 2)) + 3 * i + 0] =
            iresolution * 2 * (iresolution - 1) + 0;
	indices[6 * (resolution * 2 * (resolution - 2)) + 3 * i + 1] =
            (ii + 1) % (2 * iresolution);
	indices[6 * (resolution * 2 * (resolution - 2)) + 3 * i + 2] =
            ii;

	indices[6 * (resolution * 2 * (resolution - 2)) + 3 * (resolution * 2 + i) + 0] =
            iresolution * 2 * (iresolution - 1) + 1;
	indices[6 * (resolution * 2 * (resolution - 2)) + 3 * (resolution * 2 + i) + 1] =
            iresolution * 2 * (iresolution - 2) + ii;
	indices[6 * (resolution * 2 * (resolution - 2)) + 3 * (resolution * 2 + i) + 2] =
            iresolution * 2 * (iresolution - 2) + ((ii + 1) % (2 * iresolution));
    }

    create_object(data, indices)
}


pub fn create_cone_object(radius :f32,
		      height :f32,
		      resolution :usize) -> Object {

    let iresolution :u32 = resolution as u32;
    
    let num_vertices = resolution + 1;
    let mut vertices :Vec<f32> = vec![0.0; num_vertices * 3];

    for i in 0..resolution {
        let theta = i as f32  * 2.0 * f32::consts::PI / (resolution as f32);
        vertices[3 * i + 0] = radius * theta.cos();
        vertices[3 * i + 1] = height / 2.0;
        vertices[3 * i + 2] = radius * theta.sin();
    }

    vertices[3 * resolution + 0] = 0.0;
    vertices[3 * resolution + 1] = - height / 2.0;
    vertices[3 * resolution + 2] = 0.0;

    let num_indices = 3 * resolution + 3 * (resolution - 2);
    
    let mut indices : Vec<u32> = vec![0; num_indices];
    for i in 0..resolution {
	let ii : u32 = i as u32;
	
        indices[3 * i + 0] = ii;
        indices[3 * i + 1] = (ii + 1) % iresolution;
        indices[3 * i + 2] = iresolution;
    }

    for i in 1..(resolution - 1) {
	let ii = i as u32;
        indices[3 * resolution + 3 * (i - 1) + 0] = ii + 1;
        indices[3 * resolution + 3 * (i - 1) + 1] = ii;
        indices[3 * resolution + 3 * (i - 1) + 2] = 0;
    }

    create_object(vertices, indices)
}

pub fn create_generalized_cyllinder_object(radius : f32,
					   length : f32,
					   circ_resolution : usize,
					   len_resolution : usize) -> Object {
    let icirc_resolution = circ_resolution as u32;
    let ilen_resolution = len_resolution as u32;
    
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
		    vert_base + ii       * 2 * icirc_resolution + (start_j + dir * ij      ) as u32 % (2 * icirc_resolution);
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 2] =
		    vert_base + ii       * 2 * icirc_resolution + (start_j + dir * (ij + 1)) as u32 % (2 * icirc_resolution);

		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 3] =
		    vert_base + ii * 2 * icirc_resolution + (start_j + dir * ij      ) as u32 % (2 * icirc_resolution) - 2 * icirc_resolution;
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 4] =
		    vert_base + ii       * 2 * icirc_resolution + (start_j + dir * (ij + 1)) as u32 % (2 * icirc_resolution);
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
    let base_length = length - 2.0 * radius;
    let base_mid = base_length / 2.0;
    
    for i in 0..(len_resolution + 1) {
	let ii = i as u32;
	for j in 0..(circ_resolution * 2) {
	    let ij = j as u32;
	    let theta = ij as f32 * f32::consts::PI * 2.0 / (circ_resolution * 2) as f32;

	    // Orient along x-axis
	    vertices[3 * (i * circ_resolution * 2 + j) + 0] = base_length * ii as f32 / len_resolution as f32 - base_mid;
	    vertices[3 * (i * circ_resolution * 2 + j) + 1] = theta.sin() * radius;
	    vertices[3 * (i * circ_resolution * 2 + j) + 2] = theta.cos() * radius;
	}
    }

    // Create hemispheres
    for k in 0..2 {
	let factor = if k == 0 {-1} else {1};

	let vert_base = 3 * (num_base_vertices + k * num_end_vertices);

	for i in 0..(circ_resolution - 1) {
	    let phi = (i + 1) as f32 * f32::consts::PI / 2.0 / circ_resolution as f32;
	    let cp = phi.cos();
	    let sp = phi.sin();
	    for j in 0..(circ_resolution * 2) {
		let theta = j as f32 * f32::consts::PI * 2.0 / (circ_resolution * 2) as f32;
		
		vertices[vert_base + 3 * (i * circ_resolution * 2 + j) + 0] =
		    factor as f32 * (base_mid + sp * radius);
		vertices[vert_base + 3 * (i * circ_resolution * 2 + j) + 1] =
		    radius * cp * theta.sin();
		vertices[vert_base + 3 * (i * circ_resolution * 2 + j) + 2] =
		    radius * cp * theta.cos();

		
	    }
	}

	vertices[vert_base + 3 * (num_end_vertices - 1) + 0] = factor as f32 * (base_mid + radius);
	vertices[vert_base + 3 * (num_end_vertices - 1) + 1] = 0.0;
	vertices[vert_base + 3 * (num_end_vertices - 1) + 2] = 0.0;
    }

    create_object(vertices, indices)
}


// C++

/*

Object* createGeneralizedCyllinderObject(float radius, float length, int circ_resolution, int len_resolution) {
    int num_end_triangles = (2 * circ_resolution * (circ_resolution - 1)) * 2 + 2 * circ_resolution; // The hemisphere at each end: 2 * resolution panes around its circumference, resolution - 1 of those in height, and then 2 * resolution triangles to close it on top.

    int num_base_triangles = len_resolution * 2 * circ_resolution * 2; // len_resolution panes in length, 2 * circ_resolution panes in circumference. Two triangles per pane

    int num_total_triangles = 2 * num_end_triangles + num_base_triangles;

    std::vector<uint32_t> indices(3 * num_total_triangles);

    // Create base
    for(int i = 0; i < len_resolution; i++) {
	for(int j = 0; j < circ_resolution * 2; j++) {
	    indices[6  * (i * 2 * circ_resolution + j) + 0] = i       * 2 * circ_resolution + j;
	    indices[6  * (i * 2 * circ_resolution + j) + 1] = (i + 1) * 2 * circ_resolution + j;
	    indices[6  * (i * 2 * circ_resolution + j) + 2] = (i + 1) * 2 * circ_resolution + ((j + 1) % (2 * circ_resolution));

	    indices[6  * (i * 2 * circ_resolution + j) + 3] = i       * 2 * circ_resolution + j;
	    indices[6  * (i * 2 * circ_resolution + j) + 4] = (i + 1) * 2 * circ_resolution + ((j + 1) % (2 * circ_resolution));
	    indices[6  * (i * 2 * circ_resolution + j) + 5] = i       * 2 * circ_resolution + ((j + 1) % (2 * circ_resolution));
	}
    }

    for(int k = 0; k < 2; k++) {
	int ind_base = 3 * num_base_triangles + k * 3 * num_end_triangles;
	int vert_base = 2 * circ_resolution * (len_resolution + 1) + k * (2 * circ_resolution * (circ_resolution - 1) + 1);
	int dir = k == 0 ? -1 : 1;
	int start_j = k == 0 ? circ_resolution * 2 : 0;
			 
	for(int i = 0; i < circ_resolution - 1; i++) {

	    for(int j = 0; j < circ_resolution * 2; j++) {
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 0] =
		    vert_base + (i - 1) * 2 * circ_resolution + (start_j + dir * j      ) % (2 * circ_resolution);
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 1] =
		    vert_base + i       * 2 * circ_resolution + (start_j + dir * j      ) % (2 * circ_resolution);
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 2] =
		    vert_base + i       * 2 * circ_resolution + (start_j + dir * (j + 1)) % (2 * circ_resolution);

		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 3] =
		    vert_base + (i - 1) * 2 * circ_resolution + (start_j + dir * j      ) % (2 * circ_resolution);
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 4] =
		    vert_base + i       * 2 * circ_resolution + (start_j + dir * (j + 1)) % (2 * circ_resolution);
		indices[ind_base + 6 * (i * 2 * circ_resolution + j) + 5] =
		    vert_base + (i - 1) * 2 * circ_resolution + (start_j + dir * (j + 1)) % (2 * circ_resolution);
	    }
	} 

	// Redo bottom of hemisphere - should reuse old vertices
	int prev_base = k == 0 ? 0 : (circ_resolution * 2 * len_resolution);
	int start_vert = k == 0 ? circ_resolution * 2 : 0;
	for(int i = 0; i < circ_resolution * 2; i++) {
	    indices[ind_base + 6 * i + 0] = prev_base + (start_vert + dir * i      ) % (circ_resolution * 2);

	    indices[ind_base + 6 * i + 3] = prev_base + (start_vert + dir * i      ) % (circ_resolution * 2);

	    indices[ind_base + 6 * i + 5] = prev_base + (start_vert + dir * (i + 1)) % (circ_resolution * 2);
	} 


	// Top of hemisphere
	for(int i = 0; i < circ_resolution * 2; i++) {
	    indices[ind_base + 6 * (circ_resolution - 1) * 2 * circ_resolution + 3 * i + 0] =
		vert_base + 2 * circ_resolution * (circ_resolution - 2) + (start_j + dir * i      ) % (circ_resolution * 2);
	    indices[ind_base + 6 * (circ_resolution - 1) * 2 * circ_resolution + 3 * i + 1] =
		vert_base + 2 * circ_resolution * (circ_resolution - 1);
	    indices[ind_base + 6 * (circ_resolution - 1) * 2 * circ_resolution + 3 * i + 2] =
		vert_base + 2 * circ_resolution * (circ_resolution - 2) + (start_j + dir * (i + 1)) % (circ_resolution * 2);
	}
    }

    
    // Create vertices

    int num_base_vertices = circ_resolution * 2 * (len_resolution + 1);
    int num_end_vertices = circ_resolution * 2 * (circ_resolution - 1) + 1;

    int num_total_vertices = 2 * num_end_vertices + num_base_vertices;

    std::vector<float> vertices(num_total_vertices * 3);

    // Create base
    float base_length = length - 2 * radius;
    float base_mid = base_length / 2;
    
    for(int i = 0; i < len_resolution + 1; i++) {
	for(int j = 0; j < circ_resolution * 2; j++) {
	    float theta = j * M_PI * 2 / (circ_resolution * 2);

	    // Orient along x-axis
	    vertices[3 * (i * circ_resolution * 2 + j) + 0] = base_length * i / len_resolution - base_mid;
	    vertices[3 * (i * circ_resolution * 2 + j) + 1] = sin(theta) * radius;
	    vertices[3 * (i * circ_resolution * 2 + j) + 2] = cos(theta) * radius;
	}
    }

    // Create hemispheres
    for(int k = 0; k < 2; k++) {
	int factor = k == 0 ? -1 : 1;

	int vert_base = 3 * (num_base_vertices + k * num_end_vertices);

	for(int i = 0; i < circ_resolution - 1; i++) {
	    float phi = (i + 1) * M_PI / 2 / circ_resolution;
	    float cp = cos(phi);
	    float sp = sin(phi);
	    for(int j = 0; j < circ_resolution * 2; j++) {
		float theta = j * M_PI * 2 / (circ_resolution * 2);
		
		vertices[vert_base + 3 * (i * circ_resolution * 2 + j) + 0] =
		    factor * (base_mid + sp * radius);
		vertices[vert_base + 3 * (i * circ_resolution * 2 + j) + 1] =
		    radius * cp * sin(theta);
		vertices[vert_base + 3 * (i * circ_resolution * 2 + j) + 2] =
		    radius * cp * cos(theta);

		
	    }
	}

	vertices[vert_base + 3 * (num_end_vertices - 1) + 0] = factor * (base_mid + radius);
	vertices[vert_base + 3 * (num_end_vertices - 1) + 1] = 0;
	vertices[vert_base + 3 * (num_end_vertices - 1) + 2] = 0;
    }


    Object* object = createVertexObject(vertices, indices);
    
    return object;
}


Object* createConeObject(float radius, float height, int resolution) {
    int num_vertices = resolution + 1;
    std::vector<float> vertices(num_vertices * 3);

    for(int i = 0;  i < resolution; i++) {
        float theta = i * 2 * M_PI / resolution;
        vertices[3 * i + 0] = radius * cos(theta);
        vertices[3 * i + 1] = height / 2;
        vertices[3 * i + 2] = radius * sin(theta);
    }

    vertices[3 * resolution + 0] = 0.0f;
    vertices[3 * resolution + 1] = -height / 2;
    vertices[3 * resolution + 2] = 0.0f;

    int num_indices = 3 * resolution + 3 * (resolution - 2);
    std::vector<uint32_t> indices(num_indices);
    for(int i = 0; i < resolution; i++) {
        indices[3 * i + 0] = i;
        indices[3 * i + 1] = (i + 1) % resolution;
        indices[3 * i + 2] = resolution;
    }

    for(int i = 1; i < resolution - 1; i++) {
        indices[3 * resolution + 3 * (i - 1) + 0] = i + 1;
        indices[3 * resolution + 3 * (i - 1) + 1] = i;
        indices[3 * resolution + 3 * (i - 1) + 2] = 0;
    }

    Object* object = createVertexObject(vertices, indices);

    return object;
}

Object* createSphereObject(float radius, int resolution) {


    std::vector<float> data(3 * (resolution * 2 * (resolution - 1) + 2));

    float invres = 1.0f / resolution;
    
    for(int i = 0; i < resolution - 1; i++) {
        float phi = M_PI * (invres * (i + 1) - 1.f / 2);
        float cphi = cosf(phi);
        float sphi = sinf(phi);
        for(int j = 0; j < resolution * 2; j++) {
            float theta = M_PI * (invres * j);
            float ctheta = cosf(theta);
            float stheta = sinf(theta);
            
            data[3 * (i * resolution * 2 + j) + 0] = cphi * ctheta * radius;
            data[3 * (i * resolution * 2 + j) + 1] = sphi * radius;
            data[3 * (i * resolution * 2 + j) + 2] = cphi * stheta * radius;
        }
    }

    data[3 * (resolution * 2 * (resolution - 1) + 0) + 0] = 0.f;
    data[3 * (resolution * 2 * (resolution - 1) + 0) + 1] = - radius;
    data[3 * (resolution * 2 * (resolution - 1) + 0) + 2] = 0.f;

    data[3 * (resolution * 2 * (resolution - 1) + 1) + 0] = 0.f;
    data[3 * (resolution * 2 * (resolution - 1) + 1) + 1] = radius;
    data[3 * (resolution * 2 * (resolution - 1) + 1) + 2] = 0.f;

    std::vector<uint32_t> indices(6 * resolution * 2 * (resolution - 2) + 2 * 3 * 2 * resolution);

    for(int i = 0; i < resolution - 2; i++){
      for(int j = 0; j < resolution * 2; j++) {
        indices[6 * (resolution * 2 * i + j) + 0] =
            (i + 0) * 2 * resolution + ((j + 0) % (2 * resolution));
        indices[6 * (resolution * 2 * i + j) + 1] =
            (i + 0) * 2 * resolution + ((j + 1) % (2 * resolution));
        indices[6 * (resolution * 2 * i + j) + 2] =
            (i + 1) * 2 * resolution + ((j + 0) % (2 * resolution));

        indices[6 * (resolution * 2 * i + j) + 3] =
            (i + 1) * 2 * resolution + ((j + 0) % (2 * resolution));
        indices[6 * (resolution * 2 * i + j) + 4] =
            (i + 0) * 2 * resolution + ((j + 1) % (2 * resolution));
        indices[6 * (resolution * 2 * i + j) + 5] =
            (i + 1) * 2 * resolution + ((j + 1) % (2 * resolution));
      }
    }

    for(int i = 0; i < resolution * 2; i++) {
      indices[6 * (resolution * 2 * (resolution - 2)) + 3 * i + 0] =
          resolution * 2 * (resolution - 1) + 0;
      indices[6 * (resolution * 2 * (resolution - 2)) + 3 * i + 1] =
          (i + 1) % (2 * resolution);
      indices[6 * (resolution * 2 * (resolution - 2)) + 3 * i + 2] =
          i;

      indices[6 * (resolution * 2 * (resolution - 2)) + 3 * (resolution * 2 + i) + 0] =
        resolution * 2 * (resolution - 1) + 1;
      indices[6 * (resolution * 2 * (resolution - 2)) + 3 * (resolution * 2 + i) + 1] =
        resolution * 2 * (resolution - 2) + i;
      indices[6 * (resolution * 2 * (resolution - 2)) + 3 * (resolution * 2 + i) + 2] =
        resolution * 2 * (resolution - 2) + ((i + 1) % (2 * resolution));
    }

    Object* object = createVertexObject(data, indices);

    return object;
}
 */
    
/*
Object* createCubeObject(float side) {
        //Dirty trick for creating cube:
        int cubeNumbers[8] = {0, 1, 3, 2, 6, 4, 5, 7};

        std::vector<float> cube_vertices(3*8);
        for(int i = 0; i < 8; i++){
                int pop = 0;
                for(int j = 0; j < 3; j++){
                        if(cubeNumbers[i] & (1 << j)){
                                cube_vertices[3*i + j] = side / 2;
                                pop++;
                        }else{
                                cube_vertices[3*i + j] = - side / 2;
                        }
                }
        }

        std::vector<uint32_t> cube_indices(3*12);
        int pivots[2] = {0, 7};
        for(int i = 0; i < 2; i++){
                for(int j = 0; j < 6; j++){
                        cube_indices[3*(i*6 + j)] = pivots[i];
                        cube_indices[3*(i*6 + j) + (i?1:2)] = 1 + j;
                        cube_indices[3*(i*6 + j) + (i?2:1)] = 1 + ((j + 1) % 6);
                }
        }

        Object* object = createVertexObject(cube_vertices,
                                            cube_indices);
        
        return object;
} */
