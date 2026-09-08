"""Execute FIX-AUTH07 through the real primary PyO3 adapter; no mock runtime."""
import json
import sys
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
sys.path[:0] = [str(ROOT / 'packages/python'), str(Path(sys.argv[1]).resolve())]
import finstack_chart as c

out = Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
def write(name, value): (out / f'{name}.json').write_text(json.dumps(value, indent=2, allow_nan=False))
def error(call, code=None):
    try: call()
    except Exception as e:
        if code:
            assert isinstance(e,c.ChartError), (type(e),e)
            assert e.code == code, (e.code,code)
            assert e.diagnostic['code'] == code and isinstance(e.context,dict) and e.correction
        return
    raise AssertionError('Expected rejected input')
def data(x,y,keys): return c.Data.columns({'x':x,'y':y},keys=keys)

# External views retain one committed source, their own state, names and definition edits.
external_plot = c.plot(data([1.],[2.],[1])).aes(c.aes().x('x').y('y')).layer(c.points()).build()
owner = external_plot.chart(); owner.legend_visible(False)
view = owner.external_view(); sibling = owner.external_view()
error(view.transaction, 'CHART_UNSUPPORTED_CAPABILITY')
owner.commit(owner.transaction().append('data',data([2.],[4.],[2])).build())
assert view.revisions()['store'] == 0
view.accept_from(owner)
assert view.revisions()['store'] == 1 and sibling.revisions()['store'] == 0
view.apply_plot(external_plot.edit().title(c.title('Independent view')).build(),0)
assert view.revisions()['definition'] == 1 and owner.revisions()['definition'] == 0
owner.dispose()
assert sum(len(b['keys']) for b in view.semantics()['datasets'][0]['chunks']) == 2
error(lambda: view.accept_from(owner),'CHART_DISPOSED_HANDLE')
view.dispose(); sibling.dispose()

x = [1.,2.,3.]
source = data(x,[2.,4.,3.],[1001,1002,1003])
x[0] = -999 # Owned materialization must not observe future host mutation.
authored = (c.plot(source).aes(c.aes().x(source.field('x')).y('y'))
    .layer(c.line().name('prices').size(1.5)).layer(c.points().name('observations').size(3.))
    .layer(c.labels().id('peak').at(2.,4.).text('Peak').offset(0.,14.))
    .title(c.title('Primary parity')).x_axis(c.x_axis().label('Time')).y_axis(c.y_axis().label('Value')).build())
