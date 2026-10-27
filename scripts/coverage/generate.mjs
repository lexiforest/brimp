import fs from 'node:fs';
import path from 'node:path';
import http from 'node:http';
import {fileURLToPath} from 'node:url';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {isDeepStrictEqual} from 'node:util';
import {inventory} from './inventory.mjs';
import {probe} from './probe.mjs';
import {tracking, preserveProgress} from './progress.mjs';
import {webdriver} from './webdriver.mjs';
import {reuseSafariReference} from './safari-reference.mjs';
import * as brimp from '../../bindings/node/index.js';

const root = fileURLToPath(new URL('../../', import.meta.url));
const require = createRequire(new URL('../../docs/package.json', import.meta.url));
const YAML = require('yaml');
const args = new Set(process.argv.slice(2));
if ([...args].some(a => !['--allow-unverified-safari','--reuse-safari'].includes(a)) || args.size > 1) throw Error('Usage: node scripts/coverage/generate.mjs [--allow-unverified-safari | --reuse-safari]');
const allowUnverified = args.has('--allow-unverified-safari');
const reuseSafari = args.has('--reuse-safari');
const outputPath=path.join(root,'COVERAGE.yaml');
const previous=fs.existsSync(outputPath)?YAML.parse(fs.readFileSync(outputPath,'utf8')):null;
const hash = file => createHash('sha256').update(fs.readFileSync(path.join(root,file))).digest('hex');
const html = '<!doctype html><html lang="en"><head><meta charset="utf-8"><title>Brimp coverage fixture</title></head><body></body></html>';
const server = http.createServer((req,res) => {
  res.writeHead(200, {'content-type':'text/html; charset=utf-8','cache-control':'no-store'}); res.end(html);
});
const collected = inventory(root);
const rows = collected.rows.map(({id,target,name}) => ({id,target,name}));
const reference = reuseSafari ? reuseSafariReference(previous, rows, hash('scripts/coverage/probe.mjs'), html) : null;
const source = `(${probe.toString()})(${JSON.stringify(rows)})`;
const plugins = ['environment','permissions','devices','media','workers','trusted-types','canvas','graphics','audio','rtc'];
let safariId;
function tally(values) { const counts = {}; for (const v of values) counts[v] = (counts[v] || 0)+1; return counts; }
function implemented(result) {
  if (result.presence === 'absent') return 'absent_in_fixture';
  if (result.presence !== 'present') return 'unverified';
  return result.callable ? 'callable_surface' : 'property_surface';
}
function compare(actual, expected) {
  if (expected.presence === 'unverified' || actual.presence === 'unverified') return {presence:'unverified',descriptor:'unverified',value:'unverified'};
  if (expected.presence === 'absent' || actual.presence === 'absent') return {
    presence:expected.presence === actual.presence?'both_absent':expected.presence === 'absent'?'brimp_only':'missing_in_brimp',
    descriptor:'not_applicable',value:'not_applicable',
  };
  const same = isDeepStrictEqual;
  const a=actual.value,b=expected.value;
  let value='unverified';
  if (a && b && !['unverified','read_error'].includes(a.type) && !['unverified','read_error'].includes(b.type)) {
    if (a.type === 'object' && b.type === 'object') value=typeof a.tag==='string'&&typeof b.tag==='string'?(a.tag===b.tag?'same_object_tag_only':'different_object_tag'):'unverified';
    else if (a.type === 'function' && b.type === 'function') value=same(a,b)?'same_function_name_and_length_only':'different_function_name_or_length';
    else value=same(a,b)?'same_sample':'different_sample';
  }
  return {presence:'both_present',descriptor:same(actual.descriptor,expected.descriptor)&&actual.declared_own===expected.declared_own&&actual.depth===expected.depth?'same':'different',value};
}
try {
  await new Promise(resolve => server.listen(0,'127.0.0.1',resolve));
  const url = `http://127.0.0.1:${server.address().port}/coverage.html`;
  const observations = {};
  for (const mode of ['core','with_plugins']) {
    console.log(`Probing Brimp ${mode}: ${rows.length} fields`);
    const session = await brimp.createSession({plugins:mode==='core'?[]:plugins.map(p=>brimp[brimp.pluginCatalog()[p].installers.node]())});
    try { const page = await session.get(url); observations[mode] = await page.evaluate(source); }
    finally { await session.close(); }
  }
  let safari;
  if (reference) {
    console.log(`Reusing Safari reference measured at ${reference.runtime.measured_at}`);
    safari = reference.runtime;
    observations.safari = reference.observations;
  } else {
  let installedDriver;
  try { installedDriver = execFileSync('/usr/bin/safaridriver',['--version'],{encoding:'utf8'}).trim(); }
  catch { installedDriver = 'unavailable'; }
  try {
    console.log('Probing installed Safari through WebDriver');
    const session = await webdriver('POST','/session',{capabilities:{alwaysMatch:{browserName:'safari'}}});
    safariId=session.sessionId;
    await webdriver('POST',`/session/${safariId}/timeouts`,{script:60000,pageLoad:60000,implicit:0});
    await webdriver('POST',`/session/${safariId}/url`,{url});
    observations.safari = await webdriver('POST',`/session/${safariId}/execute/sync`,{script:`return ${source};`,args:[]});
    safari = {status:'measured',reused:false,measured_at:new Date().toISOString(),fixture:{url,html},installed_driver:installedDriver,capabilities:session.capabilities};
  } catch (e) {
    if (!allowUnverified) throw e;
    safari = {status:'unverified',installed_driver:installedDriver,reason:String(e.message)};
    console.warn(`Safari unverified: ${e.message}`);
  }
  }
  const fields = {};
  for (const row of collected.rows) {
    const core=observations.core.fields[row.id], withPlugins=observations.with_plugins.fields[row.id];
    const expected=observations.safari?.fields[row.id] ?? {presence:'unverified',reason:'safari_session_unavailable'};
    if (!core || !withPlugins) throw Error(`Missing probe result: ${row.id}`);
    fields[row.id] = {target:row.target,name:row.name,legacy:row.legacy,
      implemented:{core:implemented(core),with_plugins:implemented(withPlugins),behavior:'not_tested'},
      brimp:{core,with_plugins:withPlugins},safari:expected,
      comparison:{core:compare(core,expected),with_plugins:compare(withPlugins,expected)}};
  }
  preserveProgress(fields,previous?.fields);
  const report = {
    schema_version:2,
    generated_at:new Date().toISOString(),
    scope:'Every statically extracted browser-facing declaration and explicit global removal in the supplied node-env tree, deduplicated by target and key; not every Web API or every inherited member.',
    tracking,
    regenerate:{command:'node scripts/coverage/generate.mjs',prerequisites:['Build the current native bindings and copy them with python3 bindings/prepare_local.py.','Install existing docs dependencies (TypeScript and yaml).','Enable Safari Allow Remote Automation and run safaridriver -p 4447.'],
      webdriver_url_env:'SAFARI_WEBDRIVER_URL',reuse_reference:'node scripts/coverage/generate.mjs --reuse-safari',explicit_partial_mode:'node scripts/coverage/generate.mjs --allow-unverified-safari',validate:'node --test scripts/coverage/coverage.test.mjs'},
    interpretation:{
      implemented:'Surface coverage in this fixture. callable_surface includes useful no-ops. It does not assert successful invocation, correct returns, side effects, setters, or full browser conformance.',
      absence:'absent_in_fixture means the key is unavailable through the declared target and representative receiver in this fixture. This includes targets not exposed at all; it does not prove that a feature has no alternative API. Receiver errors are retained under targets. Unsampled instances of exposed constructors remain unverified.',
      safari:'Observed reference for this exact Safari build and fixture; runtimes.safari records its measurement date and fixture when reused. Unverified fields have no asserted correct Safari value. Safari absence is a valid result for Chromium-only/obsolete APIs.',
      values:'Typed values distinguish undefined, null, non-finite numbers, functions and object tags. Object contents/identity and function behavior are not compared. Getter exceptions are preserved.',
      declared_target:'The requested prototype/static/namespace object; declared_own and lookup depth distinguish own, inherited and instance implementations.',
      receivers:'Representative receiver recipes are under targets. Most elements are detached and empty; events have type coverage; context-dependent getters may throw legitimately.',
      samples:'URLs, user agent, languages, screen/viewport, activation, storage, timing, GPU/audio and permission state are environment-dependent observations, not constants to hardcode.',
      extraction:'AST extraction includes object literals, assignments, accessor/constant maps, safe-member helper lists, CSS lists, symbols and named/indexed patterns. Legacy source is never executed; missing helpers prevent verifying its actual runtime descriptors.',
      removals:'legacy.kinds: [removed] records names the old project explicitly deletes, including Node globals and SharedArrayBuffer. These are audited fields, not surfaces provided by the old environment.',
      exclusions:'Loader/configuration/harness internals are accounted for by source file roles; internal lwVm state and code embedded in Worker demonstration strings are not browser API declarations.',
      dynamic:'[index] and [name] represent unbounded runtime collection properties, not literal browser keys. They require a populated collection workload and are unverified here.',
      comparison:'same_sample is equality in this fixture only. Descriptor comparison includes descriptor flags, declared-own status and lookup depth; no prototype-chain conformance claim.',
    },
    fixture:{url,html,top_level:true,network:'Only the local fixture is requested; no devices, permission prompts, worker payloads or WebRTC negotiation.',
      mutations:'Creates detached receivers, a temporary performance mark, audio graphs (never starts sources or resumes playback), and a local unnegotiated peer/data channel when available. Cleans up marks, WebGL, AudioContext and peer connections.'},
    runtimes:{
      brimp:{git_head:execFileSync('git',['rev-parse','HEAD'],{cwd:root,encoding:'utf8'}).trim(),working_tree:'Includes uncommitted changes; native binary hash identifies tested build.',node:process.version,
        binding_sha256:hash('bindings/node/brimp.node'),plugins,
        javascript_sources:Object.fromEntries(['index.js','plugins.js',...plugins.map(p=>`bundled/${p==='cssTypedOm'?'css-typed-om':p}.js`)].map(f=>[`bindings/node/${f}`,hash(`bindings/node/${f}`)])),
        core:observations.core.environment,with_plugins:observations.with_plugins.environment,
        cleanup_errors:{core:observations.core.cleanup_errors,with_plugins:observations.with_plugins.cleanup_errors}},
      safari:{...safari,...(observations.safari?{environment:observations.safari.environment,cleanup_errors:observations.safari.cleanup_errors}:{})},
    },
    summary:{legacy_files:collected.files.length,fields:rows.length,targets:new Set(rows.map(r=>r.target)).size,
      explicit_global_removals:collected.rows.filter(r=>r.legacy.kinds.includes('removed')).length,
      core:tally(Object.values(fields).map(f=>f.implemented.core)),with_plugins:tally(Object.values(fields).map(f=>f.implemented.with_plugins)),
      safari:tally(Object.values(fields).map(f=>f.safari.presence)),
      comparison_core:tally(Object.values(fields).map(f=>f.comparison.core.presence)),comparison_with_plugins:tally(Object.values(fields).map(f=>f.comparison.with_plugins.presence))},
    generator_sources:Object.fromEntries(['scripts/coverage/inventory.mjs','scripts/coverage/probe.mjs','scripts/coverage/generate.mjs','scripts/coverage/progress.mjs','scripts/coverage/webdriver.mjs','scripts/coverage/safari-reference.mjs'].map(f=>[f,hash(f)])),
    inventory_issues:collected.unresolved,
    legacy_prototype_relationships:collected.relationships,
    legacy_prototype_aliases:collected.aliases,
    sources:collected.files,
    targets:Object.fromEntries([...new Set(rows.map(r=>r.target))].map(target=>[target,{core:observations.core.targets[target],with_plugins:observations.with_plugins.targets[target],
      safari:observations.safari?.targets[target] ?? {status:'unverified'}}])),
    fields,
  };
  const doc = new YAML.Document(report);
  // Compact leaf maps keep thousands of rows scannable without hiding provenance.
  YAML.visit(doc,{Map(_key,node){if(node.items.length && node.items.every(p=>YAML.isScalar(p.value)) && node.items.reduce((n,p)=>n+String(p.key.value).length+String(p.value.value).length,0)<240) node.flow=true;},Seq(_key,node){if(node.items.length<=5 && node.items.every(YAML.isScalar)) node.flow=true;}});
  const output = '# Measurements are generated; fields.<id>.progress is editable and preserved on regeneration.\n' + doc.toString({lineWidth:0,aliasDuplicateObjects:false});
  const parsed = YAML.parse(output);
  if (Object.keys(parsed.fields).length !== rows.length || JSON.stringify(parsed)!==JSON.stringify(report)) throw Error('YAML round-trip validation failed');
  const temporary=`${outputPath}.${process.pid}.tmp`;
  try { fs.writeFileSync(temporary,output); fs.renameSync(temporary,outputPath); }
  finally { if(fs.existsSync(temporary)) fs.unlinkSync(temporary); }
  console.log(JSON.stringify(report.summary,null,2));
} finally {
  if (safariId) await webdriver('DELETE',`/session/${safariId}`).catch(e=>console.warn(`Safari cleanup: ${e.message}`));
  server.closeAllConnections(); await new Promise(resolve=>server.close(resolve));
}
