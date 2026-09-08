"""Primary builders replay the independent action and input fixtures without chart envelopes."""
import json
from families import dataset

def subset(actual, expected):
    if isinstance(expected,dict):
        for key,value in expected.items(): subset(actual[key],value)
    elif isinstance(expected,list):
        assert len(actual)==len(expected),(actual,expected)
        for a,b in zip(actual,expected): subset(a,b)
    elif isinstance(expected,(int,float)) and not isinstance(expected,bool):
        assert abs(actual-expected)<=1e-12*max(abs(expected),1),(actual,expected)
    else: assert actual==expected,(actual,expected)

def author(c,case,actions=False):
    source=dataset(c,case)
    if actions:
        return (c.plot(source).aes(c.aes().x('x').y('y'))
            .layer(c.histogram().breaks([0.,1.,2.]).color('#b4c8d78c')).layer(c.line()).layer(c.points())
            .layer(c.labels().id('threshold').figure_at(.1,.1).text('Threshold')).build())
    name=case['name']
    layer=c.bars().width(20.) if name=='input-bars' else c.line() if name in ('input-line','input-utc') else c.points()
    p=c.plot(source).aes(c.aes().x('field_1').y('field_2')).layer(layer.name('observations').color('#1e7db4d2'))
    if name in ('input-category','input-host-category-link'):
        p=p.x_axis(c.x_axis().visible(False).scale(c.scale_point().categories(['Alpha','Beta','Gamma']).point_padding(.5))).y_axis(c.y_axis().visible(False))
    elif name=='input-utc':
        p=p.aes(c.aes().x({'field':'field_1','origin':'1709164800000'}).y('field_2')).x_axis(c.x_axis().visible(False).scale(c.scale_utc().interval({'Days':1}))).y_axis(c.y_axis().visible(False))
    else:
        p=p.x_axis(c.x_axis().visible(False).scale(c.scale_linear().domain(0.,4.))).y_axis(c.y_axis().visible(False).scale(c.scale_linear().domain(0.,4.)))
    if name=='input-host-tools':
        p=p.layer(c.callout().label(c.labels().id('Threshold').at(.5,3.).text('Threshold').offset(0.,-20.)).to_data(3.5,3.).connector_origin('Anchor'))
    return p.build()

def remapper(case,plot,chart):
    envelope=json.loads(plot.to_json())
    pairs={'epoch':{case['data']['epoch']:str(chart.revisions()['epoch'])},'dataset':{case['data']['datasets'][0]['id']:envelope['data'][0]['id']},'layer':{a['id']:b['id'] for a,b in zip(case['chart']['definition']['layers'],envelope['definition']['layers'])}}
    def remap(value,key='',reverse=False):
        if isinstance(value,dict): return {k:remap(v,k,reverse) for k,v in value.items()}
        if isinstance(value,list): return [remap(v,key,reverse) for v in value]
        if isinstance(value,str):
            mapping=pairs.get(key,{})
            if reverse: mapping={v:k for k,v in mapping.items()}
            value=mapping.get(value,value)
            if key=='description':
                a,b=next(iter(pairs['dataset'].items()))
                if reverse:a,b=b,a
                value=value.replace(f'in dataset {a}',f'in dataset {b}')
        return value
    return remap

def run(c,root,output,write):
    read=lambda p:json.loads((root/p).read_text())
    options=c.export_options(400,200).layout(c.layout_options().padding(0.).font_size(10.))
    def checked(call,step):
        try:
            result=call();assert 'error' not in step,step['name'];return result
        except c.ChartError as e:
            assert e.code==step.get('error'),(step['name'],e.code,e.message);return {'error':e.code}
    inputs=[]
    for case in read('fixtures/interaction/cases.json')+read('fixtures/host-tools/cases.json'):
        plot=author(c,case);chart=plot.chart();remap=remapper(case,plot,chart)
        stamp=chart.present(output,options).scene()['stamp'];basis=None;initial=chart.semantics()
        for raw in case['queries']:
            step=remap(raw);before=chart.state();result=None
            if 'query' in step:
                query_stamp=dict(basis if step.get('gesture') else stamp)
                if step.get('stale'):query_stamp['layout']='999999'
                result=checked(lambda:chart.query(step['query'],gesture=step.get('gesture',False),stamp=query_stamp),step)
                assert chart.state()==before
                if 'expect' in step:subset(result,step['expect'])
            action=step.get('action');apply=step.get('apply')
            if apply=='annotation_preview':action={'PreviewGesture':{'id':step['id'],'preview':{'Annotation':result['annotation']}}}
            elif apply=='linked':action=result['action']
            elif apply=='windows':action={'SetAxisWindows':result['windows']}
            elif apply=='preview':action={'PreviewGesture':{'id':step['id'],'preview':{'AxisWindows':result['windows']}}}
            elif apply=='targets':action={'Select':{'change':step.get('change','Replace'),'targets':result['targets']}}
            if action is not None:
                if 'BeginGesture' in action:basis=dict(stamp)
                chart.act(action,origin=result['origin'] if apply=='linked' else 'Pointer')
                if 'CancelGesture' in action or 'CommitGesture' in action:basis=None
            after=chart.state()
            if 'state' in step:subset(after,step['state'])
            if step.get('present'):stamp=chart.present(output,options).scene()['stamp']
            semantic=chart.semantics();assert semantic['datasets']==initial['datasets']
            for a,b in zip(semantic['layers'],initial['layers']):assert a['rows']==b['rows'] and a['domains']==b['domains']
            inputs.append(remap(dict(case=case['name'],name=step['name'],result=result,state=after),reverse=True))
        chart.dispose()
    case=dict(chart=read('fixtures/actions/chart.json'),data=read('fixtures/bindings/data.json'))
    plot=author(c,case,True);chart=plot.chart();remap=remapper(case,plot,chart);chart.present(output,options);actions=[]
    for raw in read('fixtures/actions/trace.json'):
        step=remap(raw);before=chart.state()
        result=checked(lambda:chart.act(step['action'],origin=step.get('origin','Control'),expected=int(step['expected_state']) if 'expected_state' in step else None),step)
        after=chart.state()
        if 'expect' in step:subset(result,step['expect'])
        if 'state' in step:subset(after,step['state'])
        if 'error' in step:assert before==after
        if step.get('present'):chart.present(output,options)
        actions.append(remap(dict(name=step['name'],result=result,state=after),reverse=True))
    chart.dispose()
    assert len(actions)==23 and len(inputs)==47
    write('runtime-actions',actions);write('runtime-input',inputs)
    print('PASS primary Python 23 action transitions and 47 independent input steps')

