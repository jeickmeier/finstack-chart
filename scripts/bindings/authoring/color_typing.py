"""Positive standalone color consumers; color math remains in core."""
import finstack_chart as c

parsed=c.color('steelblue')
if parsed is not None:
    copied=parsed.copy({'opacity':0.5}).convert('Lab')
    channel:float=copied.channel('l')
    text:str=copied.format_hsl()
    c.ColorValue.from_json(copied.to_json())
    c.rgb(copied)
c.rgb(1,2,3,opacity=0.5).brighter().clamp()
c.hsl(120,0.5,0.2).darker(-1)
c.lab(50,30,40).convert('Hcl')
c.hcl(30,40,50)
c.lch(50,40,30)
c.gray(50,opacity=0.5)
c.cubehelix(30,0.5,0.2).displayable()

# Every public paint alias accepts owned color values.
paint=c.lab(50.,20.,30.)
c.points().color(paint).style(c.style().mark(paint).panel(paint))
c.text_style().color(paint)
c.theme().geometry(ink=paint,paper=paint,accent=paint)
c.color_continuous('ramp',0.,1.).palette([paint,'red']).missing(paint)
c.color_discrete('group').palette([paint]).missing(paint)
c.export_options(400.,300.).background(paint)
