extern crate gl;
extern crate glm;

use crate::objects;

use std::f32;
use std::collections::HashMap;
use std::clone::Clone;

pub struct LineObject { 
    pub vao: gl::types::GLuint, 
    pub vbo: gl::types::GLuint,
    pub ebo: gl::types::GLuint,
    pub vertices: Vec<f32>,
    pub indices: Vec<u32>,
    pub all_indices: Vec<u32>,
    pub all_vao: gl::types::GLuint,
    pub all_ebo: gl::types::GLuint,
}

impl LineObject {
    pub fn update(self : &mut LineObject, vertices : Vec<f32>, indices : &Vec<u32>) {
        let (indices, all_indices) = get_lineobject_indices(&vertices, &indices);
        
        self.indices = indices;
        self.vertices = vertices;
        self.all_indices = all_indices;

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                self.vertices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                self.indices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW);

            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.all_ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (self.all_indices.len() * std::mem::size_of::<u32>()) as gl::types::GLsizeiptr,
                self.all_indices.as_ptr() as *const gl::types::GLvoid,
                gl::DYNAMIC_DRAW);
        }
    }
}

struct HalfEdge {
    ind0: u32,
    ind1: u32,
    opposite_num: i32,
    next_num: i32,
}

impl Clone for HalfEdge {
    fn clone(&self) -> HalfEdge {
	HalfEdge {ind0: self.ind0,
		  ind1: self.ind1,
		  opposite_num: self.opposite_num,
		  next_num: self.next_num}
    }
}

fn get_angle(h_edge_num: usize, vertices: &Vec<f32>, half_edges: &Vec<HalfEdge>) -> f32 {

    let h_edge = &half_edges[h_edge_num];
    
    if h_edge.opposite_num == -1 {
	return 0.0;
    }

    let uind0 = h_edge.ind0 as usize;
    let uind1 = h_edge.ind1 as usize;
    
    let edge0 = glm::vec3(vertices[3 * uind1 + 0] - vertices[3 * uind0 + 0],
			  vertices[3 * uind1 + 1] - vertices[3 * uind0 + 1],
			  vertices[3 * uind1 + 2] - vertices[3 * uind0 + 2]);
    
    let n_edge = &half_edges[half_edges[h_edge.next_num as usize].next_num as usize];
    let o_edge = &half_edges[half_edges[h_edge.opposite_num as usize].next_num as usize];

    let nuind0 = n_edge.ind0 as usize;
    let nuind1 = n_edge.ind1 as usize;

    let ouind0 = o_edge.ind0 as usize;
    let ouind1 = o_edge.ind1 as usize;
    
    let edge1 = glm::vec3(vertices[3 * nuind0 + 0] - vertices[3 * nuind1 + 0],
			  vertices[3 * nuind0 + 1] - vertices[3 * nuind1 + 1],
			  vertices[3 * nuind0 + 2] - vertices[3 * nuind1 + 2]);
    
    let edge2 = glm::vec3(vertices[3 * ouind1 + 0] - vertices[3 * ouind0 + 0],
			  vertices[3 * ouind1 + 1] - vertices[3 * ouind0 + 1],
			  vertices[3 * ouind1 + 2] - vertices[3 * ouind0 + 2]);

    let edge0_norm = glm::builtin::normalize(edge0);
    let pr_edge1 = edge1 - edge0_norm * glm::builtin::dot(edge0_norm, edge1);
    let pr_edge2 = edge2 - edge0_norm * glm::builtin::dot(edge0_norm, edge2);

    let cosang = glm::builtin::dot(pr_edge1, pr_edge2) / (glm::builtin::length(pr_edge1) *
							  glm::builtin::length(pr_edge2));
    let sinang = glm::builtin::length(glm::builtin::cross(pr_edge1, pr_edge2)) /
	(glm::builtin::length(pr_edge1) *
	 glm::builtin::length(pr_edge2));

    let angle = sinang.atan2(cosang);

    angle
}

pub fn create_line_object(old_vertices: &Vec<f32>,
			  old_indices:  &Vec<u32>) -> LineObject {

    
    let (indices, all_indices) =
        get_lineobject_indices(old_vertices,
                               old_indices);
    
    
    let vbo = objects::create_vbo(&old_vertices);

    let sharp_vao = objects::create_vao(vec![3]);
    let sharp_ebo = objects::create_ebo(&indices);

    let all_vao = objects::create_vao(vec![3]); 
    let all_ebo = objects::create_ebo(&all_indices);
    

    
    LineObject { vbo: vbo, vao: sharp_vao, ebo: sharp_ebo,
                 vertices : old_vertices.clone(), indices : indices,
                 all_indices : all_indices, all_vao : all_vao,
                 all_ebo : all_ebo }

}

