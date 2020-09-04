WARNING: Fuwa is still a work in progress and is not ready for public use. 

A software renderer written in Rust.

Goals:
1. Full featured graphics rendering library matching functionality of something like OpenGL or Direct X, or WebGPU.
1. Optimal performance suitable for real-time usage (at least as good as software rendering can be).
1. A wealth of examples and documentation to provide a solid foundation in computer graphics.

A note on safety:
Fuwa uses unsafe Rust. In fact, due to the severe performance requirements of Fuwa as a renderer, it may not be possible to achieve the high performance required for real-time rendering without it. However, special attention will be given to ensure that any exposed functionality will be completely safe, and users of the crate will not need to understand the ins and outs of safe vs unsafe Rust in order to use it.

Possiblities:
1. Integration as a rendering back-end for other crates like wgpu or gfx-hal.
1. A software fallback for WebGPU for desktop and web platforms which don't support it.
1. Support for standard shaders like GLSL or SPIR-V.

Resources:

https://github.com/ssloy/tinyrenderer/wiki

https://www.youtube.com/watch?v=uehGqieEbus&list=PLqCJpWy5Fohe8ucwhksiv9hTF5sfid8lA

https://www.gabrielgambetta.com/computer-graphics-from-scratch/introduction.html

https://fgiesen.wordpress.com/2013/02/17/optimizing-sw-occlusion-culling-index/

TODO:
1. See Rasterizer.rs and refactor the draw functions to make it easier to draw different kinds of things (flat shaded, vertex color, texture etc)
1. Add texture / rendering
1. Merge "Textures" resource and "Resource Generator" into a "Repository" or some other kind of grouping?
1. Add "Descriptors" like Vertex Descriptor, command queues, etc
1. Split folder structure into separate modules