use gl;
use gl::types::*;
use gl_context::GLContext;
use std::collections::HashMap;
use std::ptr;
use std::str;

pub struct Shader {
  pub id: GLuint,
  pub components: Vec<GLuint>,
  pub uniforms: HashMap<String, GLint>,
}

impl Shader {
  pub fn new<T: Iterator<(String, GLenum)>>(
      gl: &mut GLContext,
      shader_components: T,
  ) -> Shader {
    let mut shader_components = shader_components;
    let program = unsafe { gl::CreateProgram() };

    let mut components = Vec::new();
    for (content, component) in shader_components {
      let s = gl.compile_shader(content, component);
      unsafe { gl::AttachShader(program, s) };
      components.push(s);
    }

    unsafe {
      gl::LinkProgram(program);
    }

    // Get the link status
    let mut status = gl::FALSE as GLint;
    unsafe {
      gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);
    }

    // Fail on error
    if status != (gl::TRUE as GLint) {
        let mut len: GLint = 0;
        unsafe {
          gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
        }
        let mut buf = Vec::from_elem(len as uint - 1, 0u8); // subtract 1 to skip the trailing null character
        unsafe {
          gl::GetProgramInfoLog(program, len, ptr::null_mut(), buf.as_mut_ptr() as *mut GLchar);
        }
        panic!("{}", str::from_utf8(buf.as_slice()).expect("ProgramInfoLog not valid utf8"));
    }

    Shader {
      id: program,
      components: components,
      uniforms: HashMap::new(),
    }
  }

  pub fn with_uniform_location<T>(
    &mut self,
    gl: &mut GLContext,
    name: &'static str,
    f: |GLint| -> T,
  ) -> T {
    let s_name = String::from_str(name);
    let name = name.to_c_str().as_ptr();
    let t = match self.uniforms.get(&s_name) {
      None => {
        let (loc, t) = gl.use_shader(self, |_| {
          let loc = unsafe { gl::GetUniformLocation(self.id, name) };
          assert!(loc != -1, "couldn't find shader uniform: {}", s_name);

          (loc, f(loc))
        });

        self.uniforms.insert(s_name, loc);
        t
      },
      Some(&loc) => gl.use_shader(self, |_| f(loc)),
    };

    t
  }
}

impl Drop for Shader {
  fn drop(&mut self) {
    unsafe { gl::DeleteProgram(self.id); }
    for &s in self.components.iter() {
      unsafe { gl::DeleteShader(s); }
    }
  }
}
