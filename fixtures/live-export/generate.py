"""FIX-14 exact dual-dataset capture replay; expected populations are authored independently."""
import copy,json
from pathlib import Path
root=Path(__file__).resolve().parents[2]
chart=json.loads((root/'fixtures/actions/chart.json').read_text())
line=copy.deepcopy(chart['definition']['layers'][1]);line['id']='1';line['data']={'Dataset':'1'}
other=copy.deepcopy(line);other['id']='2';other['data']={'Dataset':'2'};other['style']['color']={'red':190,'green':60,'blue':65,'alpha':255}
chart['definition']['layers']=[line,other];chart['definition']['revision']='1'
chart['definition']['figure']['annotations'][0]['text']['lines'][0][0]['text']='Before capture'
fields=[{'id':'1','name':'x','kind':'Float64','nullable':False,'unit':None,'label':None},{'id':'2','name':'y','kind':'Float64','nullable':False,'unit':None,'label':None}]
origin=9007199254743001
def batch(rows):return {'schema_version':'1','fields':fields,'keys':[str(origin+i) for i,x,y in rows],'columns':[{'values':{'Float64':[r[k] for r in rows]},'validity':[True]*len(rows)} for k in (1,2)]}
data={'version':1,'epoch':'9007199254746001','datasets':[{'id':str(d),'batch':batch([(i,i,sign*i*i) for i in range(8)])} for d,sign in [(1,1),(2,-1)]]}
steps=[]
def step(name,kind,**kw):steps.append(dict(name=name,kind=kind,**kw))
def act(name,action):step(name,'action',action=action)
def begin(name,**kw):step(name,'begin',options=dict(format='svg',**kw))
def export(name,job,error=None):step(name,'export',job=job,**({'error':error} if error else {}))
def annotation(label):
 a=copy.deepcopy(chart['definition']['figure']['annotations'][0]);a['text']['lines'][0][0]['text']=label;return a
def transaction(n):
 rows=[(3,3,100*n)]+([(8,8,64)] if n==1 else [])
 return {'version':1,'id':f'live-export-{n}','epoch':data['epoch'],'expected':[{'dataset':str(d),'revision':str(n-1),'schema_version':'1'} for d in (1,2)],'operations':[{'dataset':str(d),'mutation':{'UpsertByKey':batch([(i,x,sign*y) for i,x,y in rows])}} for d,sign in [(1,1),(2,-1)]]}
act('viewport',{'SetViewport':{'x':[2,5],'y':None}})
act('select',{'Select':{'change':'Replace','targets':[{'epoch':data['epoch'],'layer':'1','panel':None,'identity':{'Source':{'dataset':'1','key':str(origin+3)}}}]}})
step('present-before','present');begin('visible-before')
step('atomic-one','transaction',transaction=transaction(1));act('edit-committed',{'SetAnnotation':annotation('After one')})
begin('current-full',basis='Current',full_domain=True,width_pt=600,height_pt=300,output_theme={'color_mode':'Grayscale'})
step('capacity-rejected','begin',options={'format':'svg'},error='CHART_RESOURCE_LIMIT')
export('newer-first','current-full');step('present-after-one','present');export('older-last','visible-before')
step('begin-cancel','begin',options={'format':'png'});step('cancel-pending','cancel',job='begin-cancel');export('cancelled-unavailable','begin-cancel','CHART_DISPOSED_HANDLE')
step('begin-raster-limit','begin',options={'format':'png','dpi':100000});export('raster-limit','begin-raster-limit','CHART_RESOURCE_LIMIT')
act('begin-preview',{'BeginGesture':{'id':'1','kind':{'Annotation':'threshold'}}})
act('preview-annotation',{'PreviewGesture':{'id':'1','preview':{'Annotation':annotation('Uncommitted preview')}}})
step('present-preview','present');begin('clean-preview');begin('include-preview',interaction={'selection':True,'hover':True,'focus':True,'preview':True})
act('cancel-preview',{'CancelGesture':'Explicit'});step('atomic-two','transaction',transaction=transaction(2))
export('clean-committed','clean-preview');export('included-transient','include-preview')
step('present-latest','present');begin('latest-full',basis='Current',full_domain=True);export('latest','latest-full')
act('freeze',{'SetFollow':'FreezePresentation'});step('atomic-three','transaction',transaction=transaction(3));begin('frozen-full',full_domain=True);begin('current-during-freeze',basis='Current',full_domain=True);export('frozen-historical','frozen-full');export('current-during-freeze-result','current-during-freeze');act('resume', 'ResumeLatest');step('present-after-freeze','present')
step('status','status');begin('held-at-disposal');step('dispose','dispose');export('disposed-chart','held-at-disposal','CHART_DISPOSED_HANDLE')
(root/'fixtures/live-export/replay.json').write_text(json.dumps({'chart':chart,'data':data,'steps':steps},indent=2)+'\n')
