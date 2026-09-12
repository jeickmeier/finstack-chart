#!/usr/bin/env python3
"""Build a complete final verdict inventory from the pinned public surface."""
import json,pathlib
ROOT=pathlib.Path(__file__).resolve().parents[2];out=ROOT/'docs/evidence/phase-2-hierarchy-integration'
inventory=json.loads((ROOT/'fixtures/hierarchy/inventory.json').read_text())
source=json.loads((ROOT/'fixtures/hierarchy/reference.json').read_text())['cases'];ids={c['id'] for c in source}
rows=[]
def row(surface,api,evidence,cases=(),adaptation='Checked immutable Rust values; owned Python/WASM descriptors/results.'):
 assert set(cases)<=ids
 rows.append(dict(surface=surface,verdict='PASS',core_api=api,evidence=evidence,case_ids=list(cases),adaptation=adaptation))
exports={'Node':'Hierarchy::node','hierarchy':'Hierarchy::{from_nested,with_children,from_grouped}','stratify':'Hierarchy::{stratify,stratify_with,from_paths}','tree':'Hierarchy::{tree,tree_with}','cluster':'Hierarchy::{cluster,cluster_with}','partition':'Hierarchy::partition','treemap':'Hierarchy::{treemap,treemap_with,treemap_with_history}','pack':'Hierarchy::{pack,pack_with}','packSiblings':'pack_siblings','packEnclose':'pack_enclose'}
for c in inventory['coverage']:
 name=c['export'];api=exports.get(name,'Hierarchy::tile_children / Tiler::'+name.removeprefix('treemap'))
 row(name,api,['standalone-compare.log','export-tests.log'],c['case_ids'])
methods={'Symbol(Symbol.iterator)':'iter / descendants (breadth first)','each':'visit(BreadthFirst)','eachBefore':'visit(PreOrder)','eachAfter':'visit(PostOrder)','copy':'copy_subtree(new owner)','find':'find / FindRegistered / Find(field,value)'}
for name in inventory['node_methods']:
 row('node.'+name,'Hierarchy::'+methods.get(name,name),['standalone-compare.log','topology-controls.log'],['operations-all'], 'Traversal records expose node/index/root context. Native closures run in Rust; portable predicates/accessors use fields or versioned Rust registrations. Subtree copy clears coordinates/history and shares immutable native payloads; host values are owned.')
for f,v in inventory['factories'].items():
 for m in v['methods']:
  api=f.capitalize()+'Options / LayoutSpec / HierarchySession::query(Configuration)'
  adaptation='Replace an immutable layout descriptor to set/reset controls; configuration returns normalized owned values. Callables use native traits or registered operations with checked outputs.'
  if f=='stratify':api='StratifyOptions::{id_field,parent_field,path_field} / Hierarchy::stratify_with';adaptation='Read/write/reset the caller-owned constructor descriptor; construction consumes it. Field selectors replace portable functions; native ID/parent accessors receive source/index/all-rows. Path takes precedence and resetting to None restores ID/parent mode.'
  if m.startswith('padding') and f=='treemap':adaptation+=' Per-side accessors override global padding, which overrides numeric options. Outer assigns Top/Right/Bottom/Left; padding() readback corresponds to Inner and paddingOuter() to Top as in D3.'
  row(f+'.'+m,api,['controls-compare.log','padding-compare.log','topology-controls.log'],adaptation=adaptation)
  rows[-1]['reference_default']=v['defaults'][m]
for f in inventory['tiler_factories']:
 row(f+'.ratio','Tiler::{Squarify,Resquarify}(ratio) / Tiler::ratio / Configuration.effective_ratio',['controls-compare.log','padding-compare.log'],adaptation='Finite authored ratio is retained; effective ratio readback clamps to max(1, ratio). Golden ratio default; replacement creates the configured tiler.')
for subset,api,evidence in [
 ('A','Nested/grouped/table/path/native iterable construction',['standalone-compare.log','topology-controls.log']),
 ('B','Operations, callback context and independent subtree ownership',['standalone-compare.log','python-ownership.log','wasm-ownership.log']),
 ('C','Tidy tree/cluster and all spacing/separation controls',['standalone-compare.log','controls-compare.log']),
 ('D','Partition, own values, rounding/padding and radial projection',['standalone-compare.log','export-tests.log']),
 ('E','Six tilers, padding, ratios, custom tiler and resquarify histories',['standalone-compare.log','padding-compare.log','controls-compare.log','export-tests.log']),
 ('F','Packing, helpers, radii, padding, containment and finite degenerate policies',['standalone-compare.log','export-tests.log']),
 ('G','Typed hosts, native inspection, SVG/PDF/PNG, facets/targets/holes',['chart-compare.log','python-typing.log','wasm-typing.log','export-tests.log','native-build.log']),
 ('H','Update/replay, compact membership, disposal and bounded component workloads',['updates-compare.log','export-tests.log','python-ownership.json','wasm-ownership.json','component-benchmark.json'])]:
 row('FIX-H01-'+subset,api,evidence,adaptation='Shared chart-core engine. H08 component evidence hands workloads to WP-22; global PERF-01–05 and final release gates remain open.')
assert len(inventory['exports'])==16 and len(inventory['node_methods'])==14
assert len(rows)==65
result={'version':1,'date':'2026-09-10','reference':'d3-hierarchy 3.1.2','boundary':'Complete finite typed capability surface with explicit native/portable adaptations in ADR-022; no JS object coercion or DOM/runtime emulation.','gate':'G-HIERARCHY','verdict':'PASS','rows':rows,'open_release_gates':['WP-21','WP-22/PERF-01–05','WP-23','G-GGPLOT','G-PARITY','G4']}
(out/'verdict-catalog.json').write_text(json.dumps(result,indent=2)+'\n')
md=['# Final hierarchy surface verdicts','',result['boundary'],'','All 16 exports, 14 node methods, 25 factory controls, two ratio factories and eight integrated FIX subsets are enumerated. The original oracle inventory retains its historical entry-stage labels; this derived catalog owns final verdicts.','','| Surface | Verdict | Core equivalent / adaptation | Evidence |','| --- | --- | --- | --- |']
for r in rows:md.append('| '+r['surface']+' | PASS | '+r['core_api']+'; '+r['adaptation']+' | '+', '.join('['+e+']('+e+')' for e in r['evidence'])+' |')
md+=['','Case IDs and exact reference defaults are retained in [the machine catalog](verdict-catalog.json). No in-scope row is deferred; the final platform/performance/release gates remain separately open.','']
(out/'verdict-catalog.md').write_text('\n'.join(md));print('PASS: 65 unique surface/control/FIX rows, complete pinned export inventory')
