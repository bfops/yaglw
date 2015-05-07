[![Build Status](https://travis-ci.org/bfops/yaglw.svg?branch=master)](https://travis-ci.org/bfops/yaglw)

**What is this?**

`yaglw` (Yet Another openGL Wrapper) is a safe, low-level, low-overhead OpenGL wrapper in Rust.
It's still very young and very incomplete, and code is still being moved over from [playform](https://github.com/bfops/playform/).
It does **not** handle context creation; the `GLContext::new()` function is unsafe because it relies on the programmer
to have properly initialized the OpenGL context before calling it.

There are a bunch of libraries that wrap OpenGL functionality in Rusty abstractions (both high- and low-level).
The goal of `yaglw` is to maintain a set of safe, low-level, zero-overhead abstractions over [gl-rs](https://github.com/bjz/gl-rs) for users writing performant OpenGL code.
It's mainly meant to factor out redundant code (like creating a buffer texture) in a safe, performant way, not to abstract it or take responsibility away from the user.

Please open an issue and/or send a pull request if you notice something's wrong or missing!
Performance issues definitely count as a problem!
