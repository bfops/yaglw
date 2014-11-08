//! An ownership-semantics based handle to OpenGL. This prevents us from
//! accidentally modifying OpenGL state from multiple threads.
//!
//! GLW stands for "OpenGL wrapper".
#![feature(globs)]
#![feature(phase)]
#![feature(unsafe_destructor)]

extern crate gl;
extern crate libc;
#[phase(plugin, link)]
extern crate log;

pub mod gl_context;
pub mod shader;
pub mod texture;
pub mod vertex_buffer;
