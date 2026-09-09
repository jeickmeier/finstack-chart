'use strict';
// FIX-GG01: actual primary WASM adapter. Arguments: module directory, Rust inputs, outputs.
const fs = require('node:fs'), path = require('node:path'), assert = require('node:assert/strict');
const root = path.resolve(__dirname, '../..');
const c = require(path.join(path.resolve(process.argv[2]), 'authoring.cjs'));
const inputs = path.resolve(process.argv[3]), out = path.resolve(process.argv[4]);
fs.mkdirSync(out, {recursive: true});
const output = new c.Output(fs.readFileSync(path.join(root, 'fixtures/capability/fonts/NotoSans-Regular.ttf')));
const files = fs.readdirSync(inputs).filter(x => x.endsWith('.plot.json')).sort();
assert.equal(files.length, 24);
for (const file of files) {
  const name = file.slice(0, -'.plot.json'.length);
  const plot = c.Plot.from_json(fs.readFileSync(path.join(inputs, file), 'utf8'));
  const chart = plot.chart();
  if (name.startsWith('hidden-')) chart.legend_visible(false);
  const [width, height] = name.startsWith('tight-') ? [130, 65] : [500, 300];
  const baseOptions = c.export_options(width, height), dpiOptions = baseOptions.dpi(96), options = dpiOptions.basis('current');
  baseOptions.free(); dpiOptions.free();
  const request = chart.request(output, options), frame = request.prepare(), scene = frame.scene();
  assert.deepEqual(scene, JSON.parse(fs.readFileSync(path.join(inputs, name + '.scene.json'), 'utf8')), name);
  fs.writeFileSync(path.join(out, name + '.scene.json'), JSON.stringify(scene, null, 2));
  for (const fmt of ['svg', 'pdf', 'png']) fs.writeFileSync(path.join(out, name + '.' + fmt), frame.export(fmt));
  if (name.startsWith('two-entry-')) {
    const legend = c.legend(), named = legend.scale('series'), untitled = named.untitled();
    const draft = plot.edit(), changed = draft.legend(untitled), edited = changed.build();
    chart.apply_plot(edited, 0n);
    const editedRequest = chart.request(output, options), editedFrame = editedRequest.prepare();
    const texts = editedFrame.scene().items.filter(i => i.primitive.Text).map(i => i.primitive.Text.text);
    assert(!texts.includes('Series') && !texts.includes('Color'));
    assert.equal(texts.filter(t => t === 'Alpha').length, 1);
    const generic = named.genericTitle(), genericDraft = draft.legend(generic), genericPlot = genericDraft.build();
    const genericRequest = output.request(genericPlot, options), genericFrame = genericRequest.prepare();
    const genericTexts = genericFrame.scene().items.filter(i => i.primitive.Text).map(i => i.primitive.Text.text);
    assert.equal(genericTexts.filter(t => t === 'Color').length, 1);
    assert(!genericTexts.includes('Series'));
    for (const value of [genericFrame, genericRequest, genericPlot, genericDraft, generic]) value.free();
    for (const value of [editedFrame, editedRequest, edited, changed, draft, untitled, named, legend]) value.free();
  }
  for (const value of [frame, request, chart, plot, options]) value.free();
}
output.free();
console.log('PASS FIX-GG01 WASM: 24 exact scenes, 72 exports and 6 host untitled/generic edits.');
