import init, { render_frame } from './pkg/cpu_image_js.js';

async function run() {
  await init();

  // auto-reload on rebuild
  (async () => {
    let last = 0;
    while (true) {
      const r = await fetch('/build-ts');
      const { ts } = await r.json();
      if (ts > last && last !== 0) location.reload();
      last = ts;
      await new Promise(r => setTimeout(r, 1000));
    }
  })();

  const canvas = document.getElementById('canvas');
  const ctx = canvas.getContext('2d');
  const width = 640;
  const height = 480;
  canvas.width = width;
  canvas.height = height;

  const fpsEl = document.getElementById('fps');
  let lastFrame = 0;
  let frameCount = 0;
  let fpsTime = 0;

  function frame(now) {
    frameCount++;
    if (now - fpsTime >= 1000) {
      fpsEl.textContent = `FPS: ${frameCount}`;
      frameCount = 0;
      fpsTime = now;
    }

    const pixels = render_frame(width, height, now / 1000);
    const imageData = new ImageData(new Uint8ClampedArray(pixels.buffer), width, height);
    ctx.putImageData(imageData, 0, 0);
    requestAnimationFrame(frame);
  }

  requestAnimationFrame(frame);
}

run();
