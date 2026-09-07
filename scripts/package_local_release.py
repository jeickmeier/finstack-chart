#!/usr/bin/env python3
"""Create a deterministic, unpublished workspace source candidate from a committed tree."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path
import subprocess
import tarfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('output', type=Path)
    parser.add_argument('--ref', default='HEAD', help='Existing committed revision; excludes uncommitted changes')
    args = parser.parse_args()
    commit = subprocess.check_output(['git', 'rev-parse', '--verify', args.ref + '^{commit}'], cwd=ROOT, text=True).strip()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    name = f'finstack-chart-0.1.0-local-{commit[:12]}'
    archive = output / (name + '.tar.gz')
    if archive.exists():
        raise SystemExit(f'Refusing to overwrite {archive}')
    payload = subprocess.check_output(['git', 'archive', '--format=tar', '--prefix=' + name + '/', commit], cwd=ROOT)
    with tarfile.open(fileobj=io.BytesIO(payload)) as source:
        members = source.getmembers()
        paths = [m.name.removeprefix(name + '/') for m in members if m.isfile()]
        assert {'Cargo.lock', 'Cargo.toml', 'mise.toml', 'README.md', 'scripts/check_repository.py'} <= set(paths)
        assert all(not p.startswith(('target/', 'artifacts/', '.git/')) for p in paths)
        files = {m.name.removeprefix(name + '/'): hashlib.sha256(source.extractfile(m).read()).hexdigest() for m in members if m.isfile()}
    archive.write_bytes(gzip.compress(payload, compresslevel=6, mtime=0))
    # Verify compressed payload before claiming an artifact.
    assert gzip.decompress(archive.read_bytes()) == payload
    report = {'kind': 'unpublished local workspace source candidate', 'production_certification': False, 'version': '0.1.0', 'commit': commit, 'archive': archive.name, 'sha256': hashlib.sha256(archive.read_bytes()).hexdigest(), 'bytes': archive.stat().st_size, 'file_count': len(files), 'files': files, 'reproduce': ['mise trust', 'mise install', 'mise run check', 'mise run test', 'mise run bindings-proof artifacts/release-proof'], 'exclusions': ['uncommitted work', 'build outputs', 'toolchains and dependency caches'], 'publication': 'Disabled; owner license and registry metadata unresolved; see ADR-010'}
    (output / (name + '.json')).write_text(json.dumps(report, indent=2, sort_keys=True) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k != 'files'}, indent=2))


if __name__ == '__main__':
    main()
