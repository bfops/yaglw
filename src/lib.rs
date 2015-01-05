#![feature(associated_types)]
#![feature(default_type_params)]
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
