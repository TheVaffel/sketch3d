
// This file is largely following the tutorial at http://nercury.github.io/rust/opengl/tutorial/2018/02/10/opengl-in-rust-from-scratch-03-compiling-shaders.html


extern crate gl;
extern crate glfw;

use std::ffi::{CString, CStr};


struct Shader {
    id: gl::types::GLuint,
}

impl Shader {
    fn from_source(
	source: &CStr,
	kind: gl::types::GLenum
    ) -> Result<Shader, String> {
	let id = shader_from_source(source, kind)?;
	Ok(Shader {id})
    }


    fn from_vert_source(source: &CStr) -> Result<Shader, String> {
	Shader::from_source(source, gl::VERTEX_SHADER)
    }

    fn from_frag_source(source: &CStr) -> Result<Shader, String> {
	Shader::from_source(source, gl::FRAGMENT_SHADER)
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
	unsafe {
	    gl::DeleteShader(self.id);
	}
    }
}

fn shader_from_source(source: &CStr,
		      kind: gl::types::GLuint) -> Result<gl::types::GLuint, String> {
    let id = unsafe { gl::CreateShader(kind) };

    unsafe {
	gl::ShaderSource(id, 1, &source.as_ptr(), std::ptr::null());
	gl::CompileShader(id);
    }

    let mut success: gl::types::GLint = 1;

    unsafe {
	gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
    }

    if success == 0 {
	let mut len: gl::types::GLint = 0;
	unsafe {
	    gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);
	}
	
	let error = create_whitespace_cstring_with_len(len as usize);
	
	unsafe {
	    gl::GetShaderInfoLog(
		id,
		len,
		std::ptr::null_mut(),
		error.as_ptr() as *mut gl::types::GLchar
	    );
	}

	return Err(error.to_string_lossy().into_owned());

	
	
    }

    Ok(id)
}

fn create_whitespace_cstring_with_len(len: usize) -> CString {

    let mut buffer: Vec<u8> = Vec::with_capacity(len + 1);

    buffer.extend([b' '].iter().cycle().take(len));

    unsafe { CString::from_vec_unchecked(buffer) }
}



pub struct ShaderProgram {
    pub id: gl::types::GLuint,
}

impl ShaderProgram {
    pub fn activate(&self) {
	unsafe {
	    gl::UseProgram(self.id);
	}
    }
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
	unsafe {
	    gl::DeleteProgram(self.id);
	}
    }
}

pub fn create_simple_shader(vert_source: &CString,
			    frag_source: &CString) -> Result<ShaderProgram, String> {
    let vert_shader = Shader::from_vert_source(vert_source).unwrap();
    let frag_shader = Shader::from_frag_source(frag_source).unwrap();

    let program_id = unsafe { gl::CreateProgram() };
    
    unsafe {
	gl::AttachShader(program_id, vert_shader.id);
	gl::AttachShader(program_id, frag_shader.id);
	gl::LinkProgram(program_id);
	gl::DetachShader(program_id, vert_shader.id);
	gl::DetachShader(program_id, frag_shader.id);
    }

    let mut success: gl::types::GLint = 1;

    unsafe {
	gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);
    }

    if success == 0 {
	let mut len: gl::types::GLint = 0;
	unsafe {
	    gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut len);
	}

	let error = create_whitespace_cstring_with_len(len as usize);

	unsafe {
	    gl::GetProgramInfoLog(
		program_id,
		len,
		std::ptr::null_mut(),
		error.as_ptr() as *mut gl::types::GLchar
	    );
	}

	return Err(error.to_string_lossy().into_owned());
    }

    
    let program = ShaderProgram { id: program_id };

    Ok(program)
}
