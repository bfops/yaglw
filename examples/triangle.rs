extern crate gl;
extern crate sdl2;
extern crate yaglw;

use gl::types::*;
use sdl2::event::Event;
use std::mem;
use std::time::duration::Duration;
use std::old_io::timer;

use yaglw::gl_context::GLContext;
use yaglw::shader::Shader;
use yaglw::vertex_buffer::{GLArray, GLBuffer, GLType, VertexAttribData, DrawMode};

#[repr(C)]
#[derive(Copy)]
struct Vertex {
  pub position: [GLfloat; 2],
  pub color: [GLfloat; 3],
}

const VERTEX_SHADER: &'static str = "
  #version 130

  in vec2 position;
  in vec3 color;

  out vec3 v_color;

  void main() {
    v_color = color;
    gl_Position = vec4(position, 0, 1);
  }
";

const FRAGMENT_SHADER: &'static str = "
  #version 130

  in vec3 v_color;

  void main() {
    gl_FragColor = vec4(v_color, 1.0);
  }
";

pub fn main() {
  let window = make_window();

  let _sdl_gl_context = window.gl_create_context().unwrap();

  // Load the OpenGL function pointers.
  gl::load_with(|s| unsafe {
    mem::transmute(sdl2::video::gl_get_proc_address(s))
  });

  let (gl, mut gl_context) = unsafe {
    GLContext::new()
  };

  let vertices = [
    Vertex {
      position: [0.0, 0.5],
      color: [1.0, 0.0, 0.0],
    },
    Vertex {
      position: [-0.5, -0.5],
      color: [0.0, 1.0, 0.0],
    },
    Vertex {
      position: [0.5, -0.5],
      color: [0.0, 0.0, 1.0],
    },
  ];

  let mut vbo = GLBuffer::new(&gl, &mut gl_context, 3);
  vbo.push(&mut gl_context, &vertices);

  let attribs = [
    VertexAttribData {
      name: "position",
      size: 2,
      unit: GLType::Float,
    },
    VertexAttribData {
      name: "color",
      size: 3,
      unit: GLType::Float,
    },
  ];

  let components = [
    ((gl::VERTEX_SHADER, VERTEX_SHADER)),
    ((gl::FRAGMENT_SHADER, FRAGMENT_SHADER)),
  ];

  let shader = Shader::new(&gl, components.iter().map(|&(ty, s)| (ty, String::from_str(s))));
  shader.use_shader(&mut gl_context);

  let vao =
    GLArray::new(
      &gl,
      &mut gl_context,
      &shader,
      &attribs,
      DrawMode::Triangles,
      vbo,
    );
  vao.bind(&mut gl_context);

  match gl_context.get_error() {
    gl::NO_ERROR => {},
    err => {
      println!("OpenGL error 0x{:x} in setup", err);
      return;
    },
  }

  while !quit_event() {
    gl_context.clear_buffer();
    vao.draw(&mut gl_context);
    // swap buffers
    window.gl_swap_window();

    timer::sleep(Duration::milliseconds(10));
  }
}

fn make_window() -> sdl2::video::Window {
  sdl2::init(sdl2::INIT_EVERYTHING);

  sdl2::video::gl_set_attribute(sdl2::video::GLAttr::GLContextMajorVersion, 3);
  sdl2::video::gl_set_attribute(sdl2::video::GLAttr::GLContextMinorVersion, 0);
  sdl2::video::gl_set_attribute(
    sdl2::video::GLAttr::GLContextProfileMask,
    sdl2::video::GLProfile::GLCoreProfile as i32,
  );

  sdl2::video::Window::new(
    "Triangle",
    sdl2::video::WindowPos::PosCentered,
    sdl2::video::WindowPos::PosCentered,
    800,
    600,
    sdl2::video::OPENGL,
  ).unwrap()
}

fn quit_event() -> bool {
  loop {
    match sdl2::event::poll_event() {
      Event::None => {
        return false;
      },
      Event::Quit(_) => {
        return true;
      }
      Event::AppTerminating(_) => {
        return true;
      }
      _ => {},
    }
  }
}
