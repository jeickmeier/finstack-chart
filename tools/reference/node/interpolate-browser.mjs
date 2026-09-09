// FIX-I01-F. Real browser oracle, no downloads and no production browser dependency.
// PLAYWRIGHT_MODULE names an installed Playwright module; CHROMIUM_EXECUTABLE names
// the pinned Chrome Headless Shell 151.0.7922.34 binary. Output is committed for offline use.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {pathToFileURL} from 'node:url';
import {root,workspace,provenance,writeJson,digest} from './provenance.mjs';
if(!process.env.PLAYWRIGHT_MODULE||!process.env.CHROMIUM_EXECUTABLE)throw Error('Set PLAYWRIGHT_MODULE and CHROMIUM_EXECUTABLE to installed pinned tools.');
const {chromium}=await import(pathToFileURL(process.env.PLAYWRIGHT_MODULE));
const browser=await chromium.launch({headless:true,executablePath:process.env.CHROMIUM_EXECUTABLE});
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/parity/d3-interpolate'));
try{
 if(browser.version()!=='151.0.7922.34')throw Error('Expected Chromium 151.0.7922.34, got '+browser.version());
 const page=await browser.newPage({locale:'en-US',timezoneId:'UTC'});
 await page.setContent('<!doctype html><meta charset="utf-8"><title>FIX-I01 transform oracle</title><main>Headless transform reference</main>');
 for(const name of ['d3-color','d3-interpolate'])await page.addScriptTag({path:path.join(workspace,'node_modules',name,'dist',name+'.min.js')});
 const cases=await page.evaluate(()=>{
  const times=[-.25,0,.25,.5,.75,1,1.25];
  const rows=[];
  const css=[['none','none'],['none','translate(10px, -20px)'],['translateX(4px)','translateY(8px)'],['rotate(350deg)','rotate(10deg)'],['rotate(.25turn)','rotate(3.141592653589793rad)'],['scale(1)','scale(-2,3)'],['skewX(20deg)','skewY(-10deg)'],['translate(10px,20px) rotate(30deg) scale(2,3)','scale(2) translate(4px,5px)'],['matrix(1,0,0,1,10,20)','matrix(-2,0,0,3,-10,5)'],['matrix(0,0,0,1,0,0)','matrix(1,0,0,0,3,4)'],['matrix(1,2,3,4,5,6)','matrix(2,-1,1,2,3,4)'],['none','rotate(100grad)'],['translate(1in, 2cm)','translate(72pt, 6pc)'],['none','translate(50%,0)'],['none','broken(1)'],['none','rotateX(30deg)'],['scaleX(2)','scaleY(3)'],['skew(10deg,20deg)','skew(-10deg)'],['scale(50%,200%)','scale(150%)'],['none','matrix(0,0,1,1,0,0)'],['translate(1mm,1q)','translate(1in,1pc)']];
  const svg=[['',''],['','translate(10,-20)'],['translate(4)','translate(8 2)'],['rotate(350)','rotate(10)'],['rotate(30,10,20)','rotate(-30,10,20)'],['scale(1)','scale(-2,3)'],['skewX(20)','skewY(-10)'],['translate(10 20) rotate(30) scale(2 3)','scale(2) translate(4 5)'],['matrix(1 0 0 1 10 20)','matrix(-2 0 0 3 -10 5)'],['matrix(0 0 0 1 0 0)','matrix(1 0 0 0 3 4)'],['matrix(1 2 3 4 5 6)','matrix(2 -1 1 2 3 4)'],['','rotate(1e2)'],['','broken(1)'],['','matrix(0,0,1,1,0,0)'],['translate(1-2),scale(2)','translate(+3,+4)']];
  function matrix(text,syntax){if(syntax==='css'){const m=new DOMMatrix(text);return [m.a,m.b,m.c,m.d,m.e,m.f];}const g=document.createElementNS('http://www.w3.org/2000/svg','g');g.setAttribute('transform',text);const v=g.transform.baseVal.consolidate();return v?[v.matrix.a,v.matrix.b,v.matrix.c,v.matrix.d,v.matrix.e,v.matrix.f]:[1,0,0,1,0,0];}
  for(const [syntax,seeds]of [['css',css],['svg',svg]])for(const [index,[a,b]]of seeds.entries()){
   const row={id:syntax+'-'+index,syntax,a,b,times};
   if((a+b).includes('%')&&(a+b).includes('translate'))row.adaptation='RequireResolvedContext';
   if(b.includes('broken'))row.adaptation='RejectSyntax';
   if(b.includes('rotateX'))row.adaptation='Reject3D';
   try{row.matrices=[matrix(a,syntax),matrix(b,syntax)];const f=d3[syntax==='css'?'interpolateTransformCss':'interpolateTransformSvg'](a,b);row.expected=times.map(t=>{const text=f(t);const m=matrix(text||'none',syntax);return {text,matrix:m,point:[m[0]*2+m[2]*3+m[4],m[1]*2+m[3]*3+m[5]]};});}catch(e){row.expected={error:e.name};}
   rows.push(row);
  }
  return rows;
 });
 writeJson(path.join(out,'transforms.json'),{schema_version:1,cases});
 writeJson(path.join(out,'transforms-manifest.json'),{schema_version:1,...provenance('d3-interpolate'),browser:{engine:'Chromium',version:browser.version(),executable_sha256:digest(fs.readFileSync(process.env.CHROMIUM_EXECUTABLE)),os:os.platform(),release:os.release(),arch:os.arch()},case_count:cases.length,generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'transforms.json'))),tolerances:{decomposition:{absolute:1e-12,relative:1e-12},browser_parser_resolution:{absolute:1e-5,relative:1e-6},canonical_template:'exact; resolved-matrix numeric tokens compare at 1e-12 absolute/relative',parser_note:'Chromium CSS lengths/angles and SVG values may round during parsing; binary64 headless input preserves authored precision and uses the measured geometry tolerance.'},status:'Reference only; WP-IP05/06 acceptance remains open.'});
 console.log(`PASS FIX-I01-F real Chromium ${browser.version()}: ${cases.length} CSS/SVG transform pairs.`);
}finally{await browser.close();}
