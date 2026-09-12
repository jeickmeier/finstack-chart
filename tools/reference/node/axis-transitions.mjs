// FIX-19-H: actual D3 joins and transitions, driven by a deterministic host clock.
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {root, workspace, provenance, writeJson, digest} from './provenance.mjs';
if (!process.env.PLAYWRIGHT_MODULE || !process.env.CHROMIUM_EXECUTABLE) throw Error('Set PLAYWRIGHT_MODULE and CHROMIUM_EXECUTABLE.');
const {chromium} = await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE));
const browser = await chromium.launch({headless: true, executablePath: process.env.CHROMIUM_EXECUTABLE});
const packages = ['d3-array','d3-color','d3-interpolate','d3-format','d3-time','d3-time-format','d3-scale','d3-selection','d3-dispatch','d3-timer','d3-ease','d3-transition','d3-axis'];
const out = path.resolve(process.argv[2] || path.join(root, 'fixtures/axes'));
try {
  if (browser.version() !== '151.0.7922.34') throw Error('Wrong pinned Chromium: ' + browser.version());
  const context = await browser.newContext({locale: 'en-US', timezoneId: 'UTC', deviceScaleFactor: 1});
  const page = await context.newPage();
  await page.setContent('<svg xmlns="http://www.w3.org/2000/svg"></svg>');
  await page.evaluate(() => {
    let clock = 1000, frames = [];
    Object.defineProperty(performance, 'now', {value: () => clock});
    window.requestAnimationFrame = callback => { frames.push(callback); return frames.length; };
    window.setInterval = () => 1;
    window.clearInterval = () => {};
    window.advance = delta => { clock += delta; const callbacks = frames; frames = []; callbacks.forEach(f => f(clock)); d3.timerFlush(); };
  });
  for (const name of packages) await page.addScriptTag({path: path.join(workspace, 'node_modules', name, 'dist', name + '.min.js')});
  const cases = await page.evaluate(() => {
    const cases = [], svg = d3.select('svg');
    function run(id, before, after, interrupt) {
      svg.selectAll('*').interrupt().remove(); advance(1000);
      const scale = spec => {
        const s = d3[spec.band ? 'scaleBand' : 'scaleLinear']().domain(spec.domain || [0,1]).range(spec.range || [0,100]);
        if (spec.band) s.padding(.2).round(!!spec.round);
        return s;
      };
      const axis = spec => {
        const a = d3['axis' + (spec.side || 'Bottom')](scale(spec)).tickValues(spec.values);
        for (const k of ['tickSizeInner','tickSizeOuter','tickPadding','offset']) if (k in spec) a[k](spec[k]);
        if (spec.labels) a.tickFormat((v,i) => spec.labels[i]);
        return a;
      };
      let next = 0;
      const ids = new WeakMap(), identity = n => { if (!ids.has(n)) ids.set(n,next++); return ids.get(n); };
      const attrs = n => Object.fromEntries(Array.from(n.attributes).sort((a,b)=>a.name.localeCompare(b.name,'en')).map(a=>[a.name,a.value]));
      const g = svg.append('g');
      const capture = (stage, fraction) => ({stage, fraction, domain: g.select('.domain').attr('d'), ticks: Array.from(g.node().querySelectorAll(':scope > .tick')).map(n => ({identity: identity(n), value: n.__data__, attributes: attrs(n), line: attrs(n.querySelector('line')), text: {attributes: attrs(n.querySelector('text')), label: n.querySelector('text').textContent}}))});
      g.call(axis(before)); const states = [capture('before', null)];
      const start = spec => {
        // Orientation is explicit whole-guide replacement, not reuse of historical DOM attributes.
        if ((before.side || 'Bottom') !== (spec.side || 'Bottom')) { g.selectAll('*').interrupt().remove(); g.call(axis(spec)); }
        else g.transition().duration(100).ease(d3.easeLinear).call(axis(spec));
        advance(0);
      };
      start(after); states.push(capture('start', 0)); advance(50); states.push(capture('mid', .5));
      if (interrupt) { start(interrupt); states.push(capture('interrupt-start', 0)); advance(50); states.push(capture('interrupt-mid', .5)); }
      advance(50); states.push(capture('end', 1));
      const final = interrupt || after;
      const fresh = svg.append('g').call(axis(final));
      const strip = n => ({domain: n.querySelector('.domain').getAttribute('d'), ticks: Array.from(n.querySelectorAll('.tick')).map(t=>({value:t.__data__,label:t.querySelector('text').textContent}))});
      if (JSON.stringify(strip(g.node())) !== JSON.stringify(strip(fresh.node()))) throw Error('Final/fresh mismatch ' + id);
      const labels = spec => { const node = svg.append('g').call(axis(spec)); const result = node.selectAll('.tick text').nodes().map(n=>n.textContent); node.remove(); return result; };
      cases.push({id,before,after,after_labels:labels(after),...(interrupt ? {interrupt,interrupt_labels:labels(interrupt)} : {}),states});
    }
    for (const side of ['Top','Right','Bottom','Left']) {
      run('enter-move-exit-'+side, {side,values:[0,.5,1]}, {side,domain:[0,2],values:[0,1,2]});
      run('caps-padding-'+side, {side,values:[0,1]}, {side,range:[20,80],values:[0,1],tickSizeOuter:0,tickSizeInner:-20,tickPadding:-3});
    }
    run('duplicates', {values:[.5,.5,0,1]}, {values:[1,.5,.5,0]});
    run('projected-collision', {values:[0,.5,1]}, {domain:[1,1],values:[0,.5,1,2]});
    run('labels-immediate', {values:[0,.5,1],labels:['a','b','c']}, {values:[0,.5,1],labels:['x','y','z']});
    run('interruption', {values:[0,.5,1]}, {domain:[0,2],values:[0,1,2]}, {domain:[0,4],range:[10,210],values:[0,2,4]});
    run('band-reorder', {band:true,domain:['a','b','c'],values:['a','b','c']}, {band:true,domain:['c','a','d'],values:['c','a','d'],range:[100,0],round:true});
    run('offset-change', {values:[0,.5,1]}, {domain:[0,2],values:[0,1,2],offset:3});
    run('caps-enter', {values:[0,1],tickSizeOuter:0}, {values:[0,1],range:[20,80],tickSizeOuter:6});
    // Exhaust all reorderings with interleaved exits; selection.order must not
    // silently append exits or use a semantic-value-only join.
    const permute = a => a.length ? a.flatMap((v,i)=>permute(a.filter((_,j)=>i!==j)).map(t=>[v,...t])) : [[]];
    for (const [i,values] of permute([0,.25,.5,1]).entries()) run('join-order-'+i,{values:[0,.125,.25,.375,.5,.75,1]},{values});
    run('empty', {values:[0,.5,1]}, {values:[]});
    run('orientation-replacement', {values:[0,1]}, {side:'Left',values:[0,1]});
    return cases;
  });
  writeJson(path.join(out,'transitions.json'), {schema_version:1,cases});
  writeJson(path.join(out,'transitions-manifest.json'), {schema_version:1,references:packages.map(provenance),browser:{engine:'Chromium',version:browser.version(),executable_sha256:digest(fs.readFileSync(process.env.CHROMIUM_EXECUTABLE))},generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'transitions.json'))),cases:cases.length,clock:'performance.now and requestAnimationFrame are controlled before loading D3; real public transitions, SVG transform parsing, linear duration 100',boundary:'D3 transition oracle only; downstream core, host lifecycle and publication comparisons required.'});
  console.log('PASS FIX-19-H:', cases.length, 'deterministic transition cases');
  await context.close();
} finally { await browser.close(); }
