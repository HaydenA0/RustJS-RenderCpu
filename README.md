# CPU Image JS

A simple demo of rendering CPU-generated shaders using WebAssembly.

This project was created while I was learning about GPU architectures. I became
curious about implementing similar rendering techniques directly on the CPU to
see how they would perform. Because I have limited experience with web-native
development, I chose to write the rendering engine in Rust and compile it to
WebAssembly. I then used an LLM to assist with the HTML canvas integration and
the JavaScript glue code. The final result is a basic demo capable of rendering
and recording CPU-run shaders.

# How to Use

**Prerequisites:** [Rust](https://rustup.rs/) and [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/).

```bash
# Build the Rust → WASM package
make build

# Start the dev server (auto-rebuilds on src/ changes)
make run
```

Open `http://localhost:8000` in a browser. Use the slider to adjust resolution, set duration/FPS, and click **Record** to render and download a `.webm` video.

Switch shaders by editing the `"shader"` field in `config.json` (values 1–4 map to `src/frag1.rs`–`src/frag4.rs`).

# Results

The following benchmarks were recorded on an **11th Gen Intel® Core™ i5-11400H @ 2.70GHz**:

### Blackhole
[blackhole_3s_30fps_1920x1080.webm](https://github.com/user-attachments/assets/991bf08c-e247-491b-bed2-21d61eebeb29)

### Fireworks
[fireworks_10s_30fps_1024x768.webm](https://github.com/user-attachments/assets/8b71dc7a-ff78-468f-aee8-105b79316921)

### Sphere
[sphere_10s_30fps_1920x1080.webm](https://github.com/user-attachments/assets/af543e3c-6d5f-40ab-832e-9c2d2e888de9)

### Box
[recording_10s_30fps_712x712.webm](https://github.com/user-attachments/assets/ba07bf85-51e8-49c6-abd6-9e212ecd8f5d)


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
