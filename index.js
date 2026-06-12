import init, { render_frame } from './pkg/cpu_image_js.js';

async function run() {
  await init();

  const configRes = await fetch('config.json');
  const config = await configRes.json();

  let imageWidth = config.size || 300;
  let imageHeight = config.size || 300;
  const defaultDuration = config.recording.duration;
  const defaultFps = config.recording.fps;
  const shaderIndex = config.shader || 1;

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
  canvas.width = imageWidth;
  canvas.height = imageHeight;

  const fpsEl = document.getElementById('fps');
  let frameCount = 0;
  let fpsTime = 0;
  let recording = false;
  let animId = null;

  function frame(now) {
    if (recording) return;
    if (fpsTime === 0) fpsTime = now;
    frameCount++;
    const elapsed = now - fpsTime;
    if (elapsed >= 500) {
      fpsEl.textContent = `FPS: ${Math.round(frameCount / elapsed * 1000)}`;
      frameCount = 0;
      fpsTime = now;
    }
    const pixels = render_frame(imageWidth, imageHeight, now / 1000, shaderIndex);
    const imageData = new ImageData(new Uint8ClampedArray(pixels.buffer, pixels.byteOffset, pixels.byteLength), imageWidth, imageHeight);
    ctx.putImageData(imageData, 0, 0);
    animId = requestAnimationFrame(frame);
  }

  animId = requestAnimationFrame(frame);

  const recBtn = document.getElementById('rec-btn');
  const recStatus = document.getElementById('rec-status');
  const recDuration = document.getElementById('rec-duration');
  const recFps = document.getElementById('rec-fps');

  recDuration.value = defaultDuration;
  recFps.value = defaultFps;

  const sizeSlider = document.getElementById('size-slider');
  const sizeLabel = document.getElementById('size-label');
  sizeSlider.value = imageWidth;
  sizeLabel.textContent = imageWidth;
  sizeSlider.addEventListener('input', () => {
    const s = parseInt(sizeSlider.value);
    sizeLabel.textContent = s;
    imageWidth = s;
    imageHeight = s;
    canvas.width = s;
    canvas.height = s;
  });

  recBtn.addEventListener('click', async () => {
    if (recording) return;
    recording = true;
    cancelAnimationFrame(animId);

    const durationSec = parseInt(recDuration.value) || defaultDuration;
    const fps = parseInt(recFps.value) || defaultFps;
    const totalFrames = durationSec * fps;

    recBtn.disabled = true;
    recBtn.textContent = 'Rendering...';
    recStatus.textContent = `0 / ${totalFrames}`;

    try {
      // Pre-render all frames to pixel arrays (without rendering to canvas for max speed)
      const frameData = [];
      for (let i = 0; i < totalFrames; i++) {
        const time = i / fps;
        const pixels = render_frame(imageWidth, imageHeight, time, shaderIndex);
        const imageData = new ImageData(new Uint8ClampedArray(pixels.buffer, pixels.byteOffset, pixels.byteLength), imageWidth, imageHeight);
        frameData.push(imageData.data.slice());
        recStatus.textContent = `${i + 1} / ${totalFrames}`;
        if (i % 5 === 0) await new Promise(r => setTimeout(r, 0));
      }

      recBtn.textContent = 'Recording...';
      recStatus.textContent = 'Recording...';

      // Set up capture stream and media recorder
      const stream = canvas.captureStream(fps);
      let mimeType = '';
      for (const mt of ['video/webm;codecs=vp9', 'video/webm;codecs=vp8', 'video/webm']) {
        if (MediaRecorder.isTypeSupported(mt)) { mimeType = mt; break; }
      }
      const recorder = new MediaRecorder(stream, mimeType ? { mimeType } : {});
      const chunks = [];
      recorder.ondataavailable = (e) => chunks.push(e.data);

      // Display first frame then start recording
      const first = new ImageData(frameData[0], imageWidth, imageHeight);
      ctx.putImageData(first, 0, 0);
      recorder.start();

      // Playback remaining frames at the target FPS using clock-based timing
      const startTime = performance.now();
      for (let i = 1; i < totalFrames; i++) {
        const targetTime = i * (1000 / fps);
        const elapsed = performance.now() - startTime;
        const delay = Math.max(0, targetTime - elapsed);
        await new Promise(r => setTimeout(r, delay));
        const imageData = new ImageData(frameData[i], imageWidth, imageHeight);
        ctx.putImageData(imageData, 0, 0);
        recStatus.textContent = `${i + 1} / ${totalFrames}`;
      }

      recBtn.textContent = 'Finalizing...';
      recStatus.textContent = 'Finalizing...';
      recorder.stop();

      await new Promise(resolve => {
        recorder.onstop = () => {
          const blob = new Blob(chunks, { type: 'video/webm' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `recording_${durationSec}s_${fps}fps_${imageWidth}x${imageHeight}.webm`;
          a.click();
          URL.revokeObjectURL(url);
          recStatus.textContent = 'Done!';
          recBtn.textContent = 'Record';
          recBtn.disabled = false;
          recording = false;
          animId = requestAnimationFrame(frame);
          resolve();
        };
      });
    } catch (err) {
      console.error(err);
      recStatus.textContent = 'Error!';
      recBtn.textContent = 'Record';
      recBtn.disabled = false;
      recording = false;
      animId = requestAnimationFrame(frame);
    }
  });
}

run();
