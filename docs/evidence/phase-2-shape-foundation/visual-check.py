from pathlib import Path
from PIL import Image,ImageChops,ImageDraw
import json,subprocess
root=Path.cwd();out=root/'target/shape-foundation';rust=root/'target/path-proof/linux-target/shape-foundation/rust';py=root/'target/path-proof/linux-target/shape-foundation/python';wasm=out/'wasm';visual=out/'visual';visual.mkdir(parents=True,exist_ok=True)
records=[]
for dpi in [300,600]:
 png=f'figure-{dpi}.png';expected=Image.open(rust/png).convert('RGBA')
 for name,folder in [('python',py),('wasm',wasm)]:
  actual=Image.open(folder/png).convert('RGBA');assert actual.size==expected.size and all(t==(0,0) for t in ImageChops.difference(actual,expected).getextrema()),(dpi,name)
 ratio=dpi/72
 for x,y,color in [(235,140,(255,255,255,255)),(270,140,(35,97,166,255)),(100,125,(35,97,166,255)),(70,125,(255,255,255,255))]:
  assert expected.getpixel((round(x*ratio),round(y*ratio)))==color,(dpi,x,y,expected.getpixel((round(x*ratio),round(y*ratio))))
 pdf=rust/f'figure-{dpi}.pdf';fonts=subprocess.check_output(['pdffonts',str(pdf)],text=True);assert 'NotoSans'in fonts and ' yes 'in fonts
 subprocess.run(['pdftoppm','-r','96','-singlefile','-png',str(pdf),str(visual/f'pdf-{dpi}')],check=True,stdout=subprocess.DEVNULL,stderr=subprocess.PIPE)
 records.append({'dpi':dpi,'three_host_rgba_equal':True,'analytic_sector_hole_pixels':True,'pdf_fonts':fonts,'size':expected.size})
 expected.thumbnail((1080,560));expected.save(visual/f'png-{dpi}.png')
(visual/'checks.json').write_text(json.dumps(records,indent=2)+'\n');print('PASS two resolutions, exact 3-host RGBA, independent interior/hole pixels and embedded supplied fonts; images ready for inspection.')
