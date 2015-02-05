#![feature(collections)]
#![feature(core)]
#![feature(libc)]
#![feature(std_misc)]
#![feature(unsafe_destructor)]

extern crate gl;
extern crate libc;
#[macro_use]
extern crate log;

pub mod gl_context;
pub mod shader;
pub mod texture;
pub mod vertex_buffer;
