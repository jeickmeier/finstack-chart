"""FIX-GG02 publication validation; run with Pillow and Poppler installed.

python ggplot_stage_publication.py PRIMARY_PROOF/stages OUTPUT_DIRECTORY
Visual inspection of the generated contact sheets remains separately recorded.
"""
from pathlib import Path
from PIL import Image,ImageChops,ImageDraw
import json,subprocess,sys
root=Path(__file__).resolve().parents[2]
base=Path(sys.argv[1]).resolve()
out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=True)
rendered=out/'pdf-rendered-final';rendered.mkdir(exist_ok=True)
records=[];pngs=[];pdfs=[]
for p in sorted((base/'python').glob('*.png')):
    a=Image.open(p).convert('RGBA')
    for host in ('rust','wasm'):
        b=Image.open(base/host/p.name).convert('RGBA');assert a.size==b.size
        assert all(v==(0,0) for v in ImageChops.difference(a,b).getextrema()),(p.name,host)
    pdf=p.with_suffix('.pdf');font=subprocess.check_output(['pdffonts',str(pdf)],text=True)
    assert 'NotoSans' in font and ' yes ' in font
    subprocess.run(['pdftoppm','-r','96','-singlefile','-png',str(pdf),str(rendered/p.stem)],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
    pngs.append((p.stem,a.copy()));pdfs.append((p.stem,Image.open(rendered/p.name).convert('RGBA')))
    records.append(dict(id=p.stem,size=a.size,three_host_rgba_equal=True,pdf_fonts=font))
for name,images in [('contact',pngs),('pdf-contact',pdfs)]:
    for part in range(0,len(images),8):
        group=images[part:part+8];canvas=Image.new('RGB',(840,306*((len(group)+1)//2)),'#ddd');draw=ImageDraw.Draw(canvas)
        for i,(label,im) in enumerate(group):
            im.thumbnail((420,280));x=(i%2)*420;y=(i//2)*306;draw.text((x+8,y+5),label,fill='black');canvas.paste(im,(x,y+26))
        canvas.save(out/f'{name}-{part//8+1}.png')
(out/'publication-checks-final.json').write_text(json.dumps(records,indent=2)+'\n')
print('PASS',len(records),'figures: exact Rust/Python/WASM RGBA; supplied font embedded in all PDFs; contact sheets ready for inspection')
