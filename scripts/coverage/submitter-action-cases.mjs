const values = [null,'',' ','\t\n','./submit','../submit','/submit','?x=1','#fragment','//other.test/x',
  'https://EXAMPLE.test:443/a/../submit?x=1#part','http://[bad','http://','http://ex ample.test/',
  'javascript:alert(1)','data:text/plain,x','mailto:test@example.test','https://example.test/a\nb\tc',
  'https://example.test/\0','https://example.test/\ud800','http://[bad/\udc00','你好🦀'];
const bases=[null,'https://reference.test/a/','https://reference.test/a/?q#part','../','http://[bad',
  'javascript:void(0)','data:text/html,x'];
export const submitterActionCases=bases.flatMap(base=>values.flatMap(value=>['attribute','property'].map(kind=>({base,value,kind}))));

export function measureSubmitterActions(cases, tag) {
  return cases.map(({base,value,kind})=>{
    const owner=document.implementation.createHTMLDocument(),input=owner.createElement(tag);
    if(base!==null){const element=owner.createElement('base');element.setAttribute('href',base);owner.head.append(element);}
    owner.body.append(input);
    if(value!==null) {
      if(kind==='attribute')input.setAttribute('formaction',value);
      else input.formAction=value;
    }
    return {value:input.formAction,attribute:input.getAttribute('formaction')};
  });
}
