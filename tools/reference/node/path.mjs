// WP-P01: source-backed command sequences, including observable non-Canvas state.
import {Path, path, pathRound} from 'd3-path';
import fs from 'node:fs';
import nodePath from 'node:path';
import {root, provenance, writeJson, retainLicense, digest} from './provenance.mjs';

const output = nodePath.resolve(process.argv[2] ?? nodePath.join(root, 'fixtures/parity/d3-path'));
const cases = [];
const op = (op, args) => args === undefined ? {op} : {op, args};
const move = (x,y) => op('moveTo',[x,y]), line = (x,y) => op('lineTo',[x,y]);
const close = () => op('closePath');
const arc = (x,y,r,a0,a1,anticlockwise=false) => op('arc',{x,y,r,a0,a1,anticlockwise});
const arcTo = (...args) => op('arcTo', args);
const rect = (...args) => op('rect', args);
function add(id, operations, digits=null, fixture='FIX-P01') {
  let p = digits === null ? path() : pathRound(digits);
  const observations=[];
  for (const operation of operations) {
    let error=null;
    try {
      if (operation.op==='arc') {
        const a=operation.args;p.arc(a.x,a.y,a.r,a.a0,a.a1,a.anticlockwise);
      } else p[operation.op](...(operation.args ?? []));
    } catch (e) { error=e.message; }
    observations.push({svg:String(p), current:[p._x1,p._y1], start:[p._x0,p._y0], error});
  }
  cases.push({id, fixture, digits, operations, observations, svg:String(p)});
}
add('empty',[]);
add('basic-controls',[move(1,2),line(3,4),op('quadraticCurveTo',[5,6,7,8]),op('bezierCurveTo',[9,10,11,12,13,14])]);
add('multiple-moves',[move(1,2),move(3,4),line(5,6),move(7,8)]);
add('empty-close',[close(),close()]);
add('repeat-close-and-continue',[move(1,2),line(3,4),close(),close(),line(5,6),close()]);
for (const operation of [line(1,2),op('quadraticCurveTo',[1,2,3,4]),op('bezierCurveTo',[1,2,3,4,5,6])]) {
  add('bare-'+operation.op,[operation,close(),line(7,8)]);
  add('post-close-'+operation.op,[move(1,2),line(3,4),close(),operation,close()]);
}
for (const w of [-5,0,5]) for(const h of [-4,0,4])
  add(`rect-${w}-${h}`,[rect(10,20,w,h),line(17,23),close()],null,'FIX-P03');
add('opposite-winding-holes',[rect(0,0,20,20),rect(5,5,-10,10)],null,'FIX-P03');
for(const ccw of [false,true]) for(const sweep of [0,1e-6-1e-12,1e-6,1e-6+1e-12,Math.PI,2*Math.PI-1e-6-1e-12,2*Math.PI-1e-6,2*Math.PI-1e-6+1e-12,2*Math.PI,3*Math.PI,-Math.PI,-2*Math.PI])
  add(`arc-${ccw}-${sweep}`,[arc(2,3,5,0,sweep,ccw),close(),arcTo(10,0,10,10,2)],3,'FIX-P02');
add('zero-radius-empty-state',[arc(1,2,0,0,1),close(),line(3,4)],null,'FIX-P02');
add('zero-sweep-empty-state',[arc(1,2,5,0,0),close(),arcTo(8,8,9,9,2)],null,'FIX-P02');
add('connecting-arc',[move(0,0),arc(10,20,5,-1,2),line(4,5)],3,'FIX-P02');
for(const delta of [1e-6-1e-12,1e-6,1e-6+1e-12])
  add(`arc-connection-${delta}`,[move(5+delta,0),arc(0,0,5,0,Math.PI)],3,'FIX-P02');
add('negative-radius-atomic',[move(2,3),arc(0,0,-1,0,1),line(4,5),arcTo(5,6,7,8,-2),line(9,10)],null,'FIX-P05');
for (const prefix of [[],[move(0,0)],[move(10,0)],[move(9.999,0)]])
  for(const radius of [0,2,20]) add(`arcTo-${prefix.length ? prefix[0].args.join('-'):'empty'}-${radius}`,[...prefix,arcTo(10,0,10,10,radius),close(),line(3,4)],3,'FIX-P02');
add('arcTo-reverse',[move(0,0),arcTo(10,0,10,-10,2)],3,'FIX-P02');
add('arcTo-collinear',[move(0,0),arcTo(10,0,20,0,2)],null,'FIX-P02');
add('arcTo-coincident-end',[move(0,0),arcTo(10,0,10,0,2)],null,'FIX-P02');
for(const delta of [1e-6-1e-12,1e-6,1e-6+1e-12]) {
  add(`arcTo-distance-${delta}`,[move(10-Math.sqrt(delta),0),arcTo(10,0,10,10,2)],3,'FIX-P02');
  add(`arcTo-cross-${delta}`,[move(0,0),arcTo(1,0,2,delta,2)],3,'FIX-P02');
}
for(const digits of [0,1,3,15,16,2.9]) {
  add(`round-${digits}`,[move(1.5,-1.5),line(-0,1.23456789),line(1e-9,1e22)],digits,'FIX-P04');
  add(`round-small-arc-${digits}`,[arc(0,0,0.00001,0,Math.PI/2),arcTo(1,1,2,0,0.5)],digits,'FIX-P04');
}
add('unrounded-magnitudes',[move(-0,1e-9),line(1e22,1e-7),line(1e-6,1e21)],null,'FIX-P04');
const constructors = [{name:'path',svg:String(path())},{name:'Path',svg:String(new Path())},
  {name:'pathRound-default',svg:''}];
// Methods return undefined in JavaScript; constructor precision is observed separately.
let defaultRounded=pathRound();defaultRounded.moveTo(1.23456,2.34567);constructors[2].svg=String(defaultRounded);
const invalid_digits=[-1,-0.1].map(digits=>{try{pathRound(digits);return {digits,error:null};}catch(e){return {digits,error:e.message};}});
writeJson(nodePath.join(output,'cases.json'), {schema_version:1, reference:provenance('d3-path'), constructors,invalid_digits,cases});
retainLicense('d3-path',output);
writeJson(nodePath.join(output,'manifest.json'), {schema_version:1, reference:provenance('d3-path'),generator:'tools/reference/node/path.mjs', generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),case_count:cases.length, cases_sha256:digest(fs.readFileSync(nodePath.join(output,'cases.json'))), parity:'OPEN: stored reference inputs; implementation packages own comparisons.'});
console.log(`PASS WP-P01: ${cases.length} deterministic path sequences; all constructors and eight drawing methods.`);
