extern crate nalgebra;

use nalgebra as na;
use na::sparse as nsp;

use std::collections::BTreeMap;

extern crate glm;

struct LaplacianEditingSystem {
    system_matrix: nsp::CsMatrix<f32, na::Dynamic, na::Dynamic>,
    rhs:           nsp::CsMatrix<f32, na::Dynamic, na::U1>
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

pub fn setup_system (points : &Vec<glm::Vec3>) {
    let n = points.len();
    // Now, since we are only dealing with a single spline, we assume that every node
    // is only connected to its neighbors in the point list.

    // Construct vector separating x- and y-coordinates
    // let mut point_vector  =
    // na::Matrix::<f32, na::Dynamic, na::U1, _> ::new_uninitialized(2 * n);

    let mut rows : Vec<usize> = Vec::with_capacity(2 * n);
    let mut cols : Vec<usize> = Vec::with_capacity(2 * n);
    let mut vals : Vec<f32> = Vec::with_capacity(2 * n);
    
    for i in 0..n {
	// point_vector[i] = points[i].x;
	// point_vector[i + n] = points[i].y;

	rows.push(i);
	cols.push(0);
	vals.push(points[i].x);

	rows.push(i + n);
	cols.push(0);
	vals.push(points[i].y);
    }

    let point_vector =
	na::CsMatrix::from_triplet(n * 2, 1,
				   &rows, &cols, &vals);
    
    let mut rows : Vec<usize> = Vec::with_capacity(3 * n);
    let mut cols : Vec<usize> = Vec::with_capacity(3 * n);
    let mut vals : Vec<f32>   = Vec::with_capacity(3 * n);

    let mut index_map : BTreeMap<(usize, usize), usize> = BTreeMap::new();


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

    let laplacian = nsp::CsMatrix::from_triplet(2 * n, 2 * n,
						&rows, &cols, &vals);

    let delta_vector = na::Matrix::from(&laplacian * &point_vector);

    println!("Delta vector: {:?}", delta_vector);
    
    // Redeclare these, to construct a new matrix
    let mut rows : Vec<usize> = Vec::with_capacity(2 * n);
    let mut cols : Vec<usize> = Vec::with_capacity(2 * n);
    let mut vals : Vec<f32>   = Vec::with_capacity(2 * n);

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
	let mut C = na::Matrix::<f32, na::Dynamic, na::U4, _>::zeros(2 * un);

	for j in 0..neighbors.len() {
	    C[(j, 0)] =   points[neighbors[j]].x;
	    C[(j, 1)] = - points[neighbors[j]].y;
	    C[(j, 2)] = 1.0;
	    C[(j, 3)] = 0.0;

	    C[(j + un, 0)] = points[neighbors[j]].y;
	    C[(j + un, 1)] = points[neighbors[j]].x;
	    C[(j + un, 2)] = 0.0;
	    C[(j + un, 3)] = 1.0;
	}

	let tmp1 = &C.transpose() * &C;
	
	let inn = tmp1.try_inverse();
	if inn == None {
	    println!("Matrix inversion failed");
	    println!("{:?}", tmp1);
	    ::std::process::exit(-1);
	}

	let M = &inn.unwrap() * &C.transpose();

	for neigh in neighbors {
	    rows.push(i);
	    cols.push(neigh)
	}
    }

    let t_tilde = nsp::CsMatrix::from_triplet(2 * n, 2 * n,
					      &rows, &cols, &vals);
    //let system = nsp::CsMatrix::from_triplet(3 * n, 3 * n,
    //					     rows, cols, vals);

    ::std::process::exit(0);
}
