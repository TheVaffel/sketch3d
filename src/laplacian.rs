
extern crate nalgebra;

use nalgebra as na;
use na::sparse as nsp;

use std::collections::BTreeMap;

extern crate glm;
extern crate generic_array;

static FIXED_POINT_FACTOR: f32 = 10.0;

pub struct LaplacianEditingSystem {
    pub original_matrix_trans: nsp::CsMatrix<f32, na::Dynamic>,
    pub system_composition: nsp::CsCholesky<f32, na::Dynamic>,
    rhs:           na::MatrixMN::<f32, na::Dynamic, na::U1>,
    original_points : na::MatrixMN::<f32, na::Dynamic, na::U1>,
    fixed:         Vec<usize>,
}

impl LaplacianEditingSystem {
    pub fn solve(&mut self,
		 new_positions : &mut Vec<glm::Vec3>) {
	
 	let l = self.system_composition.l().unwrap();

	let n = l.ncols() / 2;
	for i in 0..self.fixed.len() {
	    self.rhs[2 * n + i] = FIXED_POINT_FACTOR * new_positions[self.fixed[i]].x;
	    self.rhs[2 * n + self.fixed.len() + i] = FIXED_POINT_FACTOR * new_positions[self.fixed[i]].y;
	    // self.rhs[2 * n + i] = FIXED_POINT_FACTOR * self.original_points[self.fixed[i]];
	    // self.rhs[2 * n + self.fixed.len() + i] = FIXED_POINT_FACTOR * self.original_points[self.fixed[i] + n];
	}
	
	let rhs = na::Matrix::from(&self.original_matrix_trans * &na::CsMatrix::from(self.rhs.clone()));
	let a = l.solve_lower_triangular(&rhs);
	let a = l.tr_solve_lower_triangular(&a.unwrap());
	
	if a == None {
	    println!("Could not solve system for b = {:?}",
		     rhs);
	    std::process::exit(-1);
	}

	let a = a.unwrap();
	
	let n = a.len() / 2;
	for i in 0..n {
	    new_positions[i].x = a[i];
	    new_positions[i].y = a[i + n];
	}
    }

    pub fn setup_fixed_points(&mut self,
			      fixed: Vec<usize>) {
	let n = self.original_points.len() / 2;
	
	let mut rows : Vec<usize> = Vec::with_capacity(3 * n);
	let mut cols : Vec<usize> = Vec::with_capacity(3 * n);
	let mut vals : Vec<f32>   = Vec::with_capacity(3 * n);

	let mut index_map : BTreeMap<(usize, usize), usize> = BTreeMap::new();

	
	// Now, since we are only dealing with a single spline, we assume that every node
	// is only connected to its neighbors in the point list.
	
	// Create the Laplacian so that it has separate elements for x- and y-components of input points
	// (In other words, we will multiply the Laplacian by a vector representing 2D points such that
	// it first holds all x-components of the points, then the y-components)    
	for i in 0..n {
	    if i > 0 {
		let ci = i - 1;

		let val = if i == n - 1 { - 1.0 } else { - 0.5 };
		
		insert_triplet(&mut rows, &mut cols, &mut vals,
			       i, ci, val,
			       &mut index_map);

		insert_triplet(&mut rows, &mut cols, &mut vals,
			       i + n, ci + n, val,
			       &mut index_map);
	    }

	    if i < n - 1 {

		let ci = i + 1;
		
		let val = if i == 0 { -1.0 } else { -0.5 };
		
		insert_triplet(&mut rows, &mut cols, &mut vals,
			       i, ci, val,
			       &mut index_map);
		insert_triplet(&mut rows, &mut cols, &mut vals,
			       i + n, ci + n, val,
			       &mut index_map);
	    }

	    insert_triplet(&mut rows, &mut cols, &mut vals,
			   i, i, 1.0,
			   &mut index_map);

	    
	    insert_triplet(&mut rows, &mut cols, &mut vals,
			   i + n, i + n, 1.0,
			   &mut index_map);
	}

	let laplacian = nsp::CsMatrix::from_triplet(2 * n + 2 * fixed.len(), 2 * n,
						    &rows, &cols, &vals);

	let delta_vector = na::Matrix::from(&laplacian * &na::CsMatrix::from(self.original_points.clone()));

	// Redeclare these, to construct a new matrix
	rows.clear();
	cols.clear();
	vals.clear();
	index_map.clear();
	

	#[allow(non_snake_case)]
	for i in 0..n {
	    let neighbors =
		if i == 0 {
		    vec![i, i + 1]
		} else if i == n - 1 {
		    vec![i - 1, i]
		} else {
		    vec![i - 1, i, i + 1]
		};

	    let un = neighbors.len();
	    let mut C = na::Matrix::<f32, na::Dynamic, na::U2, _>::zeros(2 * un);

	    let point_vector = &self.original_points;
	    for j in 0..neighbors.len() {
		C[(j, 0)] = point_vector[neighbors[j]];
		C[(j, 1)] = point_vector[neighbors[j] + n];

		C[(j + un, 0)] = point_vector[neighbors[j] + n];
		C[(j + un, 1)] = point_vector[neighbors[j]];
	    }

	    let tmp1 = &C.transpose() * &C;
	    
	    let inn = tmp1.try_inverse();
	    if inn == None {
		println!("Matrix inversion failed");
		::std::process::exit(-1);
	    }

	    let M = &inn.unwrap() * &C.transpose();

	    for j in 0..neighbors.len() {
		let neigh = neighbors[j];
		
		insert_triplet(&mut rows, &mut cols, &mut vals,
			       i, neigh,
			       delta_vector[i] * M[(0, j)] - delta_vector[i + n] * M[(1, j)],
			       &mut index_map);
		insert_triplet(&mut rows, &mut cols, &mut vals,
			       i, neigh + n,
			       delta_vector[i] * M[(0, j + un)] - delta_vector[i + n] * M[(1, j + un)], 
			       &mut index_map);

		insert_triplet(&mut rows, &mut cols, &mut vals,
			       i + n, neigh,
			       delta_vector[i + n] * M[(0, j)] + delta_vector[i] * M[(1, j)], 
			       &mut index_map);
		insert_triplet(&mut rows, &mut cols, &mut vals,
			       i + n, neigh + n,
			       delta_vector[i + n] * M[(0, j + un)] + delta_vector[i] * M[(1, j + un)], 
			       &mut index_map); 
	    }
	}

	let rhs_vector = na::Matrix::<f32, na::Dynamic, na::U1, _>::zeros(2 * n + 2 * fixed.len());

	// We multiply the fixed points and their weights by a factor, so that they
	// will prefer standing still even more
	for i in 0..fixed.len() {

	    insert_triplet(&mut rows, &mut cols, &mut vals,
			   2 * n + i, fixed[i], FIXED_POINT_FACTOR * 1.0, &mut index_map);
	    insert_triplet(&mut rows, &mut cols, &mut vals,
			   2 * n + fixed.len() + i, n + fixed[i], FIXED_POINT_FACTOR * 1.0,
			   &mut index_map);
	}
	
	let t_tilde = nsp::CsMatrix::from_triplet(2 * n + 2 * fixed.len(), 2 * n,
						  &rows, &cols, &vals);
	
	let system = &t_tilde + &(laplacian * (-1.0));

	// Now, we would like to compute a least square approximation of x in 
	// system * x = rhs_vector
	// Sadly, I don't think nalgebra has that feature, so we will just do it in the poor man's way

	let ssh = system.transpose();
	let system2 = &ssh * &system;

	self.original_matrix_trans = ssh;
	self.system_composition = nsp::CsCholesky::new(&system2);
	self.rhs = rhs_vector;
	self.fixed = fixed;
    }

