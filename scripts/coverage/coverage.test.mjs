import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {inventory} from './inventory.mjs';
import {probe} from './probe.mjs';
import {tracking, initialProgress, preserveProgress, summarizeProgress, validateProgress} from './progress.mjs';
import {reuseSafariReference} from './safari-reference.mjs';

const root = fileURLToPath(new URL('../../',import.meta.url));
const require = createRequire(new URL('../../docs/package.json',import.meta.url));
const extracted = inventory(root);

test('Safari reference reuse preserves provenance and rejects changed probes or inventories', () => {
  const rows = [{id:'Event.prototype.type',target:'Event.prototype',name:'type'}];
  const observation = {presence:'present',value:{type:'string',value:'coverage'}};
  const previous = {
    generated_at:'2026-10-25T00:00:00Z',fixture:{html:'fixture',url:'http://old-fixture/'},
    runtimes:{safari:{status:'measured',installed_driver:'reference build',environment:{url:'http://old-fixture/'}}},
    generator_sources:{'scripts/coverage/probe.mjs':'probe-hash'},
    fields:{'Event.prototype.type':{...rows[0],safari:observation}},
    targets:{'Event.prototype':{safari:{status:'available'}}},
  };
  const result = reuseSafariReference(previous,rows,'probe-hash','fixture');
  assert.deepEqual(result.observations.fields['Event.prototype.type'],observation);
  assert.equal(result.runtime.measured_at,previous.generated_at);
  assert.equal(result.runtime.fixture.url,'http://old-fixture/');
  assert.equal(result.runtime.reused,true);
  assert.equal(previous.runtimes.safari.reused,undefined);
  const regenerated = {...previous,generated_at:'2026-10-26T00:00:00Z',runtimes:{safari:result.runtime}};
  assert.equal(reuseSafariReference(regenerated,rows,'probe-hash','fixture').runtime.measured_at,previous.generated_at);
  assert.throws(()=>reuseSafariReference(null,rows,'probe-hash','fixture'),/measured Safari/);
  assert.throws(()=>reuseSafariReference(previous,rows,'changed','fixture'),/probe or fixture changed/);
  assert.throws(()=>reuseSafariReference(previous,rows,'probe-hash','changed'),/probe or fixture changed/);
  assert.throws(()=>reuseSafariReference(previous,[], 'probe-hash','fixture'),/inventory changed/);
  assert.throws(()=>reuseSafariReference(previous,[{...rows[0],name:'changed'}],'probe-hash','fixture'),/inventory changed/);
});

test('inventory includes distinct declaration forms without inventing local variables as globals', () => {
  const fields = new Map(extracted.rows.map(r=>[r.id,r]));
  for (const id of ['window.alert','window.onpointerdown','document.location','CSS.px',
    'Element.prototype.ariaLabel','Event.instance.isTrusted','Event.AT_TARGET',
    'Event.prototype.AT_TARGET','Event.prototype','WebGLRenderingContext.DEPTH_BUFFER_BIT',
    'WebGLRenderingContext.prototype.DEPTH_BUFFER_BIT','CSSStyleDeclaration.instance.color',
    'Document.prototype.[Symbol.unscopables].after','Plugin.prototype.[Symbol.iterator]',
    'MimeTypeArray.instance.[index]','MimeTypeArray.instance.[name]','Storage.instance.[name]',
    'Plugin.instance.description','MimeType.instance.enabledPlugin','Blob.instance.size']) assert.ok(fields.has(id),id);
  for (const name of ['i','len','out','minValue','maxValue','tagName','defaultValue','automationRate']) assert.ok(!fields.has(`window.${name}`),name);
  assert.ok(!fields.has('Storage.instance.k'));
  assert.deepEqual(fields.get('window.SharedArrayBuffer').legacy.kinds,['removed']);
  assert.deepEqual(fields.get('document.location').legacy.kinds,['get','set']);
  assert.ok(fields.get('window.alert').legacy.body_patterns.includes('empty_no_op'));
  assert.ok(fields.get('IDBFactory.prototype.open').legacy.sources.some(s=>s.includes('BOM/IDB/IDBFactory.js')));
  assert.ok(fields.get('IDBFactory.prototype.open').legacy.sources.some(s=>s.includes('BOM/IDBFactory.js')));
  assert.deepEqual(extracted.unresolved.map(x=>x.reason),Array(4).fill('missing_loader_dependency'));
  assert.equal(extracted.files.filter(f=>f.role==='browser_surface' && f.declaration_locations===0).length,0);
});

