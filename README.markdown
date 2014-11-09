**What is this?**

`glw` is a safe, low-level, low-overhead OpenGL wrapper in Rust.
It is *not* intended to abstract anything for those who aren't familiar with OpenGL, but just to provide a performant, safer layer over what [gl-rs](https://github.com/bjz/gl-rs) provides.

Please open an issue and/or send a pull request if you notice something's wrong!
Performance issues definitely count as a problem!

**Why use it?**

There exist quite a few libraries that wrap OpenGL functionality up in low-level abstractions. `glw` prioritizes having low-to-no runtime overhead over having nice abstractions or simple code.
