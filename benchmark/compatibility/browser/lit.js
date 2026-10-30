import { LitElement, html } from 'lit';
import { check } from './check.js';

// Test driver component; Lit itself is the unchanged upstream implementation.
class CorpusCounter extends LitElement {
  static properties = { count: { type: Number } };
  constructor() {
    super();
    this.count = 0;
  }
  render() {
    return html`<button
      @click=${() => {
        this.count++;
      }}
    >
      Count: ${this.count}
    </button>`;
  }
}
customElements.define('corpus-counter', CorpusCounter);
globalThis.runWorkload = async () => {
  const element = document.createElement('corpus-counter');
  document.body.append(element);
  await element.updateComplete;
  const initial = element.shadowRoot.querySelector('button').textContent;
  element.shadowRoot.querySelector('button').click();
  await element.updateComplete;
  const clicked = element.shadowRoot.querySelector('button').textContent;
  element.count = 7;
  await element.updateComplete;
  const updated = element.shadowRoot.querySelector('button').textContent;
  check(
    initial === 'Count: 0' && clicked === 'Count: 1' && updated === 'Count: 7',
    'Lit must render and react to events/property changes',
  );
  return { initial, clicked, updated, count: element.count };
};
