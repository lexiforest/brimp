// The IDL autocomplete value describes accepted tokens; no stored credentials,
// autofill UI or credential requests are supplied by this parser.
const __autofillNormalFields = new Set(
  (
    'name honorific-prefix given-name additional-name family-name honorific-suffix nickname ' +
    'username new-password current-password one-time-code organization-title organization ' +
    'street-address address-line1 address-line2 address-line3 address-level4 address-level3 ' +
    'address-level2 address-level1 country country-name postal-code cc-name cc-given-name ' +
    'cc-additional-name cc-family-name cc-number cc-exp cc-exp-month cc-exp-year cc-csc cc-type ' +
    'transaction-currency transaction-amount language bday bday-day bday-month bday-year sex url photo ' +
    'device-eid device-imei device-imei1 device-imei2 device-nal'
  ).split(' '),
);
const __autofillContactFields = new Set(
  (
    'email impp tel tel-country-code tel-national tel-area-code ' +
    'tel-local tel-local-prefix tel-local-suffix tel-extension'
  ).split(' '),
);
function __inputAutocomplete(input) {
  const tokens =
    __asciiLowercase(__getContentAttribute(input, 'autocomplete') ?? '').match(/[^\t\n\f\r ]+/g) ??
    [];
  if (!tokens.length) return '';
  if (tokens.length === 1 && (tokens[0] === 'on' || tokens[0] === 'off')) {
    return __inputType(input) === 'hidden' ? '' : tokens[0];
  }
  if (tokens.length === 1 && tokens[0] === 'webauthn') return 'webauthn';
  let index = 0;
  if (tokens[index]?.startsWith('section-')) index++;
  if (['shipping', 'billing'].includes(tokens[index])) index++;
  const contact = ['home', 'work', 'mobile', 'fax', 'pager'].includes(tokens[index]);
  if (contact) index++;
  const field = tokens[index++];
  if (!__autofillContactFields.has(field) && (contact || !__autofillNormalFields.has(field)))
    return '';
  if (tokens[index] === 'webauthn') index++;
  return index === tokens.length ? tokens.join(' ') : '';
}
