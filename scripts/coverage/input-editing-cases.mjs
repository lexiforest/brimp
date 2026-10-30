import { inputValidityFlagNames } from './input-validity-cases.mjs';

export const inputEditingCases = ['text', 'search', 'tel', 'url', 'email', 'password'].flatMap(
  (type) =>
    ['ab', 'abcdef', '💡a', 'a@b', 'a@b,c@d', ' ab ', ' a@b ', 'a  b', ' 💡 '].map((text) => ({
      type,
      text,
    })),
);
for (const text of [
  'a',
  '-',
  '+',
  '.',
  'e',
  '1e',
  '1e-',
  '1.',
  '1e2',
  '1e309',
  '12a3',
  ' 12',
  '1,2',
  '--1',
  '-1.2',
  '+1',
  '+1.',
  '+1e2',
  '01',
  '01.',
  '0.',
  '-.5',
  '1.e2',
  '1E+02',
  '1e-999',
  '１２.３',
  '−2',
  '１ｅ２',
  '1.2.3',
  'ee1',
  '1--2',
])
  inputEditingCases.push({ type: 'number', text });

// Use actual keyboard input, not synthetic input events or script value writes.
// Safari uses WebDriver Element Send Keys; Brimp uses its native typeText API.
export async function measureInputEditing(page, cases) {
  const results = [];
  for (const { type, text } of cases) {
    await page.evaluate(`(() => {
      document.body.innerHTML='<form><input id="edit" minlength="3" required></form>';
      document.getElementById('edit').type=${JSON.stringify(type)};
      return null;
    })()`);
    await page.typeText('#edit', text);
    results.push(
      JSON.parse(
        await page.evaluate(`JSON.stringify((() => {
      const input=document.getElementById('edit'),flags=${JSON.stringify(inputValidityFlagNames)};
      const snapshot=control=>[control.value,flags.map(name=>control.validity[name])];
      const states=[snapshot(input)];
      input.setAttribute('maxlength','2');states.push(snapshot(input));
      states.push(snapshot(input.cloneNode()));
      input.value=input.value;states.push(snapshot(input));
      input.form.reset();states.push(snapshot(input));
      return states;
    })())`),
      ),
    );
  }
  return results;
}

export const inputEditingMutations = {
  none: '',
  readonly: 'i.readOnly=true',
  disabled: 'i.disabled=true',
  typeSame: 'i.type=i.type',
  typeText: 'i.type="text"',
  typeSearch: 'i.type="search"',
  typeNumber: 'i.type="number"',
  typeRoundtrip: 'const type=i.type;i.type="hidden";i.type=type',
  defaultValue: 'i.defaultValue="new"',
  rangeText: 'i.setRangeText("c",0,1)',
  adopt: 'document.implementation.createHTMLDocument().adoptNode(i)',
  remove: 'i.remove()',
  step: 'i.stepUp()',
  numeric: 'i.valueAsNumber=2',
};
export const inputEditingMutationCases = ['text', 'email', 'number'].flatMap((type) =>
  Object.keys(inputEditingMutations).map((action) => ({ type, action })),
);
export async function measureInputEditingMutations(page, cases) {
  const results = [];
  for (const { type, action } of cases) {
    await page.evaluate(
      `(()=>{document.body.innerHTML='<form><input id="edit" minlength="3" required></form>';document.getElementById('edit').type=${JSON.stringify(type)};return null})()`,
    );
    await page.typeText('#edit', type === 'number' ? '1e' : 'ab');
    results.push(
      await page.evaluate(
        `(()=>{const i=document.getElementById('edit');try{${inputEditingMutations[action]}}catch(e){return e.name}return [i.value,i.validity.tooShort,i.validity.tooLong,i.validity.badInput]})()`,
      ),
    );
  }
  return results;
}

export const inputEditingContinuationCases = ['text', 'url', 'email', 'number'].flatMap((type) =>
  (type === 'number' ? ['1', '1.', '+1', '1e', '1e2'] : ['ab', ' ab ']).flatMap((text) =>
    ['same', 'new'].map((assign) => ({ type, text, assign })),
  ),
);
export async function measureInputEditingContinuation(page, cases) {
  const results = [];
  for (const { type, text, assign } of cases) {
    await page.evaluate(
      `(()=>{document.body.innerHTML='<input id="edit">';document.getElementById('edit').type=${JSON.stringify(type)};return null})()`,
    );
    await page.typeText('#edit', text);
    await page.evaluate(
      `(()=>{const i=document.getElementById('edit');i.value=${assign === 'same' ? 'i.value' : JSON.stringify(type === 'number' ? '77' : 'xyz')};return null})()`,
    );
    await page.typeText('#edit', '2');
    results.push(await page.evaluate("document.getElementById('edit').value"));
  }
  return results;
}
