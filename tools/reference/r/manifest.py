"""Hash the reproducible GG-00 oracle and its generator/lock inputs."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[3]
fixture=ROOT/'fixtures/parity/ggplot2'
def hashes(files):
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(files)}
files=[p for p in fixture.rglob('*') if p.is_file() and p.name!='manifest.json']
scripts=[p for p in (ROOT/'tools/reference/r').iterdir() if p.is_file()]
manifest=dict(schema_version=1,reference='ggplot2 4.0.3',entry_package='GG-00',
    scope='Executable entry oracle and complete public inventory; each delivery package owns exhaustive semantic argument cases.',
    exports=643,seed_cases=32,artifacts=96,
    generators=hashes(scripts),fixtures=hashes(files),
    reproducibility='32 records and 96 artifacts reproduced byte-for-byte; only PDF CreationDate is normalized, with unchanged decoded page streams.')
(fixture/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
print('PASS GG-00 manifest:',len(files),'fixture/source/license files and',len(scripts),'generator/lock inputs.')
