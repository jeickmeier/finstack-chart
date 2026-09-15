"""FIX-GG04: exact timestamp inputs, temporal default guides and edits in Python."""
import json, math, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes());options=c.export_options(640,360).dpi(96).basis('current')
cases=json.loads((ROOT/'fixtures/parity/ggplot2/temporal-aesthetic-defaults.json').read_text())['cases'];records=[]
for index,case in enumerate(cases):
 for unit,multiplier in [('s',1),('ms',1000),('us',1000000),('ns',1000000000)]:
  owned=[]
  try:
   values=case['inputs'];factor=multiplier*(86400 if case['kind']=='date' else 1);channel=case['channel'];expected=case['result']
   data=c.Data.columns({'x':c.column(list(map(float,range(len(values)))),kind='float64'),'y':c.column(list(map(float,range(len(values)))),kind='float64'),'when':c.timestamps([int(v or 0)*factor for v in values],unit,'UTC').validity([v is not None for v in values])});owned.append(data)
   mapping=getattr(c.aes().x('x').y('y'),'color' if channel=='colour' else channel)('when')
   plot=c.plot(data).profile('Ggplot2_4_0_3').aes(mapping).layer(c.points().name('marks').stroke('#000000')).build();owned.append(plot)
   # Automatic paint provenance and numeric theme selection have distinct envelopes.
   wire=plot.to_json();assert json.loads(wire)['version']==(42 if channel in ('colour','fill') else 45)
   restored=c.Plot.from_json(wire);owned.append(restored);assert restored.to_json()==wire
   base_encoding=None
   for state in ('original','edited'):
    current=restored if state=='original' else restored.edit().layer('marks',c.points().stroke('#000000')).build()
    if current is not restored:owned.append(current)
    chart=current.chart();owned.append(chart);layer=chart.semantics()['layers'][0];assert 'error' not in expected,(index,expected)
    styles=layer.get('styles',[]);wanted=expected['values'];target={'size':'Size','alpha':'Alpha','linewidth':'StrokeWidth'}.get(channel)
    if target and values:
     encoding=layer['numeric_scales'][target]
     if base_encoding is None:base_encoding=encoding
     else:assert encoding==base_encoding,(index,unit)
    if channel in ('size','linewidth'):wanted=[v for v in wanted if v is not None]
    assert len(styles)==len(wanted),(index,unit,len(styles),len(wanted))
    for style,value in zip(styles,wanted):
     if channel=='alpha':
      assert style['color']['alpha']==(255 if value is None else round(value*255)),(index,style,value)
     elif target:
      actual=style.get({'size':'radius','alpha':'alpha','linewidth':'stroke_width'}[channel])
      assert (actual is None)==(value is None),(index,style,value)
      if value is not None:assert math.isclose(actual,value,rel_tol=0,abs_tol=2e-12),(index,style,value)
     else:
      paint=style['fill' if channel=='fill' else 'color'];rgb=[127,127,127] if value=='grey50' else [int(value[i:i+2],16) for i in (1,3,5)]
      assert [paint[k] for k in ('red','green','blue')]==rgb and paint['alpha']==255,(index,paint,value)
    legend=layer.get('paint_legends',{}).get('Fill') if channel=='fill' else layer.get('color_legend')
    guides=[] if not legend else legend.get('numeric_breaks',[])
    if channel in ('colour','fill') and expected['guide']:
     assert [v['label'] for v in guides]==expected['guide']['labels'],(index,guides)
    records.append({'index':index,'unit':unit,'state':state,'styles':styles,'guides':guides})
    if unit=='s' and state=='original' and case['kind']=='datetime' and case['population']=='spaced' and channel in ('size','alpha','colour','fill'):
     request=output.request(current,options);owned.append(request);frame=request.prepare();owned.append(frame)
     for fmt in ('svg','pdf','png'):(out/f'temporal-{channel}.{fmt}').write_bytes(frame.export(fmt))
  except c.ChartError as error:
   diagnostic=json.loads(str(error));assert 'error' in expected,(index,unit,diagnostic)
   assert diagnostic['code']=='CHART_NUMERICAL_DOMAIN',(index,diagnostic)
   records.append({'index':index,'unit':unit,'error':diagnostic['code']})
  finally:
   for value in reversed(owned):value.dispose()
assert len(records)==440
(out/'records.json').write_text(json.dumps(records,indent=2));options.dispose();output.dispose()
print('PASS Python: 440 temporal states and twelve publication files.')
