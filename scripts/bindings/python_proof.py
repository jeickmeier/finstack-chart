"""Execute the real PyO3 extension; copies, state/results, errors and headless exports."""
import gc
import json
import sys
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(Path(sys.argv[1]).resolve()))
from chart_python import Chart, ChartError

output = Path(sys.argv[2]); output.mkdir(parents=True, exist_ok=True)
fixture = ROOT / "fixtures/bindings"
read = lambda name: (fixture / f"{name}.json").read_text()
font = bytearray((ROOT / "fixtures/capability/fonts/NotoSans-Regular.ttf").read_bytes())
chart = Chart(read("chart"), read("data"), read("profile"), font)
# Mutating the source cannot affect Rust's owned font snapshot.
font[:] = b"\x00" * len(font)
save = lambda name, value: (output / f"{name}.json").write_text(value)
save("initial", chart.semantics()); save("definition", chart.definition())
save("transaction", chart.transaction(read("correction")))
save("replay", chart.transaction(read("correction")))
save("action", chart.action(read("action")))
save("final", chart.semantics()); save("scene", chart.scene()); save("state", chart.state())
state_before = chart.state()
chart.restore_state(state_before, "1")
try:
    chart.action(read("action"))
    raise AssertionError("stale action succeeded")
except ChartError as error:
    assert json.loads(error.args[0])["code"] == "CHART_REVISION_CONFLICT"
assert chart.state() == state_before
stale = json.loads(read("correction")); stale["id"] = "stale-correction"
assert "Conflict" in json.loads(chart.transaction(json.dumps(stale)))
assert chart.state() == state_before
for fmt in ("svg", "pdf", "png"):
    data = chart.export(fmt)
    assert isinstance(data, bytes)
    (output / f"chart.{fmt}").write_bytes(data)
# A second Python thread must run during long Rust-only work, not just before/after it.
# The source contract also uses Python::detach for construction, prepare, updates and exports.
progress = 0
gate = threading.Event()
stop = threading.Event()
def worker():
    global progress
    gate.wait()
    while not stop.is_set():
        progress += 1
        time.sleep(0.0001)
thread = threading.Thread(target=worker); thread.start()
interval = sys.getswitchinterval()
# Prevent ordinary Python bytecode timeslicing around the call from satisfying the test.
# The waiting worker can acquire the GIL only when the Rust entry point detaches it.
sys.setswitchinterval(10.0)
try:
    gate.set()
    chart.export("png")
    advanced = progress
    stop.set()
finally:
    sys.setswitchinterval(interval)
thread.join()
assert advanced > 0, "Python thread did not progress during detached Rust work"
save("ownership", json.dumps({"owned_font_mutation": True, "owned_output_bytes": True, "python_thread_progress": advanced}))
retained = chart.export("svg")
chart.dispose(); chart.dispose()
try:
    chart.scene()
    raise AssertionError("disposed chart succeeded")
except ChartError as error:
    assert json.loads(error.args[0])["code"] == "CHART_DISPOSED_HANDLE"
assert retained.startswith(b"<svg") or b"<svg" in retained[:200]
del chart; gc.collect()
for case in json.loads((fixture / "negative.json").read_text()):
    values = {name: json.loads(read(name)) for name in ("chart", "data", "profile")}
    node = values[case["target"]]
    parts = case["path"].split("/")[1:]
    for key in parts[:-1]: node = node[int(key)] if isinstance(node, list) else node[key]
    key = int(parts[-1]) if isinstance(node, list) else parts[-1]
    if case.get("remove"): del node[key]
    else: node[key] = case["value"]
    try:
        Chart(*(json.dumps(values[name]) for name in ("chart", "data", "profile")), (ROOT / "fixtures/capability/fonts/NotoSans-Regular.ttf").read_bytes())
        raise AssertionError(case["name"])
    except ChartError as error:
        actual = json.loads(error.args[0])["code"]
        assert actual == case["code"], (case["name"], actual)
print(f"PASS Python: actual extension, portable fixture, copies, detached thread progress, disposal and {len(json.loads((fixture / 'negative.json').read_text()))} malformed input cases")

