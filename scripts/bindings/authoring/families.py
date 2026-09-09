"""Ordinary Python authoring for the independent statistic and scale/geometry corpus."""
import json

def dataset(c,case,index=0):
    batch=case['data']['datasets'][index]['batch']; columns={}
    for field,wire in zip(batch['fields'],batch['columns']):
        kind,values=next(iter(wire['values'].items()))
        if kind=='Categorical': col=c.categorical([values['dictionary'][i] for i in values['codes']])
        elif kind=='Timestamp': col=c.timestamps([int(v) for v in values],unit={'Seconds':'s','Milliseconds':'ms','Microseconds':'us','Nanoseconds':'ns'}[field['kind']['Timestamp']['unit']],timezone=field['kind']['Timestamp']['timezone'])
        else:
            kinds={'Float64':'float64','Int64':'int64','UInt64':'uint64','Boolean':'bool','Utf8':'string'}
            col=c.column([int(v) for v in values] if kind in ('Int64','UInt64') else values,kind=kinds[kind])
        col=col.validity(wire['validity']).nullable(field['nullable'])
        if wire.get('formatted') is not None: col=col.formatted(wire['formatted'])
        if field.get('unit') is not None: col=col.unit(field['unit'])
        if field.get('label') is not None: col=col.label(field['label'])
        columns[field['name']]=col
    return c.Data.columns(columns,name=f'data_{index}',keys=[int(k) for k in batch['keys']],schema_version=int(batch['schema_version']),identity=int(case['data']['datasets'][index]['id']))

