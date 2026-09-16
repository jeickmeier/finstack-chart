"""GG07 host integration regressions for owned weight and recipe channels."""
import json, sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
d=c.Data.columns({'x':[.5,1.5],'w':[2.,-3.]})
records=[]
for kind in ['bin','count']:
    for expression in [False,True]:
        weight=c.source_expr(d.field('w')) if expression else d.field('w')
        stat=c.bin().x('x').breaks([0.,1.,2.]).ggplot_bin({}).bin_weight(weight) if kind=='bin' else c.count().x('x').ggplot_count().count_weight(weight)
        p=c.plot(d).layer(c.points().stat(stat)).build()
        q=c.Plot.from_json(p.to_json())
        for current in [p,q]:
            chart=current.chart();rows=chart.semantics()['layers'][0]['rows']
            if kind=='bin':values=[row['statistics']['count'] for row in rows['Binned']]
            else:values=[next(v['value'] for v in row['values'] if v['field']=='WeightedCount') for row in rows['Statistical']]
            assert values==[2.,-3.],values
            records.append(values);chart.dispose()
        p.dispose();q.dispose()
(out/'records.json').write_text(json.dumps(records));d.dispose()
print('PASS owned field/expression signed weights through actual host and replay.')
# A non-injective source expression partitions by its result, not its raw reads.
d=c.Data.columns({'x':[1.,1.,1.],'y':[2.,2.,2.],'v':[-1.,1.,2.]})
scale={'training':'Eligible','function':{'GgplotNumericIdentity':{'transform':None,'limits':None,'guide':False,'trained':None}}}
p=c.plot(d).profile('Ggplot2_4_0_3').aes(c.aes().x('x').y('y')).layer(c.points().recipe({'Count':{}}).numeric_scale('Size',c.source_expr(d.field('v')).abs(),scale)).build()
q=c.Plot.from_json(p.to_json());expression_records=[]
for current in [p,q]:
    chart=current.chart();rows=chart.semantics()['layers'][0]['rows']['Statistical'];assert len(rows)==2
    values=sorted((row['retained_numeric'][0][1],next(v['value'] for v in row['values'] if v['field']=='WeightedCount')) for row in rows)
    assert values==[(1.,2.),(2.,1.)],values
    expression_records.append(values);chart.dispose()
(out/'count-expression.json').write_text(json.dumps(expression_records));q.dispose();p.dispose();d.dispose()
print('PASS implicit Count expression-result grouping and replay.')
