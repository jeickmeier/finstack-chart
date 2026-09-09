// FIX-S07: independently generated stack orders/offsets, missing cells and identities.
import * as d3 from 'd3-shape';
import fs from 'node:fs';
import path from 'node:path';
import {root,provenance,writeJson,digest} from './provenance.mjs';
const out=path.resolve(process.argv[2]||path.join(root,'fixtures/shapes'));
const orders=['None','Reverse','Ascending','Descending','Appearance','InsideOut'],offsets=['None','Expand','Diverging','Silhouette','Wiggle'];
const inputs=[
 ['positive',[[1,4,2],[3,1,5],[2,6,1],[4,2,3]]],
 ['mixed',[[2,-1,0],[-4,2,1],[0,-3,5],[1,1,-2]]],
 ['discriminator',[[2,-1]]],
 ['zero',[[0,0,0],[0,0,0]]],
 ['ties',[[1,1,1],[3,3,3],[1,1,1]]],
 ['peaks',[[9,1,0,1],[1,8,0,7],[1,1,9,1],[0,0,0,0]]],
 ['missing',[[2,null,1],[null,-2,4],[0,1,null]]],
 ['one-series',[[1],[0],[-2],[4]]],
 ['one-sample',[[3,1,2]]],
 ['empty-samples',[]],
 ['empty-series',[[],[],[]]],
 ['tiny',[[1e-100,3e-100],[4e-100,1e-100]]],
 ['large',[[1e100,3e100],[4e100,1e100]]],
];
const cases=[];
for(const [name,matrix]of inputs)for(const order of orders)for(const offset of offsets)for(const missing of name==='missing'?['Gap','Zero']:['Gap']){
 const n=name==='empty-samples'?3:matrix[0].length,keys=Array.from({length:n},(_,i)=>'series-'+i),data=matrix.map((values,i)=>({id:String(9007199254741001n+2n*BigInt(i)),values}));
 const g=d3.stack().keys(keys).value((d,key)=>{const v=d.values[Number(key.slice(7))];return v===null?(missing==='Zero'?0:NaN):v;}).order(d3['stackOrder'+order]).offset(d3['stackOffset'+offset]);
 const series=g(data).map(s=>({key:s.key,index:s.index,points:s.map(p=>({data:p.data,y0:p[0],y1:p[1]}))}));cases.push({id:`${name}-${order}-${offset}-${missing}`,matrix,keys,order,offset,missing,series});
}
for(const permutation of [[2,0,1],[1,2,0],[0,1,2]])for(const offset of offsets){
 const matrix=[[1,3,2],[4,2,1]],keys=['a','b','c'],data=matrix.map((values,i)=>({id:String(9007199254741001n+2n*BigInt(i)),values})),series=d3.stack().keys(keys).value((d,k)=>d.values[keys.indexOf(k)]).order(permutation).offset(d3['stackOffset'+offset])(data).map(s=>({key:s.key,index:s.index,points:s.map(p=>({data:p.data,y0:p[0],y1:p[1]}))}));cases.push({id:`explicit-${permutation.join('')}-${offset}`,matrix,keys,order:{Explicit:permutation},offset,missing:'Gap',series});
}
writeJson(path.join(out,'stack.json'),JSON.parse(JSON.stringify({schema_version:1,orders,offsets,cases},(_,v)=>typeof v==='number'&&(Object.is(v,-0)||!Number.isFinite(v))?{number:Object.is(v,-0)?'-0':String(v)}:v)));
writeJson(path.join(out,'stack-manifest.json'),{schema_version:1,references:[provenance('d3-shape')],cases:cases.length,generator_sha256:digest(fs.readFileSync(new URL(import.meta.url))),cases_sha256:digest(fs.readFileSync(path.join(out,'stack.json')))});
console.log('PASS independent stack oracle:',cases.length,'cases; acceptance requires implementation replay.');
