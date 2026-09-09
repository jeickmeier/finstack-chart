from pathlib import Path
from PIL import Image,ImageChops
import json,subprocess
out=Path('target/shape-arc');visual=out/'visual';visual.mkdir(exist_ok=True)
records=[]
for dpi in [300,600]:
 expected=Image.open(out/'rust'/f'figure-{dpi}.png').convert('RGBA')
 for host in ['python','wasm']:
  image=Image.open(out/host/f'figure-{dpi}.png').convert('RGBA');assert image.size==expected.size and all(t==(0,0) for t in ImageChops.difference(image,expected).getextrema())
 pdf=out/'rust'/f'figure-{dpi}.pdf';fonts=subprocess.check_output(['pdffonts',str(pdf)],text=True);assert 'NotoSans' in fonts and ' yes ' in fonts
 subprocess.run(['pdftoppm','-r','96','-singlefile','-png',str(pdf),str(visual/f'pdf-{dpi}')],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
 records.append({'dpi':dpi,'three_host_rgba_equal':True,'pdf_fonts':fonts,'size':expected.size})
 expected.thumbnail((1020,960));expected.save(visual/f'png-{dpi}.png')
subprocess.run(['/private/tmp/wp08-render-probe/target/debug/sp07-render-probe',str(out/'rust/figure-300.svg'),str(visual/'svg.png')],check=True)
subprocess.run(['/private/tmp/wp08-render-probe/target/debug/sp07-render-probe',str(out/'wasm/figure-300.svg'),str(visual/'svg-wasm.png')],check=True)
a=Image.open(visual/'svg.png').convert('RGBA');b=Image.open(visual/'svg-wasm.png').convert('RGBA');assert a.size==b.size and all(t==(0,0) for t in ImageChops.difference(a,b).getextrema())
(visual/'checks.json').write_text(json.dumps(records,indent=2)+'\n');print('PASS host RGBA and PDF fonts; SVG/PDF/PNG ready for visual inspection.')