    pub fn empty() -> LaplacianEditingSystem {
	unsafe {
	    LaplacianEditingSystem {
		original_matrix_trans : nsp::CsMatrix::<f32, na::Dynamic>::new_uninitialized_generic(na::Dynamic::new(1), na::Dynamic::new(1), 0),
		system_composition : nsp::CsCholesky::new_symbolic(&nsp::CsMatrix::<f32, na::Dynamic, na::Dynamic>::new_uninitialized_generic(na::Dynamic::new(1), na::Dynamic::new(1), 0)),
		rhs:            na::MatrixMN::<f32, na::Dynamic, na::U1>::new_uninitialized_generic(na::Dynamic::new(1), na::U1),
		original_points: na::MatrixMN::<f32, na::Dynamic, na::U1>::new_uninitialized_generic(na::Dynamic::new(1), na::U1),
		fixed:          Vec::new(),
	    }
	}
    }
}


// Here's a fun story:
// Actual mature sparse linear algebra libraries (like builtin in Julia
// or Eigen for C++), you may specify each row-col index multiple times in the
// triple array, and the matrix constructor will sum the values together.
// Haha, not in nalgebra. So that's why I bring up my own function for
// insertion into the triple array...
fn insert_triplet(rows: &mut Vec<usize>,
		  cols: &mut Vec<usize>,
		  vals: &mut Vec<f32>,
		  row : usize,
		  col : usize,
		  val : f32,
		  map: &mut BTreeMap<(usize, usize), usize>) {
    let pp = (row, col);
    if !map.contains_key(&pp) {
	map.insert(pp, rows.len());
	rows.push(row);
	cols.push(col);
	vals.push(val);
    } else {
	let p_val = map.get(&pp).unwrap();
	vals[*p_val] += val;
    }
}

pub fn setup_original_points(points : &Vec<glm::Vec3>) -> LaplacianEditingSystem {
    
    let n = points.len();
    let mut point_vector = na::MatrixMN::<f32, na::Dynamic, na::U1>::zeros(n * 2);
    for i in 0..n {
	point_vector[i] = points[i].x;
	point_vector[i + n] = points[i].y;
    }

    let mut system = LaplacianEditingSystem::empty();
    system.original_points = point_vector;

    system
}

pub fn setup_system (points : &Vec<glm::Vec3>,
		     fixed : Vec<usize>)
		     -> LaplacianEditingSystem {
    
    // Construct vector separating x- and y-coordinates

    let mut final_system = setup_original_points(points);
    
    final_system.setup_fixed_points(fixed);

    final_system
}
