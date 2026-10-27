// Independent Safari corpus for the autocomplete IDL token grammar.
const fields=('name honorific-prefix given-name additional-name family-name honorific-suffix nickname '+
  'username new-password current-password one-time-code organization-title organization street-address '+
  'address-line1 address-line2 address-line3 address-level4 address-level3 address-level2 address-level1 '+
  'country country-name postal-code cc-name cc-given-name cc-additional-name cc-family-name cc-number '+
  'cc-exp cc-exp-month cc-exp-year cc-csc cc-type transaction-currency transaction-amount language '+
  'bday bday-day bday-month bday-year sex url photo email impp tel tel-country-code tel-national '+
  'tel-area-code tel-local tel-local-prefix tel-local-suffix tel-extension '+
  'device-eid device-imei device-imei1 device-imei2 device-nal').split(' ');
const values=['','on','off',' ON ','OFF','webauthn','email email','on on','off webauthn',
  'username webauthn','webauthn username','section- email','section-A SECTION-A email',
  'shipping SHIPPING email','section-A shipping home email webauthn','section-K name',
  'section-İ name','section-💡 name','email\u00a0','email\u000b','\tSECTION-X\nSHIPPING\fFAX\rTEL ',
  'shipping billing email','home work email','section- section-b name','unknown','password',
  'webidentity','email webidentity','constructor','__proto__','email webauthn webauthn',
  ...fields.flatMap(field=>[field.toUpperCase(),`section-A billing ${field}`,`home ${field}`,`section-A shipping work ${field} webauthn`])];
export const inputAutocompleteCases=['text','hidden','checkbox','month'].flatMap(type=>values.map(value=>({type,value})));
export function measureAutocomplete(cases) {
  return cases.map(({type,value})=>{
    const i=document.createElement('input');i.type=type;i.autocomplete=value;
    const results=[i.autocomplete,i.getAttribute('autocomplete')];
    i.removeAttribute('autocomplete');i.setAttributeNS(null,'autocomplete',value);results.push(i.autocomplete);
    const form=document.createElement('form');form.autocomplete='off';form.appendChild(i);results.push(i.autocomplete);
    i.type='hidden';results.push(i.autocomplete);i.type=type;results.push(i.autocomplete);
    return results;
  });
}