chart = authored.chart()
output = c.Output((ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
options = c.export_options(400,260).dpi(96)
import families
families.run(c,ROOT,output,options,write)
import replay
replay.run(c,ROOT,output,write)
replay.run_stream(c,ROOT,output,write)
write('initial',chart.semantics())
error(lambda:chart.request(output,options),'CHART_UNSUPPORTED_CAPABILITY')
assert chart.request(output,options.basis('current')).prepare().export('svg')
frame = chart.present(output,options)
write('scene',frame.scene())
for format in ('png','svg','pdf'): frame.save(out/f'primary.{format}',format)
selected = chart.select_series('observations',limit=64)
assert [t['identity']['Source']['key'] for t in selected['targets']] == ['1001','1002','1003']
write('action',chart.select(selected['targets'],expected=0))
write('zoom',chart.zoom(200,130,2))
error(lambda:chart.legend_visible(False,expected=0),'CHART_REVISION_CONFLICT')
error(lambda:chart.layer_visible('absent',False),'CHART_MISSING_RESOURCE')
editor = chart.editor(c.annotation_edit('peak'))
proposal=editor.preview(0,0); original=editor.original()['annotation']
assert proposal['id']==original['id'] and proposal['text']==original['text']
for axis in ('x','y'): assert abs(proposal['anchor']['Data'][axis]['Number']-original['anchor']['Data'][axis]['Number'])<1e-12
assert editor.nudge()['anchor'] != editor.original()['annotation']['anchor']
presented = chart.request(output,options)
transaction = chart.transaction().id('primary-append').append('data',data([4.],[8.],[1004])).build()
write('transaction',chart.commit(transaction)); write('replay',chart.commit(transaction))
edited = authored.edit().title(c.title('Edited primary')).build()
assert chart.apply_plot(edited,0)
write('final',chart.semantics())
current = chart.request(output,options.basis('current'))
# Captured matrices have independent data/state bases, view policies and transient inclusion.
matrix = []
for basis in ('presented','current'):
    for view in ('visible','full_domain'):
        for selected in (False,True):
            request = chart.request(output,options.basis(basis).view(view).interaction({'selection':selected}))
            matrix.append((basis,view,selected,request))
chart.dispose(); chart.dispose(); output.dispose(); source.dispose(); authored.dispose(); edited.dispose()
error(chart.state,'CHART_DISPOSED_HANDLE')
assert editor.preview(0,0)['id'] == 'peak'
assert b'Primary parity' in presented.prepare().export('svg')
assert b'Edited primary' in current.prepare().export('svg')
for basis,view,selected,request in matrix:
    f=request.prepare(); manifest=f.manifest()
    svg=f.export('svg')
    assert (b'Edited primary' in svg) == (basis=='current')
    assert (manifest['origin_scene'] is not None) == (basis=='presented')
    assert manifest['profile']['view'] == ('VisibleView' if view=='visible' else 'FullDomain')
    assert manifest['interaction_policy']['selection'] == selected
write('matrix',[dict(basis=b,view=v,selection=s,manifest=r.manifest()) for b,v,s,r in matrix])
queue = c.ExportQueue(max_jobs=1)
job = queue.submit(current,'svg')
error(lambda:queue.submit(current,'svg'),'CHART_RESOURCE_LIMIT')
assert job.run().startswith(b'<')
error(job.run,'CHART_DISPOSED_HANDLE')
job = queue.submit(current,'svg'); assert job.cancel()
error(job.run,'CHART_CANCELLED')
queue.dispose(); error(lambda:queue.submit(current,'svg'),'CHART_DISPOSED_HANDLE')
exact = c.Data.columns({'t':c.timestamps([2**63-2,2**63-1]), 'unsigned':[2**64-2,2**64-1],
                       'signed':[-2**63,2**63-1], 'nullable':c.column([1.25,None]), 'group':c.categorical(['b','a'])},keys=[2**64-2,2**64-1])
exact_plot = c.plot(exact).aes(c.aes().x(1.).y('nullable')).layer(c.points()).build()
write('exact',json.loads(exact_plot.to_json()))
roundtrip=c.Plot.from_json(exact_plot.to_json());assert json.loads(roundtrip.to_json())==json.loads(exact_plot.to_json())
statistics = (c.plot(data([1.,2.,3.,4.],[2.,4.,6.,8.],[11,12,13,14])).aes(c.aes().x('x').y('y'))
    .layer(c.points().name('summary').stat(c.summary().x('y')).after_stat(c.stat_aes().x(1.).y('Mean')))
    .layer(c.line().name('fit').stat(c.fit()).after_stat(c.stat_aes().x('X').y('Y'))).build())
write('statistics',statistics.chart().semantics())
# Ordinary rows/accessors are materialized once; no callable survives into Rust.
calls=[]
rows=c.Data.rows([{'a':1.},{'a':2.}],fields={'x':lambda row:(calls.append(row['a']),row['a'])[1]},keys=lambda row:int(row['a']))
assert calls==[1.,2.]
rp=c.plot(rows).aes(c.aes().x('x').y('x')).layer(c.points()).build();rp.chart().semantics();assert calls==[1.,2.]
for call in (lambda:c.column([True,1]), lambda:c.column([1,None],kind='bool'), lambda:c.column([2**64]), lambda:c.column([-1],kind='uint64'), lambda:c.column([2**53+1,0.5]), lambda:c.column([None]), lambda:c.Data.columns({'x':[1],'y':[2,3]}), lambda:c.Data.columns({'x':[1]},keys=[True]), lambda:c.Data.rows([{'x':1},{'y':2}]), lambda:c.ExportQueue(max_jobs=-1), lambda:c.Output([256]), lambda:c.column([1]).nullable('yes')): error(call)
error(lambda:c.labels().title('Wrong'),'CHART_UNSUPPORTED_CAPABILITY')
error(lambda:c.plot(rows).title(c.labels()),'CHART_UNSUPPORTED_CAPABILITY')
foreign=c.Data.columns({'x':[1.]});error(lambda:c.plot(rows).aes(c.aes().x(foreign.field('x')).y('x')).layer(c.points()).build(),'CHART_SCHEMA_CONFLICT')
# Exact event-time boundaries and explicit watermark use the core store policy.
times=c.Data.columns({'t':c.timestamps([1,2,3]),'y':[1.,2.,3.]},keys=[1,2,3]);tp=c.plot(times).aes(c.aes().x(1.).y('y')).layer(c.points()).build();tc=tp.chart()
receipt=tc.commit(tc.transaction().retain_event_time('data','t',1,3).build())
assert receipt['Applied']['operations'][0]['evicted']==1
assert tc.semantics()['datasets'][0]['chunks'][0]['keys']==['2','3']
# Source metadata is independent of numeric payloads and valid zero/false values.
metadata=c.Data.columns({'x':c.column([1.25,None]).formatted(['1.2500',None]).unit('USD').label('Net'), 'flag':c.column([False,None],kind='bool')})
mp=c.plot(metadata).aes(c.aes().x(1.).y('x')).layer(c.points()).build()
mb=json.loads(mp.to_json())['data'][0]['batch']
assert mb['fields'][0]['unit']=='USD' and mb['fields'][0]['label']=='Net'
assert mb['columns'][0]['formatted']==['1.2500',None]
assert mb['columns'][1]['values']['Boolean']==[False,False] and mb['columns'][1]['validity']==[True,False]
# Acceptance and source commit remain distinct, including stale queued work and batch equivalence.
base=c.plot(data([1.,2.,3.],[2.,4.,3.],[1001,1002,1003])).aes(c.aes().x('x').y('y')).layer(c.line()).build()
live=base.chart().stream(c.stream_options().transactions(1))
t1=live.transaction().id('queue-append').append('data',data([4.],[8.],[1004])).build()
t2=live.transaction().id('queue-stale').remove('data',[1001]).build()
assert live.enqueue(t1)=='Queued' and live.enqueue(t1)=='AlreadyQueued'
assert live.enqueue(t2)=='Backpressure' and live.revisions()['store']==0
assert 'Applied' in live.commit_next()['outcome'] and live.revisions()['store']==1
assert live.enqueue(t2)=='Queued' and 'Conflict' in live.commit_next()['outcome']
assert live.queue_status()['committed']=='1' and live.queue_status()['failed']=='1'
batch=c.plot(data([1.,2.,3.,4.],[2.,4.,3.,8.],[1001,1002,1003,1004])).aes(c.aes().x('x').y('y')).layer(c.line()).build().chart()
assert live.semantics()['layers'][0]['domains']==batch.semantics()['layers'][0]['domains']
assert live.commit(live.transaction().retain_count('data',2).build())['Applied']['operations'][0]['evicted']==2
assert [k for b in live.semantics()['datasets'][0]['chunks'] for k in b['keys']]==['1003','1004']
# Compiled extensions use primary components and checked field parameters, not manual IDs.
from finstack_chart import examples as ex
custom_source=data([0.25,0.75,1.25,1.75],[1.,1.,1.,1.],[201,202,203,204])
node=c.transform('density',ex.density_histogram(custom_source.field('x'),[0.,1.,2.]))
def custom_plot(native=False):
    return (ex.with_extensions(c.plot(custom_source)).transform(node)
        .layer(ex.chamfered_bars(native).name('density-bars').from_transform(node))
        .layer(c.points().name('density-points').from_transform(node).after_stat(c.stat_aes().x({'Custom':'left'}).y({'Custom':'density'}))).build())
custom=custom_plot();write('extension',custom.chart().semantics())
custom_output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
cf=custom_output.request(custom,options).prepare();write('extension-scene',cf.scene());cf.save(out/'extension.png','png')
assert json.loads(ex.load_plot(custom.to_json()).to_json())==json.loads(custom.to_json())
error(lambda:c.Plot.from_json(custom.to_json()))
native_only=custom_plot(True);error(native_only.chart,'CHART_UNSUPPORTED_CAPABILITY');error(native_only.to_json,'CHART_UNSUPPORTED_CAPABILITY');error(lambda:custom_output.request(native_only,options).prepare(),'CHART_UNSUPPORTED_CAPABILITY')
foreign_stat=ex.density_histogram(c.Data.columns({'x':[0.25]}).field('x'),[0.,1.,2.])
error(lambda:ex.with_extensions(c.plot(custom_source)).layer(ex.chamfered_bars().stat(foreign_stat)).build(),'CHART_SCHEMA_CONFLICT')
# Independent thread makes progress specifically during the Rust-only semantic call.
large=c.Data.columns({'x':[float(i) for i in range(100000)],'y':[float(i%19) for i in range(100000)]})
large_chart=c.plot(large).aes(c.aes().x('x').y('y')).layer(c.points()).build().chart()
stop=threading.Event(); progress=[0]
def worker():
    while not stop.wait(0.001): progress[0]+=1
thread=threading.Thread(target=worker);thread.start()
try:
    before=progress[0]; start=time.perf_counter(); result=large_chart._inner.semantics(); elapsed=time.perf_counter()-start
    detached=progress[0]-before
finally: stop.set();thread.join()
assert detached>5,(detached,elapsed)
write('host_checks',{'python_detached_thread_steps':detached,'semantics_seconds':elapsed,'source_rows':100000})
print('PASS primary Python exact data, copy/accessor ownership, actions/queries, stale fences, edits, capture matrix, queues, disposal, retention and interpreter detachment')
