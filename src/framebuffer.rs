use gl;
use gl::types::*;
use gl_context::GLContext;
use std::marker::PhantomData;

use texture::Texture2D;

pub struct Framebuffer<'a> {
  pub gl_id: GLuint,
  pub phantom: PhantomData<&'a ()>,
}

impl<'a> Framebuffer<'a> {
  pub fn new<'b:'a>(_gl: &'a GLContext) -> Framebuffer<'b> {
    let mut gl_id = 0;
    unsafe {
      gl::GenFramebuffers(1, &mut gl_id);
    }

    Framebuffer {
      gl_id: gl_id,
      phantom: PhantomData,
    }
  }

  pub fn bind(&mut self, _gl: &mut GLContext) {
    unsafe {
      gl::BindFramebuffer(gl::FRAMEBUFFER, self.gl_id);
    }
  }

  pub fn attach_2d(&mut self, _gl: &GLContext, attachment: GLenum, tex: &Texture2D) {
    unsafe {
      gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachment, gl::TEXTURE_2D, tex.handle.gl_id, 0);
    }
  }
}

impl<'a> Drop for Framebuffer<'a> {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteFramebuffers(1, &self.gl_id);
    }
  }
}
