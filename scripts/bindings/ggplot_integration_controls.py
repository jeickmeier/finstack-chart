"""GG18 independent primary combined grammar authors."""
import sys,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
registry=c.ExtensionRegistry.example();output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
def build(data,mode=0):
    method={'Registered':{'operation':{'id':'example.prescribed_slope','version':'1'},'parameters':{'slope':.3,'envelope':.5}}} if mode==3 else 'Linear'
    model=c.smooth().stat(c.model_stat(dict(method=method,n=25,terms=None,xseq=None,full_range=False,se=True,level=.95)).x('x').y('y').group('g'))
    if mode==1:model=model.filter(c.filter('x').minimum(4.))
    builder=c.plot(data).with_registry(registry).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y').color('g').color_scale('groups')).scale(c.color_discrete('groups').domain(['a','b'])).layer(c.points().key_glyph('example.diamond_key',1,dict(padding=.1))).layer(model).theme(c.theme().reference_preset('Bw',{})).title(c.title('Geographic model' if mode==3 else 'Combined grammar')).subtitle(c.subtitle('').rich(c.math_text('frac(1,2)+sqrt(4)',{}))).tag(c.rich_text('B' if mode==3 else 'A')).legend(c.legend().scale('groups').registered('example.strip_guide',1,{})).facet(c.facet_wrap('g').registered('example.reverse_facets',1,dict(columns=2)))
    builder=builder.coordinate({'Geographic':{'projection':{'Crs':'WebMercator'},'default_crs':'Wgs84'}}) if mode==3 else builder.registered_coordinate('example.wave_coordinate',1,dict(amplitude=.06))
    if mode==2:builder=builder.coordinate(dict(Cartesian=dict(xlim=[dict(Number=4.),dict(Number=11.)])))
    return builder.build()
for mode in range(4):
    data=c.Data.columns(dict(x=[float(i//2) for i in range(24)],y=[2.+.3*(i//2)+(i//2)%3+i%2 for i in range(24)],g=['a' if i%2==0 else 'b' for i in range(24)]),keys=[9007199254740993+i for i in range(24)])
    plot=build(data,mode);wire=plot.to_json();restored=c.Plot.from_json(wire,registry);request=output.request(restored,c.export_options(600,360).dpi(144));frame=request.prepare()
    with output.request(plot,c.export_options(600,360).dpi(144)) as original_request:
        with original_request.prepare() as original_frame:assert original_frame.scene()==frame.scene()
    (out/f'integration-{mode}.plot.json').write_text(wire);(out/f'integration-{mode}.scene.json').write_text(json.dumps(frame.scene()))
    for fmt in ['svg','pdf','png']:(out/f'integration-{mode}.{fmt}').write_bytes(frame.export(fmt))
    frame.dispose();request.dispose();restored.dispose();plot.dispose();data.dispose()
def materialize(rows):
    return c.Data.columns(dict(x=[r[1] for r in rows],y=[r[2] for r in rows],g=[r[3] for r in rows]),keys=[r[0] for r in rows])
rows=[(9007199254740993+i,float(i//2),2.+.3*(i//2)+(i//2)%3+i%2,'a' if i%2==0 else 'b') for i in range(24)]
source=materialize(rows);authored=build(source);live=authored.chart();options=c.export_options(600,360).dpi(144).basis('current');old=live.request(output,options);old_svg=old.prepare().export('svg')
def check_batch():
    batch_data=materialize(rows);batch=build(batch_data)
    with live.request(output,options) as request, output.request(batch,options) as expected:
        with request.prepare() as left, expected.prepare() as right:
            assert [i['primitive'] for i in left.scene()['items']]==[i['primitive'] for i in right.scene()['items']]
    batch.dispose();batch_data.dispose()
new=[(9007199254741017,12.,9.,'a'),(9007199254741018,12.,10.,'b')]
assert 'Applied' in live.commit(live.transaction().append(source,materialize(new)).build());rows.extend(new);check_batch()
rows[0]=(rows[0][0],0.,20.,'a');assert 'Applied' in live.commit(live.transaction().upsert(source,materialize(rows[:1])).build());check_batch()
assert 'Applied' in live.commit(live.transaction().remove(source,[9007199254740994]).retain_count(source,20).build());rows=[r for r in rows if r[0]!=9007199254740994][-20:];check_batch()
assert 'Rejected' in live.commit(live.transaction().append(source,materialize(rows[:1])).build());check_batch()
# Capture a presented derived path and source identity, then leave a newer definition/source/view pending.
live.act({'SetViewport':{'x':[4.,8.],'y':None}})
frame=live.present(output,options)
targets=live.select_region({'Rectangle':[0.,0.,600.,360.]})['targets']
derived=next(t for t in targets if 'Derived' in t['identity']);source_target=next(t for t in reversed(targets) if 'Source' in t['identity'])
live.select([derived,source_target]);live.hover([derived]);frame.dispose();frame=live.present(output,options)
painted=frame.manifest();assert len(painted['state']['interaction']['selection'])==2 and len(painted['state']['hover'])==1
extra=materialize([(9007199254741019,13.,11.,'a')]);assert 'Applied' in live.commit(live.transaction().append(source,extra).build());extra.dispose()
edited=authored.edit().title(c.title('Pending title')).build();live.apply_plot(edited,live.revisions()['definition']);edited.dispose()
live.act({'SetViewport':{'x':[5.,9.],'y':None}})
with live.request(output,options) as current_request:
    with current_request.prepare() as current_frame:current=current_frame.manifest()
assert int(current['stamp']['store'])>int(painted['stamp']['store']);assert int(current['stamp']['definition'])>int(painted['stamp']['definition'])
matrix=[]
for basis in ('presented','current'):
    for view in ('visible','full_domain'):
        for selected in (False,True):matrix.append((basis,view,selected,live.request(output,options.basis(basis).view(view).interaction({'selection':selected,'hover':selected}))))
live.dispose();source.dispose();authored.dispose();frame.dispose();output.dispose();registry.dispose()
with old.prepare() as old_frame:assert old_frame.export('svg')==old_svg
old.dispose();records=[]
for basis,view,selected,request in matrix:
    with request.prepare() as frame:
        manifest=frame.manifest();expected=painted if basis=='presented' else current
        assert (manifest['origin_scene'] is not None)==(basis=='presented')
        assert manifest['stamp']['store']==expected['stamp']['store'] and manifest['stamp']['definition']==expected['stamp']['definition']
        assert manifest['state']==expected['state'];assert manifest['state']['interaction']['selection']==[derived,source_target] or {json.dumps(t,sort_keys=True) for t in manifest['state']['interaction']['selection']}=={json.dumps(t,sort_keys=True) for t in [derived,source_target]}
        effective=manifest['effective_state'];assert len(effective['interaction']['selection'])==(2 if selected else 0);assert len(effective['hover'])==(1 if selected else 0)
        assert effective['viewport']['x']==(expected['state']['viewport']['x'] if view=='visible' else None)
        assert ('Pending title' in frame.export('svg').decode())==(basis=='current')
        records.append(dict(basis=basis,view=view,include=selected,store=manifest['stamp']['store'],definition=manifest['stamp']['definition'],selected=len(effective['interaction']['selection']),hover=len(effective['hover']),viewport=effective['viewport']))
    request.dispose()
(out/'capture-matrix.json').write_text(json.dumps(records))
print('PASS Python integration: publications, updates/batch, rollback, derived/source selection and hover, revised capture matrix and disposal')
