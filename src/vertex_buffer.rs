use gl;
use gl::types::*;
use gl_context::*;
use shader::*;
use std::cell::RefCell;
use std::mem;
use std::ptr;
use std::rc::Rc;

/// Gets the id number for a given input of the shader program.
#[allow(non_snake_case)]
pub fn glGetAttribLocation(shader_program: GLuint, name: &str) -> GLint {
  name.with_c_str(|ptr| unsafe { gl::GetAttribLocation(shader_program, ptr) })
}

pub struct BufferHandle<'a> {
  pub gl_id: GLuint,
}

impl<'a> BufferHandle<'a> {
  pub fn new<'b: 'a>(_gl: &'b GLContext) -> BufferHandle<'a> {
    let mut gl_id = 0;

    unsafe {
      gl::GenBuffers(1, &mut gl_id);
    }

    assert!(gl_id != 0);

    BufferHandle {
      gl_id: gl_id,
    }
  }
}

#[unsafe_destructor]
impl<'a> Drop for BufferHandle<'a> {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteBuffers(1, &self.gl_id);
    }
  }
}

/// Fixed-size VRAM buffer for individual bytes.
pub struct GLByteBuffer<'a> {
  pub handle: BufferHandle<'a>,
  /// number of bytes in the buffer.
  pub length: uint,
  /// maximum number of bytes in the buffer.
  pub capacity: uint,
}

impl<'a> GLByteBuffer<'a> {
  /// Creates a new array of objects on the GPU.
  /// capacity is provided in units of size slice_span.
  pub fn new<'b: 'a>(gl: &'b GLContext, capacity: uint) -> GLByteBuffer<'a> {
    let handle = BufferHandle::new(gl);

    unsafe {
      gl::BindBuffer(gl::ARRAY_BUFFER, handle.gl_id);

      gl::BufferData(
        gl::ARRAY_BUFFER,
        capacity as GLsizeiptr,
        ptr::null(),
        gl::DYNAMIC_DRAW,
      );
    }

    match gl.get_error() {
      gl::NO_ERROR => {},
      gl::OUT_OF_MEMORY => panic!("Out of VRAM"),
      err => panic!("OpenGL error 0x{:x}", err),
    }

    GLByteBuffer {
      handle: handle,
      length: 0,
      capacity: capacity,
    }
  }

  /// Add more data into this buffer.
  pub unsafe fn push(&mut self, vs: *const u8, count: uint) {
    assert!(
      self.length + count <= self.capacity,
      "GLByteBuffer::push {} into a {}/{} full GLByteBuffer",
      count,
      self.length,
      self.capacity
    );

    self.update_inner(self.length, vs, count);
    self.length += count;
  }

  pub fn swap_remove(&mut self, i: uint, count: uint) {
    assert!(count <= self.length);
    self.length -= count;
    assert!(i <= self.length);

    // In the `i == self.length` case, we don't bother with the swap;
    // decreasing `self.length` is enough.

    if i < self.length {
      assert!(
        i <= self.length - count,
        "GLByteBuffer::swap_remove would cause copy in overlapping regions"
      );

      unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, self.handle.gl_id);

        gl::CopyBufferSubData(
          gl::ARRAY_BUFFER,
          gl::ARRAY_BUFFER,
          self.length as i64,
          i as i64,
          count as i64,
        );
      }
    }
  }

  pub unsafe fn update(&self, idx: uint, vs: *const u8, count: uint) {
    assert!(idx + count <= self.length);
    self.update_inner(idx, vs, count);
  }

  unsafe fn update_inner(&self, idx: uint, vs: *const u8, count: uint) {
    assert!(idx + count <= self.capacity);

    gl::BindBuffer(gl::ARRAY_BUFFER, self.handle.gl_id);

    gl::BufferSubData(
      gl::ARRAY_BUFFER,
      idx as i64,
      count as i64,
      mem::transmute(vs)
    );
  }
}

/// Fixed-size typed VRAM buffer, optimized for bulk inserts.
pub struct GLBuffer<'a, T> {
  pub byte_buffer: GLByteBuffer<'a>,
  pub length: uint,
}

impl<'a, T> GLBuffer<'a, T> {
  pub fn new<'b: 'a>(gl: &'b GLContext, capacity: uint) -> GLBuffer<'a, T> {
    GLBuffer {
      byte_buffer: GLByteBuffer::new(gl, capacity * mem::size_of::<T>()),
      length: 0,
    }
  }

  pub fn push(&mut self, vs: &[T]) {
    unsafe {
      self.byte_buffer.push(
        mem::transmute(vs.as_ptr()),
        mem::size_of::<T>() * vs.len()
      );
    }
    self.length += vs.len();
  }

  pub fn update(&mut self, idx: uint, vs: &[T]) {
    unsafe {
      self.byte_buffer.update(
        mem::size_of::<T>() * idx,
        mem::transmute(vs.as_ptr()),
        mem::size_of::<T>() * vs.len(),
      );
    }
  }

  pub fn swap_remove(&mut self, idx: uint, count: uint) {
    self.byte_buffer.swap_remove(
      mem::size_of::<T>() * idx,
      mem::size_of::<T>() * count,
    );
    self.length -= count;
  }
}

pub enum DrawMode {
  Lines,
  Triangles,
  Points,
}

impl DrawMode {
  fn to_enum(&self) -> GLenum {
    match self {
      &Lines     => gl::LINES,
      &Triangles => gl::TRIANGLES,
      &Points    => gl::POINTS,
    }
  }
}

#[deriving(Show)]
pub enum GLType {
  Float,
  UInt,
  Int,
}

impl GLType {
  pub fn size(&self) -> uint {
    match *self {
      Float => mem::size_of::<GLfloat>(),
      UInt => mem::size_of::<GLuint>(),
      Int => mem::size_of::<GLint>(),
    }
  }

