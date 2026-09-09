"""Retain checksummed source provenance for every package in the exact R lock.

Explicit network setup; never called by cargo tests or by ordinary fixture comparison.
"""
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import hashlib
import json
import re
import tarfile
import urllib.error
import urllib.request

ROOT = Path(__file__).resolve().parents[3]
lock_path = ROOT / "tools/reference/r/renv.lock"
lock = json.loads(lock_path.read_text())
downloads = ROOT / "target/reference-r/sources"
downloads.mkdir(parents=True, exist_ok=True)
notices = ROOT / "fixtures/parity/ggplot2/licenses"
notices.mkdir(parents=True, exist_ok=True)
def fetch(item):
    name, package = item
    version = package['Version']
    assert re.fullmatch(r'[A-Za-z0-9.]+', name) and re.fullmatch(r'[A-Za-z0-9.+-]+', version)
    filename = f'{name}_{version}.tar.gz'
    archive = downloads / filename
    urls = [f'https://cran.r-project.org/src/contrib/{filename}',
            f'https://cran.r-project.org/src/contrib/Archive/{name}/{filename}']
    source = urls[0]
    if not archive.exists():
        for source in urls:
            try:
                with urllib.request.urlopen(source, timeout=120) as response:
                    data = response.read()
                archive.write_bytes(data)
                break
            except urllib.error.HTTPError as error:
                if error.code != 404: raise
        else: raise RuntimeError(f'No source for {name} {version}')
        (archive.with_suffix('.url')).write_text(source)
    else:
        source = archive.with_suffix('.url').read_text()
    with tarfile.open(archive) as tar:
        for member in tar.getmembers():
            parts = Path(member.name).parts
            if member.isfile() and len(parts)==2 and (parts[-1].upper().startswith(('LICENSE','COPYING'))):
                (notices / f'{name}-{parts[-1]}').write_bytes(tar.extractfile(member).read())
    return {'name':name,'version':version,'source':source,
            'sha256':hashlib.sha256(archive.read_bytes()).hexdigest(),
            'license':package.get('License'), 'archive':str(archive.relative_to(ROOT))}
with ThreadPoolExecutor(max_workers=6) as pool:
    rows=list(pool.map(fetch, sorted(lock['Packages'].items())))
manifest={'schema_version':1,'r_version':lock['R']['Version'],
          'lock_sha256':hashlib.sha256(lock_path.read_bytes()).hexdigest(),'packages':rows}
(ROOT/'fixtures/parity/ggplot2/sources.json').write_text(json.dumps(manifest,indent=2)+'\n')
print(f'PASS source archive identities and licenses: {len(rows)} R packages.')
