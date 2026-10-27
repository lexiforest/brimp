// Values are measured in installed Safari, including invalid and contextual CSS.
const values = [
  '', 'invalid', 'constructor', '__proto__', 'toString', 'red', 'RED', 'rebeccapurple', 'transparent', 'currentcolor',
  'inherit', 'initial', 'unset', 'revert', 'red blue', 'red;', ' red ', 'red\n',
  '/* comment */red/* end */', 'r\\65 d', '#abc', '#ABCD', '#abcdef', '#aabbccdd',
  '#12', '#12345', '#gggggg', 'abc', '123456', 'rgb(1 2 3',
  'rgb(25.5 50.5 75.5 / .5)', 'rgba(10,20,30,.4)', 'rgb(10%,20%,30%)',
  'rgb(10% 20% 30% / 40%)', 'rgb(-10 300 20)', 'rgb(1, 2 3)',
  'rgb(1 2 3 / .4 .5)', 'rgb(none 20 30 / none)', 'rgb(1 2 3 / var(--x))',
  'rgb(calc(1 + 2) 4 5)', 'rgb(calc(1 / 0) 20 30)', 'rgb(calc(0 / 0) 20 30)',
  'hsl(20 30% 40% / .5)', 'hsl(.5turn, 100%, 50%)', 'hsl(400grad 50% 50%)',
  'hwb(20 30% 40% / .5)', 'hwb(20 70% 70%)', 'hsl(none none 40%)',
  'lab(50% 30 40)', 'lab(46.2775% -47.5621 48.5837)', 'lch(60% 50 30)',
  'oklab(60% .1 .1)', 'oklch(60% .2 30)', 'oklch(60% .5 180)',
  'color(srgb .1 .2 .3 / .4)', 'color(srgb -1 2 .5)', 'color(srgb-linear .1 .2 .3)',
  'color(display-p3 1 .5 0)', 'color(display-p3-linear .5 .2 0)',
  'color(a98-rgb .3 .4 .5)', 'color(prophoto-rgb .3 .4 .5)', 'color(rec2020 .3 .4 .5)',
  'color(xyz .3 .4 .5)', 'color(xyz-d50 .3 .4 .5)', 'color(xyz-d65 .3 .4 .5)',
  'rgb(from red r g b / .5)', 'rgb(from #123456 calc(r + 10) g b)',
  'hsl(from red h s calc(l / 2))', 'color(from red display-p3 r g b)',
  'color-mix(in srgb, red, blue)', 'color-mix(in oklab, red 30%, blue)',
  'color-mix(in srgb, transparent, red)', 'color-mix(in srgb, currentcolor, red)',
  'color-mix(in hsl longer hue, red, blue)', 'color-mix(in srgb, red)',
  'color-mix(in srgb, red, blue, green)', 'light-dark(red,blue)',
  'color-mix(in rec2020, red, blue)', 'color-mix(in srgb, color(rec2020 .3 .4 .5), red)',
  'color(from color(rec2020 .3 .4 .5) srgb r g b)',
  'color(rec2020 0.01 0.08 0.09)', 'color(rec2020 -0.1 1.2 0.5)',
  'var(--x,red)', 'contrast-color(black)', 'contrast-color(white)', 'alpha(from red / .5)',
  'AccentColor', 'AccentColorText', 'ActiveText', 'ButtonBorder', 'ButtonFace', 'ButtonText',
  'Canvas', 'CanvasText', 'Field', 'FieldText', 'GrayText', 'Highlight', 'HighlightText',
  'LinkText', 'Mark', 'MarkText', 'SelectedItem', 'SelectedItemText', 'VisitedText',
  'color(display-p3 .123456789 .00000049 .00000051 / .123456789)',
  'color(display-p3 -1 2 .5 / .5)', 'color(display-p3 none .5 none / none)',
  'color(display-p3 1e-8 1e8 1e21)', 'color(display-p3 calc(infinity) 0 0)',
  'color(display-p3 calc(-infinity) 0 0)', 'color(display-p3 calc(NaN) .5 1)',
  'color(display-p3 .1 .2 .3 / calc(NaN))', 'color(display-p3 .1 .2 .3 / calc(infinity))',
  'color(display-p3 .1 .2 .3 / .999999999)',
  'hwb(from red h w b)', 'hsl(from #808080 h s l)',
  'rgb(from hsl(30 50% 50%) r g b)', 'color(from red srgb r g b)',
  'color-mix(in srgb, hsl(20 30% 40%), hwb(20 30% 40%))',
  'color(rec2020 -.09 1.2 .5)', 'color(rec2020 -.08 1.2 .5)',
];
for (const channel of [0, .1, .5, 1.5, 25.5, 50.5, 75.5, 127.5, 128.5, 254.5, 255, 256]) {
  values.push(`rgb(${channel} ${channel} ${channel} / .5)`);
}
for (const alpha of [0, .001, .1, .25, .5, .75, .999, 1, 2, -1]) {
  values.push(`color(srgb .1 .2 .3 / ${alpha})`);
}
export const inputColorCases = ['limited-srgb','display-p3'].flatMap(colorSpace=>
  [false,true].flatMap(alpha=>values.map(value=>({colorSpace,alpha,value}))));

