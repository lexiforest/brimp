// Shared Safari reference for the native attribute UTF-16 boundary.
export const inputStringFields = [
  'accept',
  'align',
  'alt',
  'dirName',
  'name',
  'pattern',
  'placeholder',
  'useMap',
  'formTarget',
  'autocomplete',
];
export function measureInputStringCodeUnits(fields) {
  const units = (value) => Array.from({ length: value.length }, (_, i) => value.charCodeAt(i));
  return fields.flatMap((name) =>
    [0xd800, 0xdc00].map((unit) => {
      const i = document.createElement('input');
      const value =
        name === 'autocomplete'
          ? 'section-' + String.fromCharCode(unit) + ' email'
          : String.fromCharCode(unit);
      i[name] = value;
      return {
        name,
        unit,
        property: units(i[name]),
        attribute: units(i.getAttribute(name.toLowerCase())),
      };
    }),
  );
}
