#!/usr/bin/env python3
"""Retain R graphics-engine geometry from its uncompressed PDF-device commands.

The PDF device rounds coordinates to 0.01 points. Endpoint reconstruction from
rounded rectangle origin and extent permits 0.011 points per coordinate.
No symbol geometry is implemented by this oracle extractor.
"""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "fixtures/parity/ggplot2/aesthetics.json"


def primitives(path):
    streams = re.findall(rb"(?<!end)stream\r?\n(.*?)endstream", path.read_bytes(), re.S)
    stream = next(s for s in streams if re.search(rb"\b(?:m|re)\b", s))
    result, stack, points, curves, rectangle = [], [], [], [], None
    closed = False
    for token in stream.decode("ascii").split():
        try:
            stack.append(float(token))
            continue
        except ValueError:
            pass
        if token == "m":
            assert len(stack) == 2
            points = [stack[:]]
        elif token == "l":
            assert len(stack) == 2
            points.append(stack[:])
        elif token == "c":
            assert len(stack) == 6
            curves.append(stack[:])
        elif token == "re":
            assert len(stack) == 4
            x, y, w, h = stack
            rectangle = [[x, y], [x+w, y], [x+w, y+h], [x, y+h]]
        elif token == "h":
            closed = True
        elif token in ("S", "f", "B"):
            record = {"stroke": token in ("S", "B"), "fill": token in ("f", "B")}
            if curves:
                assert len(curves) == 4 and len(points) == 1 and rectangle is None
                anchors = points + [c[-2:] for c in curves]
                x0, x1 = min(p[0] for p in anchors), max(p[0] for p in anchors)
                y0, y1 = min(p[1] for p in anchors), max(p[1] for p in anchors)
                assert abs(x1-x0-(y1-y0)) < 0.011
                record.update(kind="circle", center=[(x0+x1)/2-72,72-(y0+y1)/2], radius=(x1-x0)/2)
            else:
                if rectangle is not None:
                    points, closed = rectangle, True
                assert points
                record.update(kind="polygon" if closed else "line", points=[[x-72,72-y] for x,y in points])
            result.append(record)
            points, curves, rectangle, closed = [], [], None, False
        elif token == "cm":
            raise AssertionError("Unexpected PDF transform in the point oracle")
        stack.clear()
    assert result
    return result


def main():
    document = json.loads(FIXTURE.read_text())
    for symbol in document["symbols"]:
        path = ROOT / symbol.pop("pdf")
        symbol["primitives"] = primitives(path)
    document["symbol_coordinate_tolerance_points"] = 0.011
    FIXTURE.write_text(json.dumps(document, indent=2, ensure_ascii=False) + "\n")
    files = ["tools/reference/r/aesthetics.R", "tools/reference/r/aesthetic_records.py", "tools/reference/r/run.py",
             "tools/reference/r/renv.lock", "fixtures/parity/ggplot2/aesthetics.json", "fixtures/parity/ggplot2/sources.json"]
    manifest = {"owner":"GG-03","reference":"ggplot2 4.0.3 / R 4.6.1 r90187", "cases":len(document["cases"]),
                "symbols":len(document["symbols"]), "symbol_coordinate_tolerance_points":0.011,
                "files":{name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in files}}
    (ROOT/"fixtures/parity/ggplot2/aesthetics-manifest.json").write_text(json.dumps(manifest,indent=2)+"\n")
    print(f"PASS {manifest['cases']} aesthetic builds and {manifest['symbols']} captured R point geometries")


if __name__ == "__main__":
    main()