# WP-10: every new builtin executes through the actual extension and existing export engine.
statistics = {}; scenes = {}
for case in json.loads((ROOT / "fixtures/statistics/portable-cases.json").read_text()) + json.loads((ROOT / "fixtures/families/portable-cases.json").read_text()) + json.loads((ROOT / "fixtures/facets/portable-cases.json").read_text()) + json.loads((ROOT / "fixtures/composition/portable-cases.json").read_text()) + json.loads((ROOT / "fixtures/extensions/portable-cases.json").read_text()):
    constructor = Chart.with_example_extensions if case.get("registered") else Chart
    proof = constructor(json.dumps(case["chart"]), json.dumps(case["data"]), json.dumps(case["profile"]) if "profile" in case else (ROOT / case["profile_file"]).read_text() if "profile_file" in case else read("profile"), (ROOT / "fixtures/capability/fonts/NotoSans-Regular.ttf").read_bytes())
    statistics[case["name"]] = json.loads(proof.semantics())
    scenes[case["name"]] = json.loads(proof.scene())
    (output / f"statistics-{case['name']}.svg").write_bytes(proof.export("svg"))
    proof.dispose()
    if case.get("registered"):
        for mode in ("unregistered", "unknown-version", "native-only", "wrong-schema"):
            bad = json.loads(json.dumps(case["chart"]))
            construct = Chart.with_example_extensions
            if mode == "unregistered": construct = Chart
            elif mode == "unknown-version": bad["definition"]["transforms"][0]["statistic"]["operation"]["version"] = "99"
            elif mode == "native-only": bad["definition"]["layers"][0]["geometry_extension"]["operation"]["id"] = "example.native_bars"
            else: bad["definition"]["layers"][0]["mappings"]["Statistical"]["x"] = {"Field":{"Custom":"absent"}}
            try:
                construct(json.dumps(bad), json.dumps(case["data"]), (ROOT / case["profile_file"]).read_text(), (ROOT / "fixtures/capability/fonts/NotoSans-Regular.ttf").read_bytes())
                raise AssertionError("invalid extension accepted: " + mode)
            except ChartError as error:
                assert json.loads(error.args[0])["code"] == ("CHART_SCHEMA_CONFLICT" if mode == "wrong-schema" else "CHART_UNSUPPORTED_CAPABILITY")
        print("PASS Python FIX-17 unregistered/unknown-version/native-only/stage-schema errors")
    if case["name"] == "facet-shared-broadcast":
        for target, expected in [(None, "CHART_SCHEMA_CONFLICT"), ({"Panels":[{"values":[{"Text":"absent"}]}]}, "CHART_VALIDATION")]:
            bad = json.loads(json.dumps(case["chart"]))
            if target is None: bad["definition"]["layers"][1].pop("facet")
            else: bad["definition"]["layers"][1]["facet"] = target
            try:
                Chart(json.dumps(bad), json.dumps(case["data"]), json.dumps(case["profile"]), (ROOT / "fixtures/capability/fonts/NotoSans-Regular.ttf").read_bytes())
                raise AssertionError("invalid facet target accepted")
            except ChartError as error:
                assert json.loads(error.args[0])["code"] == expected
        print("PASS Python FIX-06: missing facet policy and unknown target panel rejected")
save("statistics", json.dumps(statistics)); save("statistics-scenes", json.dumps(scenes))
print(f"PASS {len(statistics)} statistic/position/family cases in the actual Python extension")

# WP-15: one portable action trace shared with Rust/WASM; expected values are authored.
def subset(actual, expected):
    if isinstance(expected, dict):
        for key, value in expected.items(): subset(actual[key], value)
    elif isinstance(expected, list):
        assert len(actual) == len(expected)
        for a, b in zip(actual, expected): subset(a, b)
    else: assert actual == expected, (actual, expected)