fn get_lineobject_indices(old_vertices: &Vec<f32>,
                          old_indices : &Vec<u32>)
                         -> (Vec<u32>, Vec<u32>) {
    let mut indices: Vec<u32> = Vec::new();
    let mut all_indices: Vec<u32> = Vec::new();
    let mut half_edges: Vec<HalfEdge> =
	vec![HalfEdge{ind0: 0, ind1: 0, opposite_num: -1, next_num: -1};
	     old_indices.len()];
    
    let mut lines: HashMap<(u32, u32), u32> = HashMap::new();


    for i in 0..(old_indices.len() / 3) {
	for j in 0..3 {
	    let mut ind0 = old_indices[3 * i + j];
	    let mut ind1 = old_indices[3 * i + (j + 1) % 3];

	    if ind0 < ind1 {
		all_indices.push(ind0);
		all_indices.push(ind1);
	    }

	    let curr_half = 3 * i + j;
	    half_edges[curr_half].ind0 = ind0;
	    half_edges[curr_half].ind1 = ind1;

	    if ind0 > ind1 {
		std::mem::swap(&mut ind0, &mut ind1);
	    }

	    match lines.get(&(ind0, ind1)) {
		None => {
		    lines.insert((ind0, ind1), curr_half as u32);
		},
		Some(found) => {
		    half_edges[curr_half].opposite_num = *found as i32;
		    half_edges[*found as usize].opposite_num = curr_half as i32;
		},
	    };
	}

	for j in 0..3 {
	    half_edges[i * 3 + j].next_num =
		( i * 3 + (j + 1) % 3 ) as i32;
	}	
    }

    for i in 0..half_edges.len() {
	if half_edges[i].ind0 < half_edges[i].ind1 {
	    let ang = get_angle(i, &old_vertices, &half_edges);

	    if ang.abs() < f32::consts::PI * 3.0 / 5.0 {
		indices.push(half_edges[i].ind0);
		indices.push(half_edges[i].ind1);
	    }
	}
    }

    (indices, all_indices)
}


/* 
C++ code:

LineObject* createLineObject(const std::vector<float>& oldVertices,
                         const std::vector<uint32_t>& oldIndices) {
    std::vector<float> linePoints;
    std::vector<uint32_t> indices, allIndices; // allIndices creates a full line model
    std::vector<HalfEdge> halfEdges(oldIndices.size());

    std::map<std::pair<uint32_t, uint32_t>, uint32_t> lines;

    // Copy oldVertices
    linePoints = std::vector<float>(oldVertices.begin(), oldVertices.end());

    for(uint32_t i = 0; i < oldIndices.size() / 3; i++) {
        for(int j = 0; j < 3; j++) {
            
            int ind0 = oldIndices[3 * i + j];
            int ind1 = oldIndices[3 * i + (j + 1) % 3];

            if(ind0 < ind1) {
                allIndices.push_back(ind0);
                allIndices.push_back(ind1);
            }

            int curr_half = 3 * i + j;
            halfEdges[curr_half].ind0 = ind0;
            halfEdges[curr_half].ind1 = ind1;
                        
            if(ind0 > ind1) {
                std::swap(ind0, ind1);
            }

            if(lines.find(std::make_pair(ind0, ind1)) == lines.end()) {
                lines[std::make_pair(ind0, ind1)] = curr_half;
            } else {
                halfEdges[curr_half].opposite =
                    &halfEdges[lines[std::make_pair(ind0, ind1)]];
                halfEdges[lines[std::make_pair(ind0, ind1)]].opposite =
                    &halfEdges[curr_half];
            }
        }

        for(int j = 0; j < 3; j++) {
            halfEdges[i * 3 + j].next =
                &halfEdges[i * 3 + (j + 1) % 3];
        }
    }

        
    for(uint i = 0; i < halfEdges.size(); i++) {
        if(halfEdges[i].ind0 < halfEdges[i].ind1) {
            float ang = getAngle(&halfEdges[i], linePoints);
            if(abs(ang) < M_PI * 3 / 5) {
                indices.push_back(halfEdges[i].ind0);
                indices.push_back(halfEdges[i].ind1);
            }
        }
    }

// This obviously has some memory leak flaws on GPU, but hopefully
    // we won't notice
    Object* object = createVertexObject(linePoints, indices);

    Object* object2 = createVertexObject(linePoints, allIndices);

    
    std::cout << "allIndices size: " << allIndices.size() << std::endl;
    
    LineObject* lineObject = new LineObject();
    lineObject->vbo = object->vbo;
    lineObject->ebo = object->ebo;
    lineObject->vao = object->vao;
    lineObject->all_vao = object2->vao;
    lineObject->vertices = linePoints;
    lineObject->indices = indices;
    lineObject->all_indices = allIndices;

    delete object;
    delete object2;

    return lineObject;

}


static float getAngle(HalfEdge* h_edge, const std::vector<float>& vertices) {
    glm::vec3 edge0 = *(glm::vec3*)(vertices.data() + 3 * h_edge->ind1) -
        *(glm::vec3*)(vertices.data() + 3 * h_edge->ind0);

    if(h_edge->opposite == nullptr) {
        return 0;
    }
        
    HalfEdge* n_edge = h_edge->next->next;
    HalfEdge* o_edge = h_edge->opposite->next;

    // Back wards - same start point as edge0
    glm::vec3 edge1 = *(glm::vec3*)(vertices.data() + 3 * n_edge->ind0) -
        *(glm::vec3*)(vertices.data() + 3 * n_edge->ind1); 
    glm::vec3 edge2 = *(glm::vec3*)(vertices.data() + 3 * o_edge->ind1) -
        *(glm::vec3*)(vertices.data() + 3 * o_edge->ind0);

    glm::vec3 edge0_norm = glm::normalize(edge0);
        
    glm::vec3 pr_edge1 = edge1 - glm::dot(edge0_norm, edge1) * edge0_norm;
    glm::vec3 pr_edge2 = edge2 - glm::dot(edge0_norm, edge2) * edge0_norm;

    float cosang = glm::dot(pr_edge1, pr_edge2) /
        (glm::length(pr_edge1) * glm::length(pr_edge2));
    float sinang = glm::length(glm::cross(pr_edge1, pr_edge2)) /
        (glm::length(pr_edge1) * glm::length(pr_edge2));

    float angle = atan2(sinang, cosang);
        
    return angle;
} */