def run(c, root, output, options, write):
    gray='#b4c8d78c'; blue='#1e7db4d2'; red='#dc4637d2'; missing='#80808046'
    semantics={}; scenes={}
    bold=output.register_font((root/'fixtures/composition/fonts/NotoSans-Bold.ttf').read_bytes())
    arabic=output.register_font((root/'fixtures/composition/fonts/NotoSansArabic-Regular.ttf').read_bytes())
    for family in ('statistics','families','facets','composition'):
        cases=json.loads((root/f'fixtures/{family}/portable-cases.json').read_text())
        for case in cases:
            name=case['name'];source=dataset(c,case)
            p=c.plot(source)
            if family=='statistics':
                if name in ('summary','transformed-summary'):
                    stat=c.summary().x('x').quantiles([0.,.25,.5,1.]).empty_sum_zero(False)
                    if name=='transformed-summary': stat=stat.transform(2.,10.)
                    layer=c.points().stat(stat).after_stat(c.stat_aes().x('Group').y('Mean'))
                elif name=='count':layer=c.points().stat(c.count().required(['x','y']).group('group')).after_stat(c.stat_aes().x('Group').y('Count'))
                elif name in ('ols','filtered-fit'):
                    layer=c.line().stat(c.fit().x('x').y('y')).after_stat(c.stat_aes().x('X').y('Y'))
                    if name=='filtered-fit':layer=layer.filter(c.filter('x').maximum(2.))
                elif name=='auto-bin':layer=c.histogram().aes(c.aes().x('x')).bins(30)
                elif name=='overflow-bin':layer=c.rectangle().stat(c.bin().x('x').breaks([0.,1.,2.]).outliers('Overflow')).after_bin(c.bin_aes().x('Start').x2('End').y('Count').y2(0.))
                elif name in ('stack','normalize'):layer=c.rectangle().aes(c.aes().x(0.).x2(1.).y('y').y2(0.).group('group')).position(c.stack([0,1,2,3]).normalize(name=='normalize'))
                elif name=='dodge':layer=c.rectangle().aes(c.aes().x('category').x2('category').y('y').y2(0.).group('group')).position(c.dodge([0,1,2]).width(.9))
                else:layer=c.points().aes(c.aes().x('x').y('y').group('group')).position(c.jitter(2**64-1).displacement(.25,.5).units('Data' if name=='jitter-data' else 'Display'))
                p=p.layer(layer.independent().color(gray))
            elif family=='families':
                p=p.aes(c.aes().x('field_1').y('field_2'))
                if name=='family-log-gaps':p=p.y_axis(c.y_axis().scale(c.scale_log(10.)));layer=c.line()
                elif name=='family-symlog':p=p.y_axis(c.y_axis().scale(c.scale_symlog(2.)));layer=c.points()
                elif name=='family-point-color':
                    p=p.x_axis(c.x_axis().scale(c.scale_point().categories(['Alpha','Beta','Gamma']).point_padding(.5))).scale(c.color_discrete('color').domain(['Alpha','Beta','Gamma']).palette([blue,red,missing]).missing(missing)).legend(c.legend().scale('color').generic_title())
                    layer=c.points().aes(c.aes().color('field_1').color_scale('color'))
                elif name=='family-area':layer=c.area().baseline(0.)
                elif name=='family-ribbon':layer=c.ribbon().aes(c.aes().y2('field_3'))
                elif name=='family-heatmap':
                    p=p.scale(c.color_continuous('color',0.,5.).palette([blue,red]).clamp(True).missing(missing)).legend(c.legend().scale('color').generic_title())
                    layer=c.cells().aes(c.aes().x2('field_3').y2('field_4').color('field_5').color_scale('color'))
                elif name in ('family-grouped-bars','family-stacked-bars'):
                    p=p.aes(c.aes().y('y').y2(0.).group('group')).scale(c.color_discrete('color').palette([blue,red,missing]).missing(missing)).legend(c.legend().scale('color').generic_title())
                    layer=(c.rectangle().aes(c.aes().x('category').x2('category')).position(c.dodge([0,1,2]).width(.9)) if name=='family-grouped-bars' else c.rectangle().aes(c.aes().x(0.).x2(1.)).position(c.stack([0,1,2,3])))
                    layer=layer.color_group('color').color(gray)
                elif name=='family-ohlc-volume':
                    p=p.y_axis(c.y_axis().name('volume').side('Right')).layer(c.ohlc().width(12.).aes(c.aes().y2('field_3').low('field_4').high('field_5')).color(blue))
                    layer=c.volume().width(7.).axes('x','volume').aes(c.aes().y('field_6')).color(missing)
                elif name=='family-secondary':p=p.y_axis(c.y_axis().name('fahrenheit').side('Right').secondary('y',1.8,32.));layer=c.points()
                elif name in ('family-session','family-utc-leap'):
                    scale=c.scale_utc().interval({'Days':1}) if name=='family-utc-leap' else c.scale_session({'id':'supplied-fixture-only','revision':'1','unit':'Milliseconds','sessions':[{'start':'1709164800000','end':'1709251200000'},{'start':'1709510400000','end':'1709596800000'}],'closed':'Omit'})
                    p=p.x_axis(c.x_axis().scale(scale)).aes(c.aes().x({'field':'field_1','origin':'1709164800000'}).y('field_2'));layer=c.line()
                else:raise AssertionError(name)
                if name not in ('family-grouped-bars','family-stacked-bars','family-ohlc-volume'):layer=layer.color(blue)
                p=p.layer(layer)
            elif family=='facets':
                panel=lambda text:{'values':[{'Text':text}]}
                facet=c.facet_wrap('facet').columns(2).order([['B'],['A']]).gap(12.).collect_guides(True).empty('Keep')
                if name=='facet-free-target':facet=facet.free_y(True)
                if name=='facet-grid-empty':facet=c.facet_grid('facet','group').order([[a,g] for a in ('B','A','C') for g in (2,1)]).gap(12.).collect_guides(True).empty('Keep')
                p=p.facet(facet)
                if name.endswith('summary'):
                    scope={'facet-group-summary':'Group','facet-panel-summary':'Facet','facet-chart-summary':'Chart'}[name]
                    layer=c.points().stat(c.summary().x('y').group('group').quantiles([.5]).empty_sum_zero(False)).after_stat(c.stat_aes().x('Group').y('Mean')).scope(scope).color(blue)
                    if scope=='Chart':layer=layer.facet_target('Broadcast')
                    p=p.layer(layer)
                else:
                    p=p.layer(c.points().aes(c.aes().x('x').y('y').group('group').color('group').color_scale('groups')).color(blue)).scale(c.color_discrete('groups').domain(['1','2']).palette(['#1e7db4e6','#da6928e6']).missing('#78787864')).legend(c.legend().scale('groups').title('Group'))
                    if name=='facet-shared-log':p=p.y_axis(c.y_axis().scale(c.scale_log(10.)))
                    if len(case['data']['datasets'])>1:
                        p=p.layer(c.rule().data(dataset(c,case,1)).aes(c.aes().x(0.).x2(2.).y('threshold').y2('threshold')).facet_target({'Panels':[panel('A')]} if name=='facet-free-target' else 'Broadcast').color('#8c465adc'))
            else:p=composition(c,name,source,dataset(c,case,1),bold,arabic)
            family_options=c.export_options(180,120,'mm').dpi(96) if family=='composition' else c.export_options(600,540).dpi(96) if name=='facet-grid-empty' else options
            authored=p.build();chart=authored.chart();semantics[name]=chart.semantics();frame=chart.present(output,family_options);scenes[name]=frame.scene()
            frame.dispose();chart.dispose();authored.dispose();source.dispose()
    write('families',semantics);write('family-scenes',scenes)

