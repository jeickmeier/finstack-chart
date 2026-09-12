#!/usr/bin/env python3
"""Compare actual-host replay metadata with hierarchy-family tolerances."""
import json,pathlib,sys
from compare import near
rows=[json.loads(pathlib.Path(p).read_text()) for p in sys.argv[1:]]
assert len(rows)>=2 and all(len(r)==18 for r in rows)
for other in rows[1:]:
 for a,b in zip(rows[0],other):
  assert a['replay_equal'] and a['old_snapshot_equal'] and a['batch_equal']==(a['family']!=5)
  near(a,b,*( (1e-8,1e-10) if a['family']==3 else (1e-10,1e-12) ))
print('PASS actual Python/WASM replay metadata: 18 traces, exact identities and family-specific numeric tolerances')
