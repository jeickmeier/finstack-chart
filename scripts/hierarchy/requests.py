#!/usr/bin/env python3
"""Translate oracle inputs into public descriptors. Never calculate layout expectations."""
import copy,json,pathlib,sys
ROOT=pathlib.Path(__file__).resolve().parents[2]
def reg(mode):return {"operation":{"id":"example.hierarchy","version":"1"},"parameters":{"mode":mode}}
def nested(data, key=[0]):
    key[0]+=1;k=data.get('key',str(key[0]))
    return {"key":k,"data":data,"children":[nested(c,key) for c in data.get('children',[])]}
def envelope(c):
    data,op=c['input'],c['op']
    if op=='Node':kind={'Node':{'key':'1','data':data}}
    elif op=='stratify':
        path=c['config'].get('path',False)
        kind={'Stratify':{'rows':[{'key':str(i+1),'data':{'path':r} if path else r} for i,r in enumerate(data)],'options':{'path_field':'path'} if path else {}}}
    elif c['id']=='ordered-grouped':
        nextkey=1
        def group(pair):
            nonlocal nextkey
            nextkey+=1;k=str(nextkey);name,value=pair
            return {'key':k,'label':name,'value':None if isinstance(value,list) else value,'children':[group(c) for c in value] if isinstance(value,list) else []}
        kind={'Grouped':{'root':'1','entries':[group(x) for x in data]}}
    elif c['id']=='custom-iterable':
        raw=copy.deepcopy(data);nextkey=0
        def assign(n):
            nonlocal nextkey
            nextkey+=1;n['key']=str(nextkey)
            for c in n.get('children',[]):assign(c)
        assign(raw);kind={'Children':{'root':{'key':'1','data':raw},'accessor':reg('Children')}}
    else:kind={'Nested':nested(data,[0])}
    return {'version':1,'identity':'1','input':kind}
def tiler(config):
    name=config.get('tile','treemapSquarify').removeprefix('treemap')
    if name in ('Squarify','Resquarify'):return {name:config.get('ratio',1.618033988749895)}
    return 'Custom' if name=='customEqual' else name
def layout(op,c):
    if op in ('tree','cluster'):
        options={}
        for a,b in [('size','Extent'),('nodeSize','NodeSize')]:
            if a in c:options['mode']={b:c[a]}
        if c.get('separation')=='constant':options['separation']={'Constant':c['distance']}
        # Exercise the external registered path for depth-aware separation.
        return {op.capitalize():{'options':options,'separation':reg('DepthSeparation') if c.get('separation')=='depth' else None}}
    if op=='partition':return {'Partition':c}
    if op=='treemap':
        options={'tile':tiler(c)}
        for a,b in [('size','size'),('round','round'),('paddingInner','padding_inner'),('paddingTop','padding_top'),('paddingRight','padding_right'),('paddingBottom','padding_bottom'),('paddingLeft','padding_left')]:
            if a in c:options[b]=c[a]
        if 'padding' in c:
            for p in ('inner','top','right','bottom','left'):options.setdefault('padding_'+p,c['padding'])
        return {'Treemap':{'options':options,'history':bool(c.get('history',False)),'padding':{'Registered':reg('DepthPadding')} if c.get('paddingAccessor') else None,'tiler':reg('EqualTile') if options['tile']=='Custom' else None}}
    if op=='pack':
        options={k:c[k] for k in ('size','padding') if k in c}
        if 'radius' in c:options['radius']='Explicit'
        return {'Pack':{'options':options,'radius':{'Registered':reg('Value')} if 'radius' in c else None,'padding':{'Registered':reg('DepthPadding')} if c.get('paddingAccessor') else None}}
    raise ValueError(op)