def composition(c,name,source,reference,bold,arabic):
    panel=lambda name:{'values':[{'Text':name}]}
    typography=lambda size:c.text_style().size(size)
    strong=lambda size:typography(size).font(bold).weight(700)
    rgb=lambda r,g,b:dict(red=r,green=g,blue=b,alpha=255)
    terminal=name=='composition-terminal'
    trace=c.line().aes(c.aes().x('x').y('y')).color('#1e7db4d2')
    threshold=c.rule().data(reference).aes(c.aes().x(0.).x2(2.).y('threshold').y2('threshold')).facet_target('Broadcast').color('#8c465adc')
    observations=c.points().aes(c.aes().x('x').y('y').group('group').color('group').color_scale('groups')).color('#1e7db4d2')
    theme=(c.theme().preset('Terminal' if terminal else 'Grayscale' if name=='composition-grayscale' else 'Editorial')
        .style(c.style().gradient(dict(direction='Vertical',start=rgb(27,40,56) if terminal else rgb(234,243,252),end=rgb(18,27,37) if terminal else rgb(255,255,255))))
        .layer(observations,c.style().symbol('Diamond')).layer(threshold,c.style().dashes([4.,2.]))
        .layer(trace,c.style().mark(rgb(117,187,242) if terminal else rgb(35,93,145)).stroke_width(1.5)))
    return (c.plot(source).layer(trace).layer(threshold).layer(observations)
        .scale(c.color_discrete('groups').domain(['1','2']).palette(['#1e7db4e6','#da6928e6']).missing('#78787864')).legend(c.legend().scale('groups').title('Group'))
        .facet(c.facet_wrap('facet').columns(2).order([['B'],['A']]).gap(12.))
        .x_axis(c.x_axis().text_style(typography(.7).tabular(True)).rich_label(c.rich_text('Observation x').style(typography(.75))).rotation(-35.).format(c.number_format().precision(1)))
        .y_axis(c.y_axis().text_style(typography(.7).tabular(True)).rich_label(c.rich_text('Value (units)').style(typography(.75)).rotation(-90.)).format(c.number_format().precision(0)))
        .title(c.title('').rich(c.rich_text('Small multiples').style(strong(1.4)).run(c.text_run('  ·  one prepared population').style(typography(.9)))))
        .subtitle(c.subtitle('').rich(c.rich_text('Explicit fonts and vector paint   ').style(typography(.72)).run(c.text_run('مرحبا بالعالم').style(typography(.8).language('ar').direction('RightToLeft').fallback(arabic)))))
        .caption(c.caption('Figure 1 · Shared scales; the inset reuses the same source rows.').style(typography(.72)))
        .source_note(c.source_note('Source: deterministic fixture. No external market data.').style(typography(.64)))
        .footnote(c.footnote('Notes: x/y are source units; dashed rule = supplied reference.').style(typography(.64)))
        .panel_letter(c.panel_letter('(a)').panel(panel('B')).style(strong(.8))).panel_letter(c.panel_letter('(b)').panel(panel('A')).style(strong(.8)))
        .inset(c.inset().id('detail-a').panel(panel('A')).rectangle(.52,.5,.45,.45).layer(trace).layer(observations).x_view(0.,1.).y_view(0.,4.).guides(False))
        .layer(c.callout().label(c.labels().id('peak').at(2.,9.).panel(panel('A')).text('Peak 9').style(typography(.68)).offset(-43.,15.).priority(10).collision('Keep')).to({'Data':{'panel':panel('A'),'scales':{'x':'0','y':'1'},'x':{'Number':2.},'y':{'Number':9.}}}))
        .layer(c.labels().id('units').figure_at(.73,.95).text('180 × 120 mm').style(typography(.6)).overflow(True))
        .layer(c.labels().id('lambda').output_at(430.,12.).text('λ = −0.25').style(typography(.64)).overflow(True))
        .layer(c.labels().id('direct-b').panel_at(panel('B'),.08,.4).text('Panel B').style(typography(.65)).collision('ShiftThenHide').priority(1))
        .theme(theme))
