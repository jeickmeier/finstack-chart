"""Check actual acknowledged native frames and a retained mid-transition export."""
import json
import math
import sys
from pathlib import Path

root = Path(sys.argv[1])
events = [json.loads(line) for line in (root / "native-trace.jsonl").read_text().splitlines()]
painted = [event for event in events if event["event"] == "painted"]
assert painted and all(event["error"] is None for event in painted)
assert {event["phase"] for event in painted} >= {0, 1, 2, 3}


def close(actual, expected):
    assert math.isclose(actual, expected, rel_tol=0, abs_tol=1e-8), (actual, expected)


counts = {}
for event in painted:
    phase = event["phase"]
    frame = event["guides"][0]["frame"]
    ticks = frame["ticks"]
    assert len({tick["identity"] for tick in ticks}) == len(ticks)
    assert all(0 <= tick["opacity"] <= 1 for tick in ticks)
    counts[phase] = counts.get(phase, 0) + 1
    if phase == 1:
        assert [t["identity"] for t in ticks] == [0, 1, 2, 3]
        assert [t["label"] for t in ticks] == ["ZERO", "half", "ONE", "TWO"]
        fraction = (850 - ticks[3]["position"]) / 400
        close(ticks[1]["position"], 250 - 100 * fraction)
        close(ticks[2]["position"], 450 - 200 * fraction)
        close(ticks[1]["opacity"], 1 - (1 - 1e-6) * fraction)
        close(ticks[3]["opacity"], 1e-6 + (1 - 1e-6) * fraction)
    elif phase == 2 and len(ticks) == 5:
        assert [t["identity"] for t in ticks] == [0, 1, 2, 3, 4]
        assert [t["label"] for t in ticks] == ["zero again", "half", "ONE", "two again", "four"]
        fraction = (850 - ticks[4]["position"]) / 400
        close(ticks[4]["opacity"], 1e-6 + (1 - 1e-6) * fraction)
        close(ticks[2]["opacity"], 1 - (1 - 1e-6) * fraction)
    elif phase == 3:
        assert [t["identity"] for t in ticks] == [0, 5, 3]
        assert [t["position"] for t in ticks] == [50, 250, 450]
        assert all(t["opacity"] == 1 for t in ticks)
        assert frame["next_identity"] == 6
assert counts[1] > 5 and counts[2] > 5
captures = [event for event in events if event["event"] == "capture-acquired"]
assert len(captures) == 1
captured = json.loads((root / "native-captured.presentation.json").read_text())
assert captured == captures[0]["guides"]
assert any(0 < t["opacity"] < 1 for t in captured[0]["frame"]["ticks"])
end = events[-1]
assert end["event"] == "PASS" and end["disposed"] and end["layout_attempts"] == 4
assert end["elapsed"] > captures[0]["elapsed"] + 5
result = {"status": "PASS", "painted_samples_by_phase": counts,
          "retained_capture_exact": True, "layout_attempts": end["layout_attempts"],
          "paint_submissions": end["paints"], "performance_certified": False}
(root / "native-validation.json").write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps(result))
