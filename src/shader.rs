use gl;
use gl::types::*;
use gl_context::GLContext;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::convert::AsRef;
use std::ffi::CString;
use std::iter::repeat;
use std::ptr;
use std::marker::PhantomData;
use std::str;

pub struct ProgramHandle<'a> {
  pub gl_id: GLuint,
  phantom: PhantomData<&'a ()>,
}

impl<'a> ProgramHandle<'a> {
  pub fn new<'b:'a>(_gl: &'a GLContext) -> ProgramHandle<'b> {
    let gl_id = unsafe {
      gl::CreateProgram()
    };

    assert!(gl_id != 0);

    ProgramHandle {
      gl_id: gl_id,
      phantom: PhantomData,
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
  phantom: PhantomData<&'a ()>,
}

impl<'a> ShaderHandle<'a> {
  pub fn compile_from<'b:'a>(
    _gl: &'a GLContext,
    shader_source: String,
    typ: GLenum
  ) -> ShaderHandle<'b> {
    let gl_id = unsafe {
      gl::CreateShader(typ)
    };

    assert!(gl_id != 0);

    // Attempt to compile the shader
    {
      let c_str = CString::new(shader_source.as_bytes()).unwrap();
      let ptr = c_str.as_ptr() as *const i8;
      unsafe {
        gl::ShaderSource(gl_id, 1, &ptr, ptr::null());
        gl::CompileShader(gl_id);
      }
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
      let mut buf: Vec<u8> = repeat(0).take(len as usize - 1).collect(); // subtract 1 to skip the trailing null character
      unsafe {
        gl::GetShaderInfoLog(gl_id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
      }
      let error_string =
        str::from_utf8(buf.as_ref())
          .unwrap_or_else(|_| panic!("ShaderInfoLog not valid utf8"));
      panic!("error compiling 0x{:x} shader: {}", typ, error_string);
    }

    ShaderHandle {
      gl_id: gl_id,
      phantom: PhantomData,
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
  pub fn new<'b:'a, T: Iterator<Item=(GLenum, String)>>(
    gl: &'a GLContext,
    shader_components: T,
  ) -> Shader<'b> {
    let handle = ProgramHandle::new(gl);

    let mut components = Vec::new();
    for (component, content) in shader_components {
      let s = ShaderHandle::compile_from(gl, content, component);
      unsafe {
        gl::AttachShader(handle.gl_id, s.gl_id);
      }
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
      let mut buf: Vec<u8> = repeat(0).take(len as usize - 1).collect(); // subtract 1 to skip the trailing null character
      unsafe {
        gl::GetProgramInfoLog(handle.gl_id, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
      }
      let error_string =
        str::from_utf8(buf.as_ref())
          .unwrap_or_else(|_| panic!("ProgramInfoLog not valid utf8"));
      panic!("{}", error_string);
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
    match self.uniforms.entry(s_name.clone()) {
      Entry::Occupied(entry) => *entry.get(),
      Entry::Vacant(entry) => {
        let c_name = CString::new(name.as_bytes()).unwrap();
        let ptr = c_name.as_ptr() as *const i8;
        let loc = unsafe {
          gl::GetUniformLocation(self.handle.gl_id, ptr)
        };
        assert!(loc != -1, "couldn't find shader uniform: {}", s_name);

        *entry.insert(loc)
      },
    }
  }
}
