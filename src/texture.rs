use gl;
use gl::types::*;
use gl_context::{GLContext, GLContextExistence};
use std::default::Default;
use std::kinds::marker::ContravariantLifetime;
use vertex_buffer::GLBuffer;

// TODO(cgaebel): Handle texture creation from an SDL surface.

#[deriving(Copy, Clone)]
pub struct TextureUnit {
  pub glsl_id: GLuint,
}

impl TextureUnit {
  // TODO: consider making this part of the struct to avoid recalculating;
  // is that fetch cheaper than the addition of a constant?
  pub fn gl_id(&self) -> GLuint {
    gl::TEXTURE0 + self.glsl_id
  }
}

impl Default for TextureUnit {
  fn default() -> TextureUnit {
    TextureUnit {
      glsl_id: 0,
    }
  }
}

impl Add<u32, TextureUnit> for TextureUnit {
  fn add(self, rhs: u32) -> TextureUnit {
    TextureUnit {
      glsl_id: self.glsl_id + rhs,
    }
  }
}

/// A GPU-allocated texture.
pub struct TextureHandle<'a> {
  pub gl_id: GLuint,
  pub lifetime: ContravariantLifetime<'a>,
}

impl<'a> TextureHandle<'a> {
  pub fn new(_gl: &'a GLContextExistence) -> TextureHandle<'a> {
    let mut handle = 0;
    unsafe {
      gl::GenTextures(1, &mut handle);
    }
    TextureHandle {
      gl_id: handle,
      lifetime: ContravariantLifetime,
    }
  }
}

#[unsafe_destructor]
impl<'a> Drop for TextureHandle<'a> {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteTextures(1, &self.gl_id);
    }
  }
}

/// A GPU-allocated texture.
pub struct Texture2D<'a> {
  pub handle: TextureHandle<'a>,
}

impl<'a> Texture2D<'a> {
  pub fn new(gl: &'a GLContextExistence) -> Texture2D<'a> {
    Texture2D {
      handle: TextureHandle::new(gl),
    }
  }
}

/// See the OpenGL docs on buffer textures.
pub struct BufferTexture<'a, T> {
  pub handle: TextureHandle<'a>,
  pub buffer: GLBuffer<'a, T>,
}

impl<'a, T> BufferTexture<'a, T> {
  pub fn new(
    gl: &'a GLContextExistence,
    gl_context: &mut GLContext,
    format: GLenum,
    capacity: uint,
  ) -> BufferTexture<'a, T> {
    // TODO: enforce that `format` matches T.

    let buffer = GLBuffer::new(gl, gl_context, capacity);
    let handle = TextureHandle::new(gl);

    unsafe {
      gl::BindTexture(gl::TEXTURE_BUFFER, handle.gl_id);
      gl::TexBuffer(gl::TEXTURE_BUFFER, format, buffer.byte_buffer.handle.gl_id);
    }

    BufferTexture {
      handle: handle,
      buffer: buffer,
    }
  }
}
