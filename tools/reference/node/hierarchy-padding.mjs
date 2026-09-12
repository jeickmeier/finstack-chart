// Additional HIR-06 per-side accessor coverage; pinned development-only d3-hierarchy.
import * as d3 from 'd3-hierarchy';
import fs from 'node:fs';
import crypto from 'node:crypto';
const rows=[['1',null,5,3],['2','1',2,4],['3','1',3,5],['4','2',4,6],['5','2',6,7]].map(([key,parent,value,pad])=>({key,parent,data:{value,pad}}));
const kinds={Binary:d3.treemapBinary,Dice:d3.treemapDice,Squarify:d3.treemapSquarify,Resquarify:d3.treemapResquarify};
const cases=[];
for(const [kind,tile] of Object.entries(kinds))for(const side of ['Inner','Top','Right','Bottom','Left','Outer'])for(const accessor of ['Depth','Field','Registered']) {
 const root=d3.stratify().id(r=>r.key).parentId(r=>r.parent)(rows).sum(r=>r.data.value);
 const fn=d3.treemap().size([80,40]).tile(tile).padding(1);
 fn['padding'+side](n=>accessor==='Depth'?n.depth:accessor==='Field'?n.data.data.pad:n.depth+1);fn(root);
 const scalar=accessor==='Depth'?'Depth':accessor==='Field'?{Field:'pad'}:{Registered:{operation:{id:'example.hierarchy',version:'1'},parameters:{mode:'DepthPadding'}}},padding_sides=Object.fromEntries((side==='Outer'?['Top','Right','Bottom','Left']:[side]).map(s=>[s,scalar]));
 const layout={Treemap:{options:{size:[80,40],tile:['Squarify','Resquarify'].includes(kind)?{[kind]:(1+Math.sqrt(5))/2}:kind},history:kind==='Resquarify',padding:{Constant:1},padding_sides}};
 cases.push({id:`${kind}-${side}-${accessor}`,input:{version:1,identity:'42',input:{Rows:rows}},layout,expected:Array.from(root,n=>({key:n.id,parent:n.parent?.id??null,children:(n.children??[]).map(c=>c.id),value:n.value,geometry:{Rectangle:{x0:n.x0,y0:n.y0,x1:n.x1,y1:n.y1}}}))});
}
const path=process.argv[2]??'fixtures/hierarchy/padding-accessors.json';const bytes=JSON.stringify({version:1,package:'d3-hierarchy',package_version:'3.1.2',cases},null,2)+'\n';fs.writeFileSync(path,bytes);
const sha=value=>crypto.createHash('sha256').update(value).digest('hex');
fs.writeFileSync(path.replace('.json','-manifest.json'),JSON.stringify({version:1,package:'d3-hierarchy',package_version:'3.1.2',node:process.version,generator_sha256:sha(fs.readFileSync(new URL(import.meta.url))),fixture_sha256:sha(bytes),cases:cases.length},null,2)+'\n');console.log(`${cases.length} independent per-side padding accessor cases`);