export function measureInputColors(cases) {
  return cases.map(({alpha,value,colorSpace}) => {
    const make=()=>{const i=document.createElement('input');i.setAttribute('colorspace',colorSpace);if(alpha)i.setAttribute('alpha','');return i;};
    const assigned=make();assigned.type='color';assigned.value=value;
    const attribute=make();attribute.type='color';attribute.setAttribute('value',value);
    const switched=make();switched.value=value;switched.type='color';
    const form=document.createElement('form');form.appendChild(attribute);attribute.value='red';form.reset();
    const fragment=document.createElement('div');
    const escaped=value.replace(/&/g,'&amp;').replace(/"/g,'&quot;').replace(/</g,'&lt;');
    fragment.innerHTML='<input type="color" colorspace="'+colorSpace+'" '+(alpha?'alpha ':'')+'value="'+escaped+'">';
    return [assigned.value,attribute.value,switched.value,fragment.firstChild.value];
  });
}

export function measureColorTransitions() {
  const results=[];
  for(const writer of ['attribute','namespace','attrNode','attrValue','IDL'])for(const dirty of [false,true]) {
    const form=document.createElement('form'),i=document.createElement('input');form.appendChild(i);
    i.type='color';i.alpha=true;i.colorSpace='display-p3';i.defaultValue='color(display-p3 1 .5 0 / .4)';
    if(dirty)i.value='rgba(25.5,50.5,75.5,.5)';
    const values=[i.value];
    for(const [name,value] of [['colorspace','limited-srgb'],['colorspace','display-p3'],['colorspace','DISPLAY-P3'],
      ['alpha',null],['alpha',''],['value','#aabbccdd'],['colorspace','bad'],['colorspace',null],
      ['colorspace','display-p3'],['type','text'],['colorspace','limited-srgb'],['type','color']]) {
      if(writer==='IDL' && value!==null) {
        i[name==='colorspace'?'colorSpace':name==='value'?'defaultValue':name]=name==='alpha'?true:value;
      } else if(value===null) i.removeAttribute(name);
      else if(writer==='namespace')i.setAttributeNS(null,name,value);
      else if(writer==='attrNode'){const attr=document.createAttribute(name);attr.value=value;i.setAttributeNode(attr);}
      else if(writer==='attrValue'){if(!i.hasAttribute(name))i.setAttribute(name,'');i.getAttributeNode(name).value=value;}
      else i.setAttribute(name,value);
      values.push(i.value);
    }
    values.push(i.cloneNode().value,document.importNode(i).value);
    form.reset();values.push(i.value);i.colorSpace='display-p3';values.push(i.value);
    results.push({writer,dirty,values});
  }
  return results;
}
