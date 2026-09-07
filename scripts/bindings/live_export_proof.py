"""Execute the canonical held-capture trace through the real Python extension."""
import json

def run(Chart,ChartError,root,output):
    fixture=json.loads((root/'fixtures/live-export/replay.json').read_text())
    chart=Chart(json.dumps(fixture['chart']),json.dumps(fixture['data']),(root/'fixtures/live-export/profile.json').read_text(),(root/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
    stamp=json.loads(chart.present())['stamp'];jobs={};outputs={};trace=[];disposed=False
    for step in fixture['steps']:
        name=step['name'];kind=step['kind']
        try:
            if kind=='action':
                state=json.loads(chart.state());result=json.loads(chart.dispatch(json.dumps(dict(definition_revision=state['definition_revision'],expected_state=state['state_revision'],origin='Control',scene=stamp,action=step['action']))))
            elif kind=='transaction':result=json.loads(chart.transaction(json.dumps(step['transaction'])))
            elif kind=='present':stamp=json.loads(chart.present())['stamp'];result={'stamp':stamp}
            elif kind=='begin':
                result=json.loads(chart.export_control(json.dumps({'version':1,'operation':{'Begin':step['options']}})));jobs[name]=result['job']
            elif kind=='cancel':result=json.loads(chart.export_control(json.dumps({'version':1,'operation':{'Cancel':{'job':jobs[step['job']]}}})))
            elif kind=='export':outputs[name]=bytes(chart.export_job(jobs[step['job']]));result={'bytes':len(outputs[name])}
            elif kind=='status':result=json.loads(chart.export_control('{"version":1,"operation":"Status"}'))
            elif kind=='dispose':chart.dispose();disposed=True;result={'disposed':True}
            else:raise AssertionError(kind)
            assert 'error' not in step,name
        except ChartError as error:
            code=json.loads(error.args[0])['code'];assert code==step.get('error'),(name,code);result={'error':code}
        status=None if disposed else json.loads(chart.export_control('{"version":1,"operation":"Status"}'))
        live=None if disposed else dict(store=json.loads(chart.semantics())['store_revision'],state=json.loads(chart.state()))
        trace.append(dict(name=name,result=result,status=status,live=live))
    for name,data in outputs.items():assert b'<svg' in data;(output/f'live-{name}.svg').write_bytes(data)
    (output/'live-export.json').write_text(json.dumps(trace))
    print(f'PASS Python FIX-14 {len(trace)} held-capture steps and bytes after disposal')
