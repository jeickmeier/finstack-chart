"""Independent capture identities and supplied-value projection oracle for FIX-14."""
import json,re,xml.etree.ElementTree as ET

def check(paths,same):
    traces=[json.loads((p/'live-export.json').read_text()) for p in paths]
    for trace in traces[1:]:same(traces[0],trace,1e-10,'live-export')
    records={r['name']:r for r in traces[0]};assert len(records)==40
    for r in traces[0]:
        if r['status'] is not None:
            m=r['status']['metrics'];assert m['pending']+m['running']<=2 and m['peak_jobs']<=2
    expected=[('older-last',0,'Before capture',8,49,False,(400,200)),('newer-first',1,'After one',9,100,True,(600,300)),('clean-committed',1,'After one',9,100,False,(400,200)),('included-transient',1,'Uncommitted preview',9,100,False,(400,200)),('latest',2,'After one',9,200,True,(400,200)),('frozen-historical',2,'After one',9,200,True,(400,200)),('current-during-freeze-result',3,'After one',9,300,True,(400,200))]
    for name,revision,label,count,extent,full,size in expected:
        record=records[name];metadata=record['status']['last_export']['metadata'];assert metadata['stamp']['store']==str(revision)
        assert all(v[0]['revision']==str(revision) and v[1]=='1' for v in metadata['datasets'])
        assert metadata['source_epoch']=='9007199254746001' and metadata['fonts'][0]['sha256']=='2ec33f84606cbaa0a1a944488e14f97faf2f6a25ecdd8354f5358f06da13c7d9'
        assert metadata['profile']['view']==('FullDomain' if full else 'VisibleView')
        effective=metadata['effective_state'];assert effective['effective_annotations'][0]['text']['lines'][0][0]['text']==label
        assert effective['viewport']['x']==(None if full else [2,5])
        if name!='included-transient':assert effective['interaction']['selection']==[] and effective['hover']==[] and effective['focus'] is None and effective['active_gesture'] is None
        else:assert len(effective['interaction']['selection'])==1 and effective['active_gesture'] is not None
        data=(paths[0]/f'live-{name}.svg').read_bytes()
        for p in paths[1:]:assert data==(p/f'live-{name}.svg').read_bytes(),name
        root=ET.fromstring(data);assert (root.get('width'),root.get('height'))==(f'{size[0]}pt',f'{size[1]}pt')
        assert label in [n.get('aria-label') for n in root.iter()]
        clip=root.find(".//*[@id='clip-1']")[0];assert clip.tag.endswith('rect')
        left,top,width,height=[float(clip.get(k)) for k in ['x','y','width','height']]
        xmin,xmax=(0,count-1) if full else (2,5)
        for series,sign in [(1,1),(2,-1)]:
            path=root.find(f".//*[@id='item-{series}']");commands=path.get('d');assert set(re.findall('[A-DF-Za-df-z]',commands))<=set('ML')
            numbers=[float(x) for x in re.findall(r'[-+]?(?:\d*\.\d+|\d+\.?\d*)(?:[eE][-+]?\d+)?',commands)];assert len(numbers)==count*2
            for i in range(count):
                y=sign*(100*revision if i==3 and revision else i*i)
                x_expected=left+width*(i-xmin)/(xmax-xmin);y_expected=top+height*(extent-y)/(2*extent)
                assert abs(numbers[2*i]-x_expected)<=1e-9 and abs(numbers[2*i+1]-y_expected)<=1e-9,(name,series,i)
    assert records['older-last']['live']['store']=='1' and records['clean-committed']['live']['store']=='2'
    assert records['capacity-rejected']['result']=={'error':'CHART_RESOURCE_LIMIT'}
    assert records['cancelled-unavailable']['result']=={'error':'CHART_DISPOSED_HANDLE'}
    assert records['raster-limit']['result']=={'error':'CHART_RESOURCE_LIMIT'}
    assert records['disposed-chart']['result']=={'error':'CHART_DISPOSED_HANDLE'}
    m=records['status']['status']['metrics'];assert (m['pending'],m['running'],m['rows'],m['input_bytes'])==(0,0,0,0)
    assert (m['submitted'],m['completed'],m['cancelled'],m['failed'],m['rejected'])==('9','7','1','1','1')
    print('PASS WP-20 FIX-14: 40 actual Rust/Python/WASM capture steps; coherent atomic dataset revisions; independent supplied-value projection; clean/preview annotation labels; visible/full dimensions; bounded errors/cancellation; byte-identical exports after disposal')
