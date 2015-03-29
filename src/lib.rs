#![deny(warnings)]

#![feature(collections)]
#![feature(convert)]
#![feature(core)]
#![feature(libc)]
#![feature(unsafe_destructor)]

extern crate gl;
extern crate libc;
#[macro_use]
extern crate log;

pub mod gl_context;
pub mod shader;
pub mod texture;
pub mod vertex_buffer;
