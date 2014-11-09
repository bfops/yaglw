use gl;
use gl::types::*;
use gl_context::GLContext;
use std::collections::HashMap;
use std::ptr;
use std::str;

pub struct ProgramHandle<'a> {
  pub gl_id: GLuint,
}

impl<'a> ProgramHandle<'a> {
  pub fn new<'b: 'a>(_gl: &'b GLContext) -> ProgramHandle<'a> {
    let gl_id = unsafe {
      gl::CreateProgram()
    };

    ProgramHandle {
      gl_id: gl_id,
    }
  }
}

#[unsafe_destructor]
impl<'a> Drop for ProgramHandle<'a> {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteProgram(self.gl_id);
    }
  }
}

pub struct ShaderHandle<'a> {
  pub gl_id: GLuint,
}

impl<'a> ShaderHandle<'a> {
  pub fn compile_from<'b: 'a>(
    _gl: &'b GLContext,
    shader_source: String,
    typ: GLenum
  ) -> ShaderHandle<'a> {
    let gl_id = unsafe {
      gl::CreateShader(typ)
    };
    // Attempt to compile the shader
    shader_source.with_c_str(|ptr| unsafe { gl::ShaderSource(gl_id, 1, &ptr, ptr::null()) });
    unsafe {
      gl::CompileShader(gl_id);
    }

    // Get the compile status
    let mut status = gl::FALSE as GLint;
    unsafe {
      gl::GetShaderiv(gl_id, gl::COMPILE_STATUS, &mut status);
    }

    // Fail on error
    if status != (gl::TRUE as GLint) {
      let mut len = 0;
      unsafe {
        gl::GetShaderiv(gl_id, gl::INFO_LOG_LENGTH, &mut len);
      }
      let mut buf = Vec::from_elem(len as uint - 1, 0u8); // subtract 1 to skip the trailing null character
      unsafe {
        gl::GetShaderInfoLog(gl_id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
      }
      panic!("error compiling 0x{:x} shader: {}", typ, str::from_utf8(buf.as_slice()).expect("ShaderInfoLog not valid utf8"));
    }

    ShaderHandle {
      gl_id: gl_id,
    }
  }
}

#[unsafe_destructor]
impl<'a> Drop for ShaderHandle<'a> {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteShader(self.gl_id);
    }
  }
}

pub struct Shader<'a> {
  pub handle: ProgramHandle<'a>,
  pub components: Vec<ShaderHandle<'a>>,
  pub uniforms: HashMap<String, GLint>,
}

impl<'a> Shader<'a> {
  pub fn new<'b: 'a, T: Iterator<(String, GLenum)>>(
    gl: &'b GLContext,
    shader_components: T,
  ) -> Shader<'a> {
    let mut shader_components = shader_components;
    let handle = ProgramHandle::new(gl);

    let mut components = Vec::new();
    for (content, component) in shader_components {
      let s = ShaderHandle::compile_from(gl, content, component);
      unsafe { gl::AttachShader(handle.gl_id, s.gl_id) };
      components.push(s);
    }

    unsafe {
      gl::LinkProgram(handle.gl_id);
    }

    // Get the link status
    let mut status = gl::FALSE as GLint;
    unsafe {
      gl::GetProgramiv(handle.gl_id, gl::LINK_STATUS, &mut status);
    }

    // Fail on error
    if status != (gl::TRUE as GLint) {
      let mut len: GLint = 0;
      unsafe {
        gl::GetProgramiv(handle.gl_id, gl::INFO_LOG_LENGTH, &mut len);
      }
      let mut buf = Vec::from_elem(len as uint - 1, 0u8); // subtract 1 to skip the trailing null character
      unsafe {
        gl::GetProgramInfoLog(handle.gl_id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
      }
      panic!("{}", str::from_utf8(buf.as_slice()).expect("ProgramInfoLog not valid utf8"));
    }

    Shader {
      handle: handle,
      components: components,
      uniforms: HashMap::new(),
    }
  }

  pub fn use_shader(&self, _gl: &mut GLContext) {
    unsafe {
      gl::UseProgram(self.handle.gl_id)
    }
  }

  pub fn get_uniform_location(
    &mut self,
    name: &'static str,
  ) -> GLint {
    let s_name = String::from_str(name);
    let name = name.to_c_str().as_ptr();
    match self.uniforms.get(&s_name) {
      None => {
        let loc = unsafe { gl::GetUniformLocation(self.handle.gl_id, name) };
        assert!(loc != -1, "couldn't find shader uniform: {}", s_name);

        self.uniforms.insert(s_name, loc);
        loc
      },
      Some(&loc) => loc,
    }
  }
}