test('probe preserves undefined, descriptors and instance coverage without invoking methods or illegal getters', () => {
  let methodCalls=0,illegalGetterCalls=0,setterCalls=0;
  class Element { constructor(){this.instanceValue='instance';} get value(){return 7;} set value(_v){setterCalls++;} }
  Element.prototype[Symbol.unscopables] = {after:true};
  class NoReceiver { get forbidden(){illegalGetterCalls++;throw Error('Illegal receiver');} }
  const untaggable={get [Symbol.toStringTag](){throw new TypeError('No tag');}};
  const fixture={undef:undefined,nil:null,inf:Infinity,untaggable,fn(){methodCalls++;},get throws(){throw new TypeError('fixture failure');}};
  const definitions=[['fixture','undef'],['fixture','nil'],['fixture','inf'],['fixture','fn'],['fixture','throws'],
    ['fixture','missing'],['fixture','untaggable'],['Element.prototype','instanceValue'],['Element.prototype','value'],
    ['NoReceiver.prototype','forbidden'],['NeverExposed.prototype','x'],['Element.prototype.[Symbol.unscopables]','after']];
  const rows=definitions.map(([target,name])=>({id:`${target}.${name}`,target,name}));
  const result=JSON.parse(JSON.stringify(vm.runInNewContext(`(${probe.toString()})(${JSON.stringify(rows)})`,{
    fixture,Element,NoReceiver,document:{createElement(){return new Element();},readyState:'complete',compatMode:'CSS1Compat'},
    navigator:{userAgent:'fixture',platform:'fixture'},location:{href:'http://fixture/'},
  })));
  const fields=result.fields;
  assert.deepEqual(fields['fixture.undef'].value,{type:'undefined'});
  assert.deepEqual(fields['fixture.nil'].value,{type:'null'});
  assert.deepEqual(fields['fixture.inf'].value,{type:'number',value:'Infinity'});
  assert.equal(fields['fixture.fn'].callable,true);
  assert.equal(fields['fixture.missing'].presence,'absent');
  assert.equal(fields['fixture.throws'].presence,'present');
  assert.equal(fields['fixture.throws'].value.type,'read_error');
  assert.equal(fields['fixture.untaggable'].value.type,'object');
  assert.equal(fields['fixture.untaggable'].value.tag,null);
  assert.equal(fields['Element.prototype.instanceValue'].declared_target,false);
  assert.equal(fields['Element.prototype.instanceValue'].lookup,'receiver');
  assert.deepEqual(fields['Element.prototype.value'].descriptor,{kind:'accessor',enumerable:false,configurable:true,get:true,set:true});
  assert.deepEqual(fields['Element.prototype.value'].value,{type:'number',value:7});
  assert.equal(fields['NoReceiver.prototype.forbidden'].value.type,'unverified');
  assert.equal(fields['NeverExposed.prototype.x'].presence,'absent');
  assert.deepEqual(fields['Element.prototype.[Symbol.unscopables].after'].value,{type:'boolean',value:true});
  assert.equal(methodCalls+illegalGetterCalls+setterCalls,0);
});

