"""Retain the independent GG-02 stage oracle without rewriting the historical GG-00 snapshot."""
from pathlib import Path
import hashlib
import json
ROOT=Path(__file__).resolve().parents[3]
paths=['tools/reference/r/stages.R','tools/reference/r/run.py','tools/reference/r/renv.lock','tools/reference/r/manifest_stages.py','fixtures/parity/ggplot2/stages.json','fixtures/parity/ggplot2/sources.json']
corpus=json.loads((ROOT/paths[4]).read_text())
manifest={
    'schema_version':1,'owner':'GG-02','reference':'ggplot2 4.0.3','r_version':'4.6.1',
    'ggplot2_source_sha256':'690224bd61642b6222adb109470988e87f786e193cca77a15c0923cf9da73fa5',
    'seed':1729,'case_count':len(corpus['cases']),
    'case_ids':[v['id'] for v in corpus['cases']],
    'files':{p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in paths},
    'regenerate':'mise exec -- python3 tools/reference/r/run.py tools/reference/r/stages.R',
}
(ROOT/'fixtures/parity/ggplot2/stages-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('PASS GG-02 reference manifest:',manifest['case_count'],'cases; pinned source, runner, lock and fixture hashes.')
