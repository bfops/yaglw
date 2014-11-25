use gl;
use gl::types::*;
use std::raw;
use std::mem;
use std::str;

unsafe fn from_c_str<'a>(s: *const u8) -> &'a str {
  let mut len = 0;
  {
    let mut s_shift = s;
    while *s_shift as char != '\0' {
      s_shift = s_shift.offset(1);
      len += 1;
    }
  }

  let as_slice: raw::Slice<u8> =
    raw::Slice {
      data: s,
      len: len,
    };

  str::from_utf8_unchecked(mem::transmute(as_slice))
}

/// A handle to an OpenGL context. Only create one of these per thread.
#[deriving(Send)]
pub struct GLContextExistence;

pub struct GLContext;

// TODO(bfops): Safely create GLContext from existing ones, e.g. sdl2::video::GLContext.
impl GLContext {
  pub unsafe fn new() -> (GLContextExistence, GLContext) {
    // TODO(cgaebel): Have a thread-local variable checking whether or not
    // there is only one GLContext, and fail if there's more than one.
    (GLContextExistence, GLContext)
  }

  /// Stops the processing of any triangles hidden from view when rendering.
  pub fn enable_culling(&mut self) {
    unsafe {
      gl::FrontFace(gl::CCW);
      gl::CullFace(gl::BACK);
      gl::Enable(gl::CULL_FACE);
    }
  }

  #[allow(missing_docs)]
  pub fn enable_alpha_blending(&mut self) {
    unsafe {
      gl::Enable(gl::BLEND);
      gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }
  }

  #[allow(missing_docs)]
  pub fn enable_smooth_lines(&mut self) {
    unsafe {
      gl::Enable(gl::LINE_SMOOTH);
      gl::LineWidth(2.5);
    }
  }

  /// Allows us to use the OpenGL depth buffer, which makes OpenGL do logical
  /// things when two things are rendered at the same x and y coordinates, but
  /// different z coordinates.
  pub fn enable_depth_buffer(&mut self, depth: GLclampd) {
    unsafe {
      gl::Enable(gl::DEPTH_TEST);
      gl::DepthFunc(gl::LESS);
      gl::ClearDepth(depth);
    }
  }

  /// At the beginning of each frame, OpenGL clears the buffer. This sets the
  /// color the buffer is cleared to.
  pub fn set_background_color(&mut self, r: GLfloat, g: GLfloat, b: GLfloat, a: GLfloat) {
    unsafe {
      gl::ClearColor(r, g, b, a);
    }
  }

  /// Replace the current OpenGL buffer with all pixels of the
  /// "background color", as set with `set_background_color`.
  pub fn clear_buffer(&mut self) {
    unsafe {
      gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
    }
  }

  /// Prints opengl version information.
  pub fn print_stats(&self) {
    unsafe {
      let opengl_version = gl::GetString(gl::VERSION);
      let glsl_version = gl::GetString(gl::SHADING_LANGUAGE_VERSION);
      info!(
        "OpenGL version: {}", 
        from_c_str(opengl_version),
      );
      info!(
        "GLSL version: {}",
        from_c_str(glsl_version),
      );
    }
  }

  pub fn get_error(&self) -> GLuint {
    unsafe {
      gl::GetError()
    }
  }
}
