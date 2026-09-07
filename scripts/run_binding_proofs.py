#!/usr/bin/env python3
"""Build and execute the actual WP-09/10/11/12/13/14/15 native/PyO3/wasm-bindgen proof, using pinned crates.

Run under mise; WASM_BINDGEN may name a task-local matching CLI. No downloads occur here.
"""
import os
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
output = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else ROOT / "artifacts/bindings"
cli = os.environ.get("WASM_BINDGEN", "wasm-bindgen")
node = os.environ.get("NODE", "node")
version = subprocess.check_output([cli, "--version"], text=True).strip()
if version != "wasm-bindgen 0.2.128":
    raise SystemExit("Require wasm-bindgen-cli 0.2.128 to match Cargo.lock; install it or set WASM_BINDGEN to its executable.")
for command in (node, "pdfinfo", "pdfimages", "pdffonts"):
    if shutil.which(command) is None: raise SystemExit(f"Required proof tool unavailable: {command}")
output.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, PYO3_PYTHON=sys.executable)

def run(name, command):
    print("RUN", " ".join(map(str, command)), flush=True)
    with (output / f"{name}.log").open("w") as log:
        subprocess.run(list(map(str, command)), cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
    print((output / f"{name}.log").read_text()[-1600:], end="", flush=True)

(output / "environment.txt").write_text(
    f"cwd={ROOT}\npython={sys.version}\nexecutable={sys.executable}\nwasm-cli={version}\n"
    + subprocess.check_output([node,"--version"],text=True)
    + subprocess.check_output(["rustc","-Vv"],text=True)
)
run("native", ["cargo","run","-p","chart-export","--example","binding_proof","--locked","--",output / "native"])
run("python-build", ["cargo","build","-p","chart-python","--features","extension-module,extension-proof","--locked"])
module = output / "python-module"; module.mkdir(exist_ok=True)
library = ROOT / "target/debug" / ("libchart_python.dylib" if sys.platform == "darwin" else "libchart_python.so")
# Keep an owned copy so later default-feature workspace builds cannot replace the loaded extension.
destination = module / "chart_python.so"
if destination.is_symlink(): destination.unlink()
shutil.copy2(library, destination)
run("python", [sys.executable,ROOT / "scripts/bindings/python_proof.py",module,output / "python"])
run("wasm-build", ["cargo","build","-p","chart-wasm","--features","extension-proof","--target","wasm32-unknown-unknown","--locked"])
run("wasm-generate", [cli,ROOT / "target/wasm32-unknown-unknown/debug/chart_wasm.wasm","--target","nodejs","--out-dir",output / "wasm-module"])
run("wasm", [node,ROOT / "scripts/bindings/wasm_proof.cjs",output / "wasm-module",output / "wasm"])
run("compare", [sys.executable,ROOT / "scripts/bindings/compare.py",output])
print(f"PASS WP-09/10/11/12/13/14/15/16/17/18/19/20 runtime proof. Results: {output}")
