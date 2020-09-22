WARNING: Fuwa is still a work in progress and is not ready for public use. 

A software renderer written in Rust.

Goals:
1. Full featured graphics rendering library matching functionality of something like OpenGL or Direct X, or WebGPU.
1. Performance suitable for real-time usage (at least as good as software rendering can be).
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

http://acta.uni-obuda.hu/Mileff_Nehez_Dudra_63.pdf
https://www.digipen.edu/sites/default/files/public/docs/theses/salem-haykal-digipen-master-of-science-in-computer-science-thesis-an-optimized-triangle-rasterizer.pdf


TODO:
1. Add geometric clipping algorithm for triangles.
1. Fix potential crashes when moving too close to objects (draw out of bounds)
1. Store/Upload data into a "device" (like WGPU) inside of Fuwa class?
1. Add model loader
1. Find a way to make FragmentBuffer handle multiple different shader types
1. Geometry Shader
1. Fill out "FuwaStats" struct for data logging/analysis
1. Add "Descriptors" like Vertex Descriptor, command queues, etc
1. Investigate pros/cons of fixed point
1. Continue SIMDifying Rasterizer
1. Look into better memory allocators like (bumpalo, toolshed, typed_arena)
1. Add special logic/early outs for special shaped triangle BBs
1. Try (smallvec, tinyvec, arrayvec) in raster function -- smallvec gave a performance loss??

SCENES TO MAKE:
1. Cube Textured
1. Cube Color Blend
1. Complex Scene
1. Depth Tester

BENCHMARKS TO MAKE:
1. Complex Scene
1. Many Small Triangles
1. Many Big triangles
1. Depth buffer strain