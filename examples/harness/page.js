// A classic page bundle. The function keeps its state inside this realm.
globalThis.compute = (() => {
  document.body.innerHTML = '<button id="run">Run</button><output></output>';
  let calls = 0,
    clicks = 0;
  document.querySelector('#run').addEventListener('click', () => {
    clicks++;
  });
  return async function compute(input) {
    await new Promise((resolve) => setTimeout(resolve, 5));
    const response = await fetch('https://fixture.invalid/config');
    const { prefix } = await response.json();
    const text = prefix + input;
    document.querySelector('output').textContent = text;
    return { text, calls: ++calls, clicks, now: Date.now() };
  };
})();
null;
