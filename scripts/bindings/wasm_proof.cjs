// Execute generated bindings in Node's actual WebAssembly runtime, without threads or browser APIs.
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const root = path.resolve(__dirname, '../..');
// Test-only observation of the instantiated module; no production memory API is added.
const NativeInstance = WebAssembly.Instance;
let memory;
WebAssembly.Instance = class extends NativeInstance {
    constructor(module, imports) { super(module, imports); memory = this.exports.memory; }
};
let bindings;
try { bindings = require(path.resolve(process.argv[2], 'chart_wasm.js')); }
finally { WebAssembly.Instance = NativeInstance; }
const output = path.resolve(process.argv[3]); fs.mkdirSync(output, {recursive:true});
const read = name => fs.readFileSync(path.join(root, `fixtures/bindings/${name}.json`), 'utf8');
const font = Uint8Array.from(fs.readFileSync(path.join(root, 'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const chart = new bindings.Chart(read('chart'), read('data'), read('profile'), font);
font.fill(0); // Input was copied; subsequent rendering must still work.
const save = (name, value) => fs.writeFileSync(path.join(output, `${name}.json`), value);
save('initial', chart.semantics()); save('definition', chart.definition());
save('transaction', chart.transaction(read('correction'))); save('replay', chart.transaction(read('correction')));
save('action', chart.action(read('action'))); save('final', chart.semantics());
save('scene', chart.scene()); save('state', chart.state());
const stateBefore = chart.state();
chart.restore_state(stateBefore, '1');
assert.throws(()=>chart.action(read('action')),error=>JSON.parse(error.message).code==='CHART_REVISION_CONFLICT');
assert.equal(chart.state(), stateBefore);
const stale = JSON.parse(read('correction')); stale.id = 'stale-correction';
assert('Conflict' in JSON.parse(chart.transaction(JSON.stringify(stale))));
assert.equal(chart.state(), stateBefore);
const svg = chart.svg(); assert(svg instanceof Uint8Array);
fs.writeFileSync(path.join(output, 'chart.svg'), svg);
const retained = Uint8Array.from(svg);
// Inspect generated module's exported memory solely to force and prove invalidation.
// The public chart API exposes only independent copies, never memory views.
assert(memory instanceof WebAssembly.Memory);
assert(!(memory.buffer instanceof SharedArrayBuffer));
const oldBuffer = memory.buffer;
const oldView = new Uint8Array(oldBuffer, 0, 1);
memory.grow(1);
assert.equal(oldBuffer.byteLength, 0); assert.equal(oldView.byteLength, 0);
assert.notEqual(svg.buffer, memory.buffer);
assert.deepEqual(svg, retained); assert.deepEqual(chart.svg(), retained);
svg.fill(0); assert.deepEqual(chart.svg(), retained);
for(let i=0;i<20;i++) {
    const state = JSON.parse(chart.state());
    const action = {version:1, definition_revision:state.definition_revision, expected_state:state.state_revision,
        action:{SetViewport:{x:i%2 ? [0.5,1.6] : [0.6,1.5], y:null}}};
    chart.action(JSON.stringify(action));
    assert.deepEqual(JSON.parse(chart.semantics()).layers[0].rows.Binned.map(r=>r.count), ['1','2']);
}
save('ownership', JSON.stringify({owned_font_mutation:true, owned_svg:true, invalidated_old_view:oldView.byteLength===0, memory_grew:true, repeated_updates:20}));
chart.dispose(); chart.dispose();
assert.throws(()=>chart.scene(), error => JSON.parse(error.message).code==='CHART_DISPOSED_HANDLE');
assert(retained.length>0); chart.free();
const negative = JSON.parse(read('negative'));
for(const test of negative) {
    const values = Object.fromEntries(['chart','data','profile'].map(n=>[n,JSON.parse(read(n))]));
    let node=values[test.target]; const parts=test.path.split('/').slice(1);
    for(const key of parts.slice(0,-1)) node=node[key];
    if(test.remove) delete node[parts.at(-1)]; else node[parts.at(-1)]=test.value;
    const originalFont=Uint8Array.from(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf')));
    assert.throws(()=>new bindings.Chart(JSON.stringify(values.chart),JSON.stringify(values.data),JSON.stringify(values.profile),originalFont), error=>JSON.parse(error.message).code===test.code,test.name);
}
console.log(`PASS WASM: real Node WebAssembly, portable fixture, copies, invalidated old memory view, repeated updates, disposal and ${negative.length} malformed cases`);

// WP-10: shared portable builtin semantics and final destination scenes in actual WASM.
const statistics = {}, scenes = {};
for (const proofCase of [...JSON.parse(fs.readFileSync(path.join(root,'fixtures/statistics/portable-cases.json'),'utf8')), ...JSON.parse(fs.readFileSync(path.join(root,'fixtures/families/portable-cases.json'),'utf8'))]) {
    const proof = new bindings.Chart(JSON.stringify(proofCase.chart),JSON.stringify(proofCase.data),read('profile'),Uint8Array.from(fs.readFileSync(path.join(root,'fixtures/capability/fonts/NotoSans-Regular.ttf'))));
    statistics[proofCase.name] = JSON.parse(proof.semantics()); scenes[proofCase.name] = JSON.parse(proof.scene());
    fs.writeFileSync(path.join(output,`statistics-${proofCase.name}.svg`),proof.svg());
    proof.dispose(); proof.free();
}
save('statistics',JSON.stringify(statistics)); save('statistics-scenes',JSON.stringify(scenes));
console.log(`PASS ${Object.keys(statistics).length} statistic/position/family cases in actual WASM`);
