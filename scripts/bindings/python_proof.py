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
for case in json.loads((ROOT / "fixtures/statistics/portable-cases.json").read_text()) + json.loads((ROOT / "fixtures/families/portable-cases.json").read_text()):
    proof = Chart(json.dumps(case["chart"]), json.dumps(case["data"]), read("profile"), (ROOT / "fixtures/capability/fonts/NotoSans-Regular.ttf").read_bytes())
    statistics[case["name"]] = json.loads(proof.semantics())
    scenes[case["name"]] = json.loads(proof.scene())
    (output / f"statistics-{case['name']}.svg").write_bytes(proof.export("svg"))
    proof.dispose()
save("statistics", json.dumps(statistics)); save("statistics-scenes", json.dumps(scenes))
print(f"PASS {len(statistics)} statistic/position/family cases in the actual Python extension")