  pub fn gl_enum(&self) -> GLenum {
    match *self {
      Float => gl::FLOAT,
      UInt => gl::UNSIGNED_INT,
      Int => gl::INT,
    }
  }

  pub fn is_integral(&self) -> bool {
    match *self {
      Float => false,
      UInt => true,
      Int => true,
    }
  }
}

#[deriving(Show)]
/// Specifies how to pass data from OpenGL to the vertex shaders.
pub struct VertexAttribData<'a> {
  /// Cooresponds to the shader's `input variable`.
  pub name: &'a str,
  /// The size of this attribute, in the provided units.
  pub size: uint,
  pub unit: GLType,
}

pub struct ArrayHandle<'a> {
  pub gl_id: GLuint,
}

impl<'a> ArrayHandle<'a> {
  pub fn new<'b: 'a>(_gl: &'b GLContext) -> ArrayHandle<'a> {
    let mut gl_id = 0;
    unsafe {
      gl::GenVertexArrays(1, &mut gl_id);
    }

    ArrayHandle {
      gl_id: gl_id,
    }
  }
}

#[unsafe_destructor]
impl<'a> Drop for ArrayHandle<'a> {
  fn drop(&mut self) {
    unsafe {
      gl::DeleteVertexArrays(1, &self.gl_id);
    }
  }
}

/// A fixed-capacity array of bytes passed to OpenGL.
pub struct GLArray<'a, T> {
  pub buffer: GLBuffer<'a, T>,
  pub handle: ArrayHandle<'a>,
  /// How to draw this buffer. Ex: gl::LINES, gl::TRIANGLES, etc.
  pub mode: GLenum,
  /// size of T in vertices
  pub attrib_span: uint,
  /// length in vertices
  pub length: uint,
}

impl<'a, T> GLArray<'a, T> {
  /// Creates a new array of objects on the GPU.
  /// capacity is provided in units of size slice_span.
  pub fn new<'b: 'a>(
    gl: &'b GLContext,
    shader_program: Rc<RefCell<Shader>>,
    attribs: &[VertexAttribData],
    mode: DrawMode,
    buffer: GLBuffer<'a, T>,
  ) -> GLArray<'a, T> {
    let handle = ArrayHandle::new(gl);

    unsafe {
      gl::BindVertexArray(handle.gl_id);
    }

    let mut offset = 0;
    let attrib_span = {
      let mut attrib_span = 0;
      for attrib in attribs.iter() {
        attrib_span += attrib.size * attrib.unit.size();
      }
      attrib_span
    };
    for attrib in attribs.iter() {
      let shader_attrib =
        glGetAttribLocation(
          shader_program.borrow().handle.gl_id,
          attrib.name
        );
      assert!(shader_attrib != -1, "shader attribute \"{}\" not found", attrib.name);
      let shader_attrib = shader_attrib as GLuint;

      unsafe {
        gl::EnableVertexAttribArray(shader_attrib);

        if attrib.unit.is_integral() {
          gl::VertexAttribIPointer(
            shader_attrib,
            attrib.size as i32,
            attrib.unit.gl_enum(),
            attrib_span as i32,
            ptr::null().offset(offset),
          );
        } else {
          gl::VertexAttribPointer(
            shader_attrib,
            attrib.size as i32,
            attrib.unit.gl_enum(),
            gl::FALSE as GLboolean,
            attrib_span as i32,
            ptr::null().offset(offset),
          );
        }
      }
      offset += (attrib.size * attrib.unit.size()) as int;
    }

    match unsafe { gl::GetError() } {
      gl::NO_ERROR => {},
      err => panic!("OpenGL error 0x{:x}", err),
    }

    assert!(mem::size_of::<T>() % attrib_span == 0);

    GLArray {
      buffer: buffer,
      handle: handle,
      mode: mode.to_enum(),
      attrib_span: mem::size_of::<T>() / attrib_span,
      length: 0,
    }
  }

  pub fn push(&mut self, vs: &[T]) {
    self.buffer.push(vs);
    self.length += vs.len() * self.attrib_span;
  }

  pub fn swap_remove(&mut self, idx: uint, count: uint) {
    self.buffer.swap_remove(idx, count);
    self.length -= count * self.attrib_span;
  }

  /// Draws all the queued triangles to the screen.
  pub fn draw(&self, gl: &GLContext) {
    self.draw_slice(gl, 0, self.buffer.length);
  }

  /// Draw some subset of the triangle array.
  pub fn draw_slice(&self, _gl: &GLContext, start: uint, len: uint) {
    assert!(start + len <= self.length);

    unsafe {
      gl::BindVertexArray(self.handle.gl_id);
      gl::BindBuffer(gl::ARRAY_BUFFER, self.buffer.byte_buffer.handle.gl_id);

      gl::DrawArrays(self.mode, (start * self.attrib_span) as i32, (len * self.attrib_span) as i32);
    }
  }
}
