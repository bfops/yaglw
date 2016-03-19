extern crate gl;
extern crate sdl2;
extern crate yaglw;

use gl::types::*;
use sdl2::EventPump;
use sdl2::event::Event;
use std::mem;

use yaglw::gl_context::GLContext;
use yaglw::shader::Shader;
use yaglw::vertex_buffer::{GLArray, GLBuffer, GLType, VertexAttribData, DrawMode};

#[repr(C)]
#[derive(Copy, Clone)]
struct Vertex {
  pub position: [GLfloat; 2],
  pub color: [GLfloat; 3],
}

const VERTEX_SHADER: &'static str = "
  #version 330

  in vec2 position;
  in vec3 color;

  out vec3 v_color;

  void main() {
    v_color = color;
    gl_Position = vec4(position, 0, 1);
  }
";

const FRAGMENT_SHADER: &'static str = "
  #version 330

  in vec3 v_color;

  layout (location = 0) out vec4 frag_color;

  void main() {
    frag_color = vec4(v_color, 1.0);
  }
";

pub fn main() {
  let sdl = sdl2::init().unwrap();

  let _event = sdl.event().unwrap();
  let video = sdl.video().unwrap();

  let window = make_window(&video);
  let mut event_pump = sdl.event_pump().unwrap();

  let _sdl_gl_context = window.gl_create_context().unwrap();

  // Load the OpenGL function pointers.
  gl::load_with(|s| unsafe {
    mem::transmute(video.gl_get_proc_address(s))
  });

  let mut gl = unsafe {
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

  let mut vbo = GLBuffer::new(&mut gl, 3);
  vbo.push(&mut gl, &vertices);

  let attribs = [
    VertexAttribData {
      name: "position",
      size: 2,
      unit: GLType::Float,
      divisor: 0,
    },
    VertexAttribData {
      name: "color",
      size: 3,
      unit: GLType::Float,
      divisor: 0,
    },
  ];

  let components = [
    ((gl::VERTEX_SHADER, VERTEX_SHADER)),
    ((gl::FRAGMENT_SHADER, FRAGMENT_SHADER)),
  ];

  let shader = Shader::new(&gl, components.iter().map(|&(ty, s)| (ty, String::from(s))));
  shader.use_shader(&mut gl);

  let vao =
    GLArray::new(
      &mut gl,
      &shader,
      &attribs,
      DrawMode::Triangles,
      vbo,
    );
  vao.bind(&mut gl);

  match gl.get_error() {
    gl::NO_ERROR => {},
    err => {
      println!("OpenGL error 0x{:x} in setup", err);
      return;
    },
  }

  while !quit_event(&mut event_pump) {
    gl.clear_buffer();
    vao.draw(&mut gl);
    // swap buffers
    window.gl_swap_window();

    std::thread::sleep_ms(10);
  }
}

fn make_window(video: &sdl2::VideoSubsystem) -> sdl2::video::Window {
  let gl_attr = video.gl_attr();
  gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
  gl_attr.set_context_version(3, 3);

  // Open the window as fullscreen at the current resolution.
  let mut window =
    video.window(
      "Triangle",
      800, 600,
    );

  let window = window.position(0, 0);
  window.opengl();

  window.build().unwrap()
}

fn quit_event(event_pump: &mut EventPump) -> bool {
  loop {
    match event_pump.poll_event() {
      None => {
        return false;
      },
      Some(Event::Quit{..}) => {
        return true;
      }
      Some(Event::AppTerminating{..}) => {
        return true;
      }
      _ => {},
    }
  }
}
