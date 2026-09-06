#!/usr/bin/env python3
"""WP-03 independent artifact checks; Python stdlib plus Poppler command-line tools.

This checks stored publication artifacts, not native visuals or full FIX conformance.
"""

import base64
import hashlib
import re
import struct
import subprocess
import sys
import unicodedata
import xml.etree.ElementTree as ET
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SVG = "{http://www.w3.org/2000/svg}"
FONT_HASHES = {
    "2ec33f84606cbaa0a1a944488e14f97faf2f6a25ecdd8354f5358f06da13c7d9",
    "5f9173ce3d05fadef74c7eed06570d54e4f75bd0cd9860726fb2987a7f848292",
}


def command(*args):
    return subprocess.check_output(args, text=True)


def check(output):
    for mode in ("text", "outline"):
        svg_path = output / f"capability-{mode}.svg"
        source = svg_path.read_text()
        root = ET.fromstring(source)
        assert (root.get("width"), root.get("height")) == ("180mm", "120mm")
        # Two curves, three cap/join probes and one thin rule are authored paths.
        assert len(list(root.iter(SVG + "path"))) >= 6
        assert len(list(root.iter(SVG + "linearGradient"))) == 1
        assert len(list(root.iter(SVG + "clipPath"))) == 1
        assert not list(root.iter(SVG + "image"))
        text_count = len(list(root.iter(SVG + "text")))
        assert (text_count > 0) if mode == "text" else (text_count == 0)
        if mode == "text":
            embedded = re.findall(r"data:font/ttf;base64,([A-Za-z0-9+/=]+)", source)
            assert {hashlib.sha256(base64.b64decode(font)).hexdigest() for font in embedded} == FONT_HASHES
        pdf = str(output / f"capability-{mode}.pdf")
        info = command("pdfinfo", pdf)
        page_size = re.search(r"Page size:\s+([\d.]+) x ([\d.]+) pts", info)
        assert page_size, info
        width, height = map(float, page_size.groups())
        # Independent physical expectations; Poppler prints only 3 decimal places.
        assert abs(width - 510.2362204724) < 0.002
        assert abs(height - 340.1574803150) < 0.002
        assert re.search(r"Pages:\s+1\b", info)
        images = command("pdfimages", "-list", pdf).strip().splitlines()[2:]
        assert images == [], images
        fonts = command("pdffonts", pdf).strip().splitlines()[2:]
        extracted = command("pdftotext", "-layout", pdf, "-")
        if mode == "text":
            assert len(fonts) == 2, fonts
            assert all(re.search(r"yes\s+yes\s+yes", row) for row in fonts), fonts
            assert any("NotoSans-Regular" in row for row in fonts)
            assert any("FiraMono-Medium" in row for row in fonts)
            normalized = unicodedata.normalize("NFC", extracted)
            for expected in ("Native / publication capability proof", "café", "Ω", "12,345.67", "Rotated vector text"):
                assert expected in normalized, expected
        else:
            assert fonts == []
            assert extracted.strip() == ""
        print(f"PASS {mode}: SVG dimensions/vector structure; PDF 180x120 mm, no images; font/text policy")
    for dpi, expected_size, expected_ppm in ((300, (2126, 1417), 11811), (600, (4252, 2835), 23622)):
        data = (output / f"capability-{dpi}.png").read_bytes()
        assert data[:8] == b"\x89PNG\r\n\x1a\n"
        chunks = {}
        offset = 8
        while offset < len(data):
            length = struct.unpack_from(">I", data, offset)[0]
            kind = data[offset + 4:offset + 8]
            chunks[kind] = data[offset + 8:offset + 8 + length]
            offset += length + 12
        assert struct.unpack_from(">II", chunks[b"IHDR"]) == expected_size
        assert struct.unpack(">IIB", chunks[b"pHYs"]) == (expected_ppm, expected_ppm, 1)
        print(f"PASS PNG {dpi}: {expected_size[0]}x{expected_size[1]}, physical density {expected_ppm} pixels/metre")
    print("Structural checks passed. Human visual/native/runtime review remains separate.")


if __name__ == "__main__":
    check(Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "docs/evidence/wp-03")
