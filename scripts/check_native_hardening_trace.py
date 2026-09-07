#!/usr/bin/env python3
"""Check FIX-18 native paint observations, separate from CPU or memory timing claims."""
import json
import sys
from pathlib import Path

records = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
assert not any(r['event'] == 'FAILED' for r in records)
before = {r['stage']: r for r in records if r['event'] == 'before-step'}
assert set(before) == set(range(48))
for stage, width in [(0, 698), (1, 698), (2, 478), (3, 10), (4, 798)]:
    c = before[stage]['capture']
    assert c['source'] == c['stamp']['store'] == '0'
    assert c['bounds']['width'] == width
    assert c['profile']['output_theme']['background'] is None
    assert before[stage]['error'] is None
revisions = [int(before[s]['capture']['stamp']['layout']) for s in [1, 2, 3, 4]]
assert revisions == sorted(set(revisions))
assert before[5]['capture']['source'] == '1'
assert before[5]['capture']['profile']['output_theme']['background'] is not None
assert before[6]['capture']['stamp'] == before[5]['capture']['stamp']
assert before[6]['error']['code'] == 'CHART_MISSING_RESOURCE'
assert before[7]['error'] is None
font = next(r['error'] for r in records if r['event'] == 'unavailable-font')
assert font['code'] == 'CHART_INVALID_RESOURCE' and font['context']['resource'] == '999'
last = records[-1]
assert last['event'] == 'after-step' and last['stage'] == 48 and last['released'] == 20
assert last['status'].startswith('PASS:')
print(json.dumps({'status': 'PASS', 'frozen_widths': [698,478,10,798],
                  'frozen_layout_revisions': revisions, 'resumed_source': 1,
                  'recovered_definition': True, 'rejected_font': font['code'],
                  'released_old_entities': 20}, indent=2))
