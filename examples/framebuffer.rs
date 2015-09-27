extern crate gl;
extern crate sdl2;
extern crate yaglw;

use gl::types::*;
use sdl2::EventPump;
use sdl2::event::Event;
use std::mem;

use yaglw::framebuffer::Framebuffer;
use yaglw::gl_context::GLContext;
use yaglw::shader::Shader;
use yaglw::texture::Texture2D;
use yaglw::vertex_buffer::{ArrayHandle, GLArray, GLBuffer, GLType, VertexAttribData, DrawMode};

const WINDOW_WIDTH: i32 = 800;
const WINDOW_HEIGHT: i32 = 600;

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

const DEFERRED_VERTEX_SHADER: &'static str = "
  #version 330

  void main() {
    if (gl_VertexID == 0) {
      gl_Position = vec4(1, -1, 0, 1);
    } else if (gl_VertexID == 1) {
      gl_Position = vec4(1, 1, 0, 1);
    } else if (gl_VertexID == 2) {
      gl_Position = vec4(-1, -1, 0, 1);
    } else if (gl_VertexID == 3) {
      gl_Position = vec4(-1, 1, 0, 1);
    } else {
      gl_Position = vec4(0, 0, 0, 1);
    }
  }
";

const DEFERRED_FRAGMENT_SHADER: &'static str = "
  #version 330

  uniform sampler2D colors;

  layout (location = 0) out vec4 frag_color;

  void main() {
    vec2 tex_pos = vec2(gl_FragCoord.x / 800, gl_FragCoord.y / 600);
    vec4 color = texture(colors, tex_pos);
    frag_color = color;
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

  let shader = Shader::new(&gl, components.iter().map(|&(ty, s)| (ty, String::from(s))));
  shader.use_shader(&mut gl);

  let components = [
    ((gl::VERTEX_SHADER, DEFERRED_VERTEX_SHADER)),
    ((gl::FRAGMENT_SHADER, DEFERRED_FRAGMENT_SHADER)),
  ];

  let mut deferred_shader = Shader::new(&gl, components.iter().map(|&(ty, s)| (ty, String::from(s))));
  deferred_shader.use_shader(&mut gl);

  let vao =
    GLArray::new(
      &mut gl,
      &shader,
      &attribs,
      DrawMode::Triangles,
      vbo,
    );
  vao.bind(&mut gl);

  let empty_vao = ArrayHandle::new(&gl);

  match gl.get_error() {
    gl::NO_ERROR => {},
    err => {
      println!("OpenGL error 0x{:x} in setup 1", err);
      return;
    },
  }

  let mut fbo = Framebuffer::new(&gl);
  let colors = Texture2D::new(&gl);

  unsafe {
    gl::ActiveTexture(gl::TEXTURE0);
    gl::BindTexture(gl::TEXTURE_2D, colors.handle.gl_id);
    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA32F as i32, WINDOW_WIDTH, WINDOW_HEIGHT, 0, gl::RGB, gl::FLOAT, std::ptr::null());

    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
  }

  fbo.bind(&mut gl);
  fbo.attach_2d(&gl, gl::COLOR_ATTACHMENT0, &colors);

  match gl.get_error() {
    gl::NO_ERROR => {},
    err => {
      println!("OpenGL error 0x{:x} in setup 2", err);
      return;
    },
  }

  let color_uniform = deferred_shader.get_uniform_location("colors");
  deferred_shader.use_shader(&mut gl);
  unsafe {
    gl::Uniform1i(color_uniform, 0);
  }

  match gl.get_error() {
    gl::NO_ERROR => {},
    err => {
      println!("OpenGL error 0x{:x} in setup", err);
      return;
    },
  }

  while !quit_event(&mut event_pump) {
    fbo.bind(&mut gl);
    shader.use_shader(&mut gl);

    gl.clear_buffer();
    vao.bind(&mut gl);
    vao.draw(&mut gl);

    unsafe {
      gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
      gl::BindTexture(gl::TEXTURE_2D, colors.handle.gl_id);
    }

    deferred_shader.use_shader(&mut gl);

    unsafe {
      gl::BindVertexArray(empty_vao.gl_id);
      gl::DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
    }

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
      WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32,
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
