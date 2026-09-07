#!/usr/bin/env python3
"""WP-08 stored-artifact checks, independent of the Rust implementation.

Uses Python stdlib and Poppler. Visual inspection is recorded separately.
"""
import base64
import hashlib
import re
import struct
import subprocess
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

SVG = "{http://www.w3.org/2000/svg}"
FONT_HASH = "2ec33f84606cbaa0a1a944488e14f97faf2f6a25ecdd8354f5358f06da13c7d9"


def command(*args):
    return subprocess.check_output(args, text=True)


def check(output):
    for mode in ("text", "outline"):
        source = (output / f"publication-{mode}.svg").read_text()
        root = ET.fromstring(source)
        for key, expected in (("width", 180 * 72 / 25.4), ("height", 120 * 72 / 25.4)):
            assert root.get(key).endswith("pt")
            assert abs(float(root.get(key)[:-2]) - expected) < 0.00001
        assert len(list(root.iter(SVG + "path"))) >= 2
        assert list(root.iter(SVG + "clipPath"))
        assert not list(root.iter(SVG + "image"))
        assert bool(list(root.iter(SVG + "text"))) == (mode == "text")
        if mode == "text":
            embedded = re.findall(r"data:font/ttf;base64,([A-Za-z0-9+/=]+)", source)
            assert [hashlib.sha256(base64.b64decode(f)).hexdigest() for f in embedded] == [FONT_HASH]
            # Independent projection of two line segments around a missing y value.
            paths = [p for p in root.iter(SVG + "path") if p.get("stroke") == "#235a96"]
            assert len(paths) == 2
            expected = [(43.72, 288.5374814, 151.0840551, 120.7191601),
                        (365.8121657, 204.6283207, 473.1762210, 36.81)]
            for path, coordinates in zip(paths, expected):
                actual = [float(v) for v in re.findall(r"[-+]?\d*\.?\d+(?:e[-+]?\d+)?", path.get("d"))]
                assert len(actual) == 4
                assert max(abs(a - b) for a, b in zip(actual, coordinates)) < 0.0001
            assert len(list(root.iter(SVG + "circle"))) == 4
        pdf = str(output / f"publication-{mode}.pdf")
        info = command("pdfinfo", pdf)
        match = re.search(r"Page size:\s+([\d.]+) x ([\d.]+) pts", info)
        assert match, info
        assert abs(float(match[1]) - 180 * 72 / 25.4) < 0.002
        assert abs(float(match[2]) - 120 * 72 / 25.4) < 0.002
        assert re.search(r"Pages:\s+1\b", info)
        assert not command("pdfimages", "-list", pdf).strip().splitlines()[2:]
        fonts = command("pdffonts", pdf).strip().splitlines()[2:]
        text = command("pdftotext", "-layout", pdf, "-")
        if mode == "text":
            assert len(fonts) == 1 and "NotoSans-Regular" in fonts[0]
            assert re.search(r"yes\s+yes\s+yes", fonts[0]), fonts
            assert "Captured publication - café Ω" in text
        else:
            assert not fonts and not text.strip()
        print(f"PASS {mode}: SVG/PDF physical dimensions, vector marks, clips, explicit font/text policy; no images")
    assert (output / "publication-preview.svg").read_bytes() == (output / "publication-outline.svg").read_bytes()
    print("PASS preview is the exact outlined publication SVG")
    for dpi, size, ppm in ((300, (2126, 1417), 11811), (600, (4252, 2835), 23622)):
        data = (output / f"publication-{dpi}.png").read_bytes()
        assert data[:8] == b"\x89PNG\r\n\x1a\n"
        chunks, offset = {}, 8
        while offset < len(data):
            length = struct.unpack_from(">I", data, offset)[0]
            chunks[data[offset + 4:offset + 8]] = data[offset + 8:offset + 8 + length]
            offset += length + 12
        assert struct.unpack_from(">II", chunks[b"IHDR"]) == size
        assert struct.unpack(">IIB", chunks[b"pHYs"]) == (ppm, ppm, 1)
        print(f"PASS PNG {dpi}: {size}, {ppm} pixels/metre")


if __name__ == "__main__":
    check(Path(sys.argv[1]) if len(sys.argv) > 1 else Path("docs/evidence/wp-08"))