test('COVERAGE.yaml accounts for every extracted field and current source hash', () => {
  const doc = require('yaml').parse(fs.readFileSync(`${root}/COVERAGE.yaml`,'utf8'));
  assert.equal(doc.schema_version,2);
  assert.deepEqual(doc.tracking,tracking);
  assert.equal(summarizeProgress(doc.fields).total_fields,extracted.rows.length);
  assert.equal(doc.summary.fields,extracted.rows.length);
  assert.equal(Object.keys(doc.fields).length,extracted.rows.length);
  assert.deepEqual(doc.sources,extracted.files);
  for (const [file,expected] of Object.entries(doc.generator_sources)) {
    assert.equal(createHash('sha256').update(fs.readFileSync(`${root}/${file}`)).digest('hex'),expected,`${file} changed; regenerate COVERAGE.yaml`);
  }
  for (const row of extracted.rows) {
    const f=doc.fields[row.id]; assert.ok(f,row.id); assert.deepEqual(f.legacy,row.legacy);
    for (const result of [f.brimp.core,f.brimp.with_plugins,f.safari]) {
      assert.ok(['present','absent','unverified'].includes(result.presence),row.id);
      if (result.presence==='present') assert.ok(result.value || result.reason,row.id);
    }
    if(doc.runtimes.safari.status==='unverified') {
      assert.equal(f.safari.presence,'unverified'); assert.equal(f.safari.value,undefined);
      assert.equal(f.comparison.core.value,'unverified');
    }
  }
});

test('regeneration preserves edited progress while replacing measurements and initializing new fields', () => {
  const progress={status:'done',target:'with_plugins',contract:'Callable no-op returns undefined without throwing.',tests:['tests/no-op.test.mjs'],notes:'Discarded-return workload verified.'};
  const previous={method:{progress,implemented:{core:'absent_in_fixture'}}};
  const fresh={method:{implemented:{core:'callable_surface'}},newField:{implemented:{core:'property_surface'}}};
  preserveProgress(fresh,previous);
  assert.deepEqual(fresh.method.progress,progress);
  assert.equal(fresh.method.implemented.core,'callable_surface');
  assert.deepEqual(fresh.newField.progress,initialProgress());
  fresh.method.progress.notes='Updated decision';
  assert.equal(previous.method.progress.notes,progress.notes);
  const next={method:{},newField:{}};
  preserveProgress(next,require('yaml').parse(require('yaml').stringify(fresh)));
  assert.deepEqual(next.method.progress,fresh.method.progress);
  assert.deepEqual(summarizeProgress(next,'done').fields,[{id:'method',...fresh.method.progress}]);
  assert.equal(summarizeProgress(next).statuses.unreviewed,1);
});

test('progress validation rejects misleading completion and invalid records', () => {
  const bad=[
    {...initialProgress(),status:'complete'},
    {...initialProgress(),status:['done']},
    {...initialProgress(),status:'planned'},
    {...initialProgress(),status:'done',target:'core',contract:'Supports reads'},
    {...initialProgress(),status:'blocked'},
    {...initialProgress(),status:'deferred'},
    {...initialProgress(),status:'not_planned'},
    {...initialProgress(),target:'plugins'},
    {...initialProgress(),tests:['']},
    {...initialProgress(),stats:'typo'},
  ];
  for(const p of bad) assert.throws(()=>validateProgress(p,'field'),/field.progress:/);
  assert.throws(()=>summarizeProgress({field:{progress:initialProgress()}},'unknown'),/Unknown status/);
  assert.doesNotThrow(()=>validateProgress({...initialProgress(),status:'blocked',notes:'Needs a browser fixture'},'field'));
});

test('regeneration does not silently discard decisions for removed inventory fields', () => {
  const old={removed:{progress:{...initialProgress(),status:'deferred',notes:'Waiting for a workload'}}};
  assert.throws(()=>preserveProgress({},old),/Tracked field removed from inventory: removed/);
  assert.throws(()=>preserveProgress({}, {removed:{progress:{...initialProgress(),notes:'Decision pending'}}}),/Tracked field removed/);
  const fields={};
  preserveProgress(fields,{removed:{progress:initialProgress()}});
  assert.deepEqual(fields,{});
});
