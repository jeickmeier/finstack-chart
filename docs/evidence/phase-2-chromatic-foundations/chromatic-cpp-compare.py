from pathlib import Path
import json,subprocess
p=json.loads(Path('fixtures/parity/d3-scale-chromatic/trig.json').read_text());vals=[r for r in p['cases'] if isinstance(r[0],(float,int)) and -1000<r[0]<1000][:2200]
res=subprocess.check_output(['/private/tmp/chromatic-v8-probe-fma',*[str(v[0]) for v in vals]],text=True).splitlines()
errors=[]
for row,line in zip(vals,res):
 x,a,std=map(float,line.split())
 if a!=row[1]:errors.append([x,a,row[1]])
print('FMA reference probe mismatches',len(errors),errors[:3])