def requests(c):
    steps=[]
    def step(action,data=None,out=None,session='main',**extra):
        item={'action':action,'session':session,**extra}
        if data is not None:item['data']=data
        if out is not None:item['out']=out
        steps.append(item)
    op=c['op'];root={'hierarchy':'1','node':'1'}
    if op in ('packSiblings','packEnclose'):
        circles=[{'x':0,'y':0,'r':r} for r in c['input']] if op=='packSiblings' else c['input']
        step('packing',{'version':1,'siblings':op=='packSiblings','circles':circles},'result')
        return steps
    step('construct',envelope(c))
    if op in ('Node','hierarchy','stratify'):step('query','Nodes','nodes')
    elif op=='operations':
        step('apply',{'Sum':{'Registered':reg('Value')}});step('query','Nodes','sum')
        for field,order in [('before','PreOrder'),('after','PostOrder'),('each','BreadthFirst')]:step('query',{'Visit':{'root':root,'order':order}},field)
        for field,query in [('iterator',{'Descendants':root}),('leaves',{'Leaves':root}),('links',{'Links':root}),('find',{'FindRegistered':{'root':root,'predicate':reg('FindDepthOne')}}),('noMatch',{'Find':{'root':root,'field':'name','value':'absent'}})]:step('query',query,field)
        C={'hierarchy':'1','node':'3'};B={'hierarchy':'1','node':'5'};A={'hierarchy':'1','node':'2'}
        step('query',{'Ancestors':C},'ancestors');step('query',{'Path':{'start':C,'end':B}},'path');step('query',{'Path':{'start':C,'end':C}},'selfPath')
        step('copy_subtree',{'node':A,'identity':'2'},target='subtree');step('query','Nodes','copy',session='subtree')
        step('clone',target='old');step('apply','Count');step('query','Nodes','count')
        step('apply',{'Sort':{'Registered':reg('DescendingValue')}},session='old');step('query','Nodes','sort',session='old')
    elif op in ('history','historyTopology'):
        count=len(c['expected'])
        step('clone',target='original')
        for i in range(count):
            current=copy.deepcopy(c['input'])
            ratio=c['config'].get('ratio',1.618033988749895)
            if op=='historyTopology':current=c['config']['steps'][i]['tree'];ratio=c['config']['steps'][i]['ratio'];size=c['config']['size']
            else:
                if i:
                    for leaf in current['children']:
                        n=int(leaf['name'][1:]);leaf['value']=21-n if i==1 else (20 if n%2 else 1)
                size=[[80,40],[40,80],[100,100],[100,100]][i]
            step('apply',{'Replace':envelope({'id':'history-step','op':'hierarchy','input':current,'config':{}})})
            step('apply',{'Sum':{'Field':'value'}})
            if op=='history' and i==3:step('apply','ResetHistory')
            step('apply',{'Layout':layout('treemap',{'size':size,'tile':'treemapResquarify','ratio':ratio,'history':True})})
            step('query','Nodes',f'step{i}')
            step('snapshot',out=f'snapshot{i}')
            step('restore') # roundtrip history before the next update
        step('query','Nodes','original',session='original')
    else:
        step('apply',{'Sum':{'Field':'value'}})
        if op=='tile':step('tile',{'version':1,'parent':root,'bounds':c['config']['bounds'],'tiler':tiler(c['config']),'history':False,'operation':None},'tiles');step('query','Nodes','nodes')
        else:step('apply',{'Layout':layout(op,c['config'])});step('query','Nodes','nodes')
    step('snapshot',out='snapshot');step('restore');step('snapshot',out='roundtrip')
    return steps
if __name__=='__main__':
    cases=json.loads((ROOT/'fixtures/hierarchy/reference.json').read_text())['cases']
    result=[{'id':c['id'],'steps':requests(c)} for c in cases]
    path=pathlib.Path(sys.argv[1]);path.parent.mkdir(parents=True,exist_ok=True);path.write_text(json.dumps(result,separators=(',',':'))+'\n')
    print(f'{len(result)} public hierarchy request sequences -> {path}')
