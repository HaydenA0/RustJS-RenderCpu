# CPU Image JS

A simple demo of rendering CPU-generated shaders using WebAssembly.

This project was created while I was learning about GPU architectures. I became
curious about implementing similar rendering techniques directly on the CPU to
see how they would perform. Because I have limited experience with web-native
development, I chose to write the rendering engine in Rust and compile it to
WebAssembly. I then used an LLM to assist with the HTML canvas integration and
the JavaScript glue code. The final result is a basic demo capable of rendering
and recording CPU-run shaders.

# Results

The following benchmarks were recorded on an **11th Gen Intel® Core™ i5-11400H @ 2.70GHz**:

### Blackhole
<video src="./videos/blackhole_3s_30fps_1920x1080.webm" controls width="100%"></video>

### Fireworks
<video src="./videos/fireworks_10s_30fps_1024x768.webm" controls width="100%"></video>

### Sphere
<video src="./videos/sphere_10s_30fps_1920x1080.webm" controls width="100%"></video>

*Shader formulas adapted from:* [Xor](https://x.com/XorDev)

---

As expected, CPU-bound real-time rendering is highly resource-intensive, making
high-performance playback difficult to achieve on standard hardware.
Nonetheless, this project serves as a valuable proof of concept and was highly
educational to build. 

The primary objective was to gain hands-on experience with WebAssembly, which
was successfully achieved alongside generating these visual demos. While I do
not have immediate plans for major updates, I will keep this repository as a
reference for shader prototyping. In the future, I may revisit the code to
experiment with further performance optimizations.
