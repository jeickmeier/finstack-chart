#!/usr/bin/env python3
"""Build/run local macOS benchmark processes with actual Metal presentation callbacks.

A graphical session is required. Each short benchmark uses a temporary always-visible
window. The default stream duration is 60 seconds; 1800 must be explicitly requested.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[2]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--stream-seconds', type=int, default=60)
    parser.add_argument('--modes', nargs='+', default=['lines', 'million', 'dashboard-raw', 'dashboard-dense', 'stream'], choices=['lines', 'million', 'dashboard-raw', 'dashboard-dense', 'stream'])
    args = parser.parse_args()
    assert platform.system() == 'Darwin' and args.stream_seconds > 0
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    for mode in args.modes:
        assert not (output / mode).exists(), f'Refusing to replace existing evidence: {mode}'
    subprocess.run(['mise', 'exec', '--', 'cargo', 'build', '-p', 'chart-gallery', '--example', 'finite_benchmark', '--example', 'sustained_benchmark', '--features', 'performance', '--release', '--locked'], cwd=ROOT, check=True)
    library = output / 'present-probe.dylib'
    subprocess.run(['xcrun', 'clang', '-O2', '-fobjc-arc', '-fblocks', '-dynamiclib', str(ROOT / 'scripts/performance/macos_present_probe.m'), '-framework', 'Foundation', '-framework', 'Metal', '-framework', 'QuartzCore', '-o', str(library)], check=True)
    for mode in args.modes:
        out = output / mode
        out.mkdir()
        executable = ROOT / 'target/release/examples' / ('sustained_benchmark' if mode == 'stream' else 'finite_benchmark')
        (out / 'build.json').write_text(json.dumps({'executable': executable.name, 'sha256': hashlib.sha256(executable.read_bytes()).hexdigest(), 'probe_sha256': hashlib.sha256(library.read_bytes()).hexdigest(), 'window': 'temporary PopUp to avoid occlusion', 'platform': platform.platform()}, indent=2) + '\n')
        env = dict(os.environ, DYLD_INSERT_LIBRARIES=str(library), FINSTACK_METAL_LOG=str(out / 'metal.jsonl'))
        command = [str(executable), str(args.stream_seconds), str(out)] if mode == 'stream' else [str(executable), mode]
        with (out / 'events.jsonl').open('w') as log, (out / 'stderr.txt').open('w') as error:
            subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=error, check=True)
        command = ['python3', str(ROOT / 'scripts/performance/check_native_performance.py'), str(out / 'events.jsonl'), str(out / 'metal.jsonl'), '--frames', str(out / 'frames.jsonl')]
        if mode == 'stream':
            command += ['--seconds', str(args.stream_seconds)]
        with (out / 'report.json').open('w') as report:
            subprocess.run(command, stdout=report, check=True)
        print(f'{mode}: evidence complete', flush=True)


if __name__ == '__main__':
    main()
