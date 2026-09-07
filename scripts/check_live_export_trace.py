#!/usr/bin/env python3
"""Check the finite native FIX-14 trace; this is not the PERF-03/05 gate."""
import json
import sys
from pathlib import Path

rows = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
commits = [r for r in rows if r['event'] == 'commit']
assert [r['tick'] for r in commits] == list(range(1, 401))
assert all(int(r['committed']) == r['tick'] for r in commits)
captures = {r['job']: r for r in rows if r['event'] == 'capture'}
completed = {r['job']: r for r in rows if r['event'] == 'export-complete'}
errors = {r['job']: r for r in rows if r['event'] == 'export-error'}
assert set(captures) == {'0', '1', '2', '3'}
assert set(completed) == {'0', '1', '3'}
assert set(errors) == {'2'} and errors['2']['error']['code'] == 'CHART_CANCELLED'
for capture in captures.values():
    a, b = capture['source_pair']
    assert a + b == 0
    m = capture['manifest']
    assert all(d['version']['revision'] == m['store'] for d in m['datasets'])
for job, result in completed.items():
    captured = captures[job]['manifest']
    m = result['metadata']
    assert m['stamp']['store'] == captured['store']
    assert m['definition'] == captured['definition']
    assert m['profile'] == captured['profile']
    assert m['fonts'] == captured['fonts']
    assert all(d[0]['revision'] == captured['store'] for d in m['datasets'])
    assert result['bytes'] > 0
for job in ['0', '1']:
    assert captures[job]['manifest']['store'] == '0'
    assert completed[job]['tick'] >= 70
    assert completed[job]['metadata']['effective_state']['effective_annotations'][0]['text']['lines'][0][0]['text'] == 'Before capture'
assert completed['3']['metadata']['stamp']['store'] == '400'
assert completed['3']['metadata']['profile']['view'] == 'FullDomain'
assert any(r['event'] == 'capacity-rejected' for r in rows)
for r in rows:
    if 'exports' in r:
        m = r['exports']
        assert m['pending'] + m['running'] <= 2 and m['peak_jobs'] <= 2
last = rows[-1]
assert last['event'] == 'Dispose'
m = last['exports']
assert m['disposed'] and (m['pending'], m['running'], m['rows'], m['input_bytes']) == (0, 0, 0, 0)
assert (m['submitted'], m['completed'], m['cancelled'], m['failed'], m['rejected']) == ('4', '3', '1', '0', '1')
timings = []
for job, result in completed.items():
    phases = {r['phase']: int(r['elapsed_ns']) for r in rows if r['event'] == 'export-phase' and r['job'] == job}
    resumes = {r['phase']: int(r['elapsed_ns']) for r in rows if r['event'] == 'export-resume' and r['job'] == job}
    timings.append({'job': job, 'format': captures[job]['format'], 'capture_ms': int(captures[job]['capture_ns']) / 1e6, 'prepare_ms': (phases['Prepared'] - resumes['Captured']) / 1e6, 'encode_ms': (phases['Encoded'] - resumes['Prepared']) / 1e6, 'host_save_ms': int(result['save_ns']) / 1e6, 'completed_at_commit': result['tick']})
print(json.dumps({'status': 'PASS', 'scope': 'finite native FIX-14; not sustained PERF-03/05', 'commits': len(commits), 'metrics': m, 'timings': timings}, indent=2))
