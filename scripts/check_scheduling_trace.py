#!/usr/bin/env python3
"""Validate retained native FIX-11 accounting, independent of timing claims."""
import json
import sys
from pathlib import Path

records = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines() if line.startswith('{')]
snapshots = [record for record in records if 'charts' in record]
assert snapshots and any(r['event'] == 'render' and r['active'] for r in records)
arrivals = [r for r in snapshots if r['event'] == 'arrival']
assert [r['tick'] for r in arrivals] == list(range(1, 161))
assert len(arrivals[0]['charts']) == 4
result = []
for i in range(4):
    presented = []
    for record in snapshots:
        chart = record['charts'][i]
        state = chart['schedule']
        assert state['active'] in (0, 1) and state['pending'] in (0, 1)
        assert chart['error'] is None and int(state['failed']) == 0
        shown = int(chart['presented_store'])
        assert shown <= record['tick']
        presented.append(shown)
    assert presented == sorted(presented)
    progressing = {int(r['charts'][i]['presented_store']) for r in arrivals if r['tick'] < 160}
    assert len(progressing) >= 10
    drained = [r['charts'][i] for r in snapshots if r['event'] == 'drain'][-1]
    state = drained['schedule']
    assert int(state['committed']) == int(state['presented']) == 160 and int(state['lag']) == 0
    assert state['active'] == state['pending'] == 0
    assert int(state['coalesced']) > 0 and int(state['stale']) >= 2
    assert snapshots[-1]['event'] == 'Dispose workers'
    assert snapshots[-1]['charts'][i]['schedule']['disposed']
    result.append({'chart': i, 'observed_presented_revisions': sorted(set(presented)), 'final_before_disposal': state})
print(json.dumps({'status': 'PASS', 'scope': 'four native charts; 160 committed updates; actual painted revision observations; viewport/theme invalidation; drained disposal', 'charts': result}, indent=2))
