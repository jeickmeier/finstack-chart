"""GG17 actual Python save policy and owned custom-device callback controls."""
import json,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];sys.path[:0]=[str(ROOT/'packages/python'),str(Path(sys.argv[1]).resolve())]
import finstack_chart as c
out=Path(sys.argv[2]);out.mkdir(parents=True,exist_ok=True)
output=c.Output((ROOT/'fixtures/capability/fonts/NotoSans-Regular.ttf').read_bytes())
records=[]
for case in json.loads((ROOT/'fixtures/parity/ggplot2/device-controls.json').read_text())['dimensions']:
    try:
        plan=output.resolve_save('figure.PNG',case['input'])
        assert 'error' not in case['result']
        assert abs(plan['page']['width']/72-case['result'][0])<1e-12
        assert abs(plan['page']['height']/72-case['result'][1])<1e-12
        records.append(plan)
    except c.ChartError:
        assert 'error' in case['result'];records.append({'error':True})
data=c.Data.columns(dict(x=[0.,1.],y=[1.,2.]));plot=c.plot(data).aes(c.aes().x('x').y('y')).layer(c.points()).build()
with tempfile.TemporaryDirectory(prefix='chart-save-') as directory:
    target=Path(directory)/'nested/fig-%02d.png'
    options=dict(width=144.,height=72.,units='px',dpi=72,create_dir=True)
    plan=output.save_figure(plot,target,options,page_number=3)
    assert Path(plan['path']).read_bytes()[:8]==b'\x89PNG\r\n\x1a\n'
    def custom(frame):
        assert frame.scene()['items'];return b'custom-device'
    plan=output.save_figure(plot,Path(directory)/'custom.dat',dict(width=2.,height=1.,device='custom'),device=custom)
    assert Path(plan['path']).read_bytes()==b'custom-device'
    sentinel=RuntimeError('custom failure')
    def failed(frame): raise sentinel
    try: output.save_figure(plot,Path(directory)/'failed.dat',dict(width=2.,height=1.,device='custom'),device=failed);raise AssertionError('callback passed')
    except RuntimeError as error: assert error is sentinel
    assert not (Path(directory)/'failed.dat').exists()
    try: output.save_figure(plot,Path(directory)/'oversize.dat',dict(width=2.,height=1.,device='custom'),device=lambda frame:bytes(4194305),max_bytes=4194304);raise AssertionError('budget passed')
    except ValueError:pass
    assert not (Path(directory)/'oversize.dat').exists()
(out/'records.json').write_text(json.dumps(records));plot.dispose();data.dispose();output.dispose()
print('PASS Python save dimensions, filename inference, writes, callback errors and budgets')
