// Illustrative deterministic mocks, installed only when explicitly supplied as Plugin source.
// No shader execution, rendering, rasterization, audio processing, or device access.
function installGraphicsAudio({ global, host }) {
  class MockGraphics {
    constructor(width = 300, height = 150) {
      this.width = width;
      this.height = height;
      this.clearColor = [0, 0, 0, 0];
    }
    setClearColor(...rgba) {
      if (rgba.length !== 4 || !rgba.every(Number.isFinite))
        throw new TypeError('four finite components required');
      this.clearColor = rgba;
    }
    getParameter(name) {
      if (name === 'MAX_TEXTURE_SIZE') return 4096;
      if (name === 'RENDERER') return 'Explicit JavaScript mock';
      return null;
    }
    // An explicit asynchronous extension can use a real user-supplied backend.
    renderAsync(commands) {
      return host.call('render', {
        width: this.width,
        height: this.height,
        clearColor: this.clearColor,
        commands,
      });
    }
  }
  class MockAudioContext {
    constructor({ sampleRate = 48000 } = {}) {
      if (!Number.isFinite(sampleRate) || sampleRate <= 0)
        throw new TypeError('positive sample rate required');
      this.sampleRate = sampleRate;
      this.state = 'suspended';
      this.currentTime = 0;
    }
    async resume() {
      if (this.state === 'closed') throw new Error('closed');
      this.state = 'running';
    }
    async suspend() {
      if (this.state === 'closed') throw new Error('closed');
      this.state = 'suspended';
    }
    async close() {
      this.state = 'closed';
    }
    processAsync(samples) {
      return host.call('processAudio', { sampleRate: this.sampleRate, samples });
    }
  }
  // Deliberately distinct names: these are illustrative APIs, not browser-complete WebGL/WebGPU/WebAudio.
  Object.assign(global, { MockWebGL: MockGraphics, MockWebGPU: MockGraphics, MockAudioContext });
}