proof = Chart((ROOT / 'fixtures/actions/chart.json').read_text(),read('data'),read('profile'),(ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
stamp = json.loads(proof.present())['stamp']; trace = []
for step in json.loads((ROOT / 'fixtures/actions/trace.json').read_text()):
    before = json.loads(proof.state())
    request = dict(definition_revision=before['definition_revision'],expected_state=step.get('expected_state',before['state_revision']),origin=step.get('origin','Control'),scene=stamp,action=step['action'])
    try:
        result = json.loads(proof.dispatch(json.dumps(request)))
        assert 'error' not in step, step['name']
    except ChartError as error:
        code = json.loads(error.args[0])['code']; assert code == step.get('error'), (step['name'], code)
        result = {'error':code}
    after = json.loads(proof.state())
    if 'expect' in step: subset(result,step['expect'])
    if 'state' in step: subset(after,step['state'])
    if 'error' in step: assert after == before
    if 'export' in step: (output / f"actions-{step['export']}.svg").write_bytes(proof.export('svg'))
    if step.get('present'): stamp = json.loads(proof.present())['stamp']
    trace.append(dict(name=step['name'],result=result,state=after))
save('actions-state-trace',json.dumps(trace)); proof.dispose()
print(f'PASS WP-15 Python shared action trace: {len(trace)} transitions')

# WP-16 consumes exactly the shared independent query/action cases used by Rust and WASM.
input_trace = []
for case in sum((json.loads((ROOT / p).read_text()) for p in ['fixtures/interaction/cases.json','fixtures/host-tools/cases.json']), []):
    chart = Chart(json.dumps(case['chart']),json.dumps(case['data']),(ROOT / 'fixtures/interaction/profile.json').read_text(),(ROOT / 'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
    stamp = json.loads(chart.present())['stamp']; basis = None
    initial = json.loads(chart.semantics())
    for step in case['queries']:
        before = json.loads(chart.state()); result = None
        if 'query' in step:
            query_stamp = dict(basis if step.get('gesture') else stamp)
            if step.get('stale'): query_stamp['layout'] = '999999'
            try:
                result = json.loads(chart.query(json.dumps(dict(scene=query_stamp,gesture=step.get('gesture',False),query=step['query']))))
                assert 'error' not in step
            except ChartError as error:
                code = json.loads(error.args[0])['code']; assert code == step.get('error'), (case['name'],step['name'],code)
                result = {'error':code}
            assert json.loads(chart.state()) == before
            if 'expect' in step: subset(result,step['expect'])
        action = step.get('action')
        if step.get('apply') == 'annotation_preview': action = {'PreviewGesture':{'id':step['id'],'preview':{'Annotation':result['annotation']}}}
        if step.get('apply') == 'linked': action = result['action']
        if step.get('apply') == 'windows': action = {'SetAxisWindows':result['windows']}
        if step.get('apply') == 'preview': action = {'PreviewGesture':{'id':step['id'],'preview':{'AxisWindows':result['windows']}}}
        if step.get('apply') == 'targets': action = {'Select':{'change':step.get('change','Replace'),'targets':result['targets']}}
        if action is not None:
            if 'BeginGesture' in action: basis = dict(stamp)
            current = json.loads(chart.state())
            chart.dispatch(json.dumps(dict(definition_revision=current['definition_revision'],expected_state=current['state_revision'],scene=basis or stamp,origin=result['origin'] if step.get('apply') == 'linked' else 'Pointer',action=action)))
            if 'CancelGesture' in action or 'CommitGesture' in action: basis = None
        after = json.loads(chart.state())
        if 'state' in step: subset(after,step['state'])
        if step.get('present'):
            stamp = json.loads(chart.present())['stamp']
            (output / f"{case['name']}-{step['name']}.svg").write_bytes(chart.export('svg'))
        semantic = json.loads(chart.semantics())
        assert semantic['datasets'] == initial['datasets']
        for a,b in zip(semantic['layers'],initial['layers']):
            assert a['rows'] == b['rows']; assert a['domains'] == b['domains']
        input_trace.append(dict(case=case['name'],name=step['name'],result=result,state=after))
    chart.dispose()
save('input-trace',json.dumps(input_trace))
print(f'PASS WP-16/17 Python shared input queries/actions: {len(input_trace)} steps')

stream_case=json.loads((ROOT/'fixtures/streaming/replay.json').read_text())
chart=Chart(json.dumps(stream_case['chart']),json.dumps(stream_case['data']),(ROOT/'fixtures/interaction/profile.json').read_text(),(ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
stamp=json.loads(chart.present())['stamp'];stream_trace=[]
for step in stream_case['steps']:
    try:
        if 'transaction' in step: result=json.loads(chart.transaction(json.dumps(step['transaction'])))
        elif 'stream' in step: result=json.loads(chart.stream(json.dumps(dict(version=1,operation=step['stream']))))
        else:
            state=json.loads(chart.state())
            result=json.loads(chart.dispatch(json.dumps(dict(definition_revision=state['definition_revision'],expected_state=state['state_revision'],scene=stamp,origin='Control',action=step['action']))))
        assert 'error' not in step,step['name']
    except ChartError as error:
        code=json.loads(error.args[0])['code'];assert code==step['error'];result={'error':code}
    semantic=json.loads(chart.semantics());state=json.loads(chart.state())
    if step.get('present'):
        stamp=json.loads(chart.present())['stamp'];(output/f"stream-{step['name']}.svg").write_bytes(chart.export('svg'))
    stream_trace.append(dict(name=step['name'],result=result,semantics=semantic,state=state))
save('stream-trace',json.dumps(stream_trace));chart.dispose()
print(f'PASS WP-18 Python streaming replay: {len(stream_trace)} steps')