def run_stream(c,root,output,write):
    case=json.loads((root/'fixtures/streaming/replay.json').read_text())
    p=(c.plot(dataset(c,case))
        .transform(c.transform('bins',c.bin().x('value').breaks([0.,10.,20.,30.,50.])))
        .transform(c.transform('summary',c.summary().x('value').quantiles([0.,.5,1.]).empty_sum_zero(False)))
        .layer(c.points().aes(c.aes().x({'field':'event_time','origin':'9007199254741001'}).y('value')).color('#1e7db4d2')).build())
    chart=p.chart();remap=remapper(case,p,chart)
    options=c.export_options(400,200).layout(c.layout_options().padding(0).font_size(10))
    def present():
        frame=chart.present(output,options);frame.dispose()
    present();cache={};trace=[]
    def transaction(wire):
        if wire['id'] in cache:return cache[wire['id']]
        b=chart.transaction().id(wire['id'])
        for operation in wire['operations']:
            kind,v=next(iter(operation['mutation'].items()))
            if kind in ('AppendBatch','UpsertByKey','ReplaceSnapshot'):
                data=dataset(c,{'data':{'datasets':[{'id':'1','batch':v}]}})
                b=getattr(b,{'AppendBatch':'append','UpsertByKey':'upsert','ReplaceSnapshot':'replace'}[kind])('data_0',data)
                data.dispose()
            elif kind=='RemoveKeys':b=b.remove('data_0',[int(k) for k in v])
            elif kind=='AdvanceWatermark':b=b.watermark('data_0',int(v))
            elif kind=='SetRetention':
                if 'EventTime' in v:
                    e=v['EventTime'];b=b.retain_event_time('data_0','event_time',int(e['width']),int(e['watermark']),allowed_lateness=int(e['allowed_lateness']),late=e['late'])
                else:b=b.retain_count('data_0',v.get('Count') if isinstance(v,dict) else None)
            else:raise AssertionError(kind)
        result=b.build();cache[wire['id']]=result;return result
    for raw in case['steps']:
        step=remap(raw)
        try:
            if 'transaction' in step:result=chart.commit(transaction(step['transaction']))
            elif 'stream' in step:
                stream=step['stream']
                if isinstance(stream,dict) and 'ConfigureQueue' in stream:
                    v=stream['ConfigureQueue'];chart.stream(c.stream_options().transactions(v['transactions']).rows(v['rows']).bytes(v['bytes']).overload(v['overload']));result=chart.stream_status()['limits']
                elif isinstance(stream,dict) and 'Enqueue' in stream:result=chart.enqueue(transaction(stream['Enqueue']))
                else:result=getattr(chart,{'Status':'stream_status','Pinned':'pinned','CommitNext':'commit_next'}[stream])()
            else:result=chart.act(step['action'],origin='Control')
            assert 'error' not in step,step['name']
        except c.ChartError as e:
            assert e.code==step.get('error'),(step['name'],e.code,e.message);result={'error':e.code}
        if 'expected' in step:subset(result,step['expected'])
        semantic=chart.semantics();state=chart.state()
        assert semantic['store_revision']==step['revision']
        if step.get('present'):present()
        trace.append(dict(name=step['name'],result=remap(result,reverse=True),state=remap(state,reverse=True),semantics=semantic))
    for t in cache.values():t.dispose()
    chart.dispose();p.dispose();assert len(trace)==70
    write('runtime-stream',trace);print('PASS primary Python 70-step streaming replay')
