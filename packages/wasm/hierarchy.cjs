'use strict';
// Descriptor syntax and owned handles only; all operations execute in Rust.
module.exports=function(native,Owned,ShapeRegistry){
  function encode(value){
    const pending=[[value,0]],seen=new WeakSet();let count=0;
    while(pending.length){
      const [v,depth,leaving]=pending.pop();if(leaving){seen.delete(v);continue;}if(++count>200000||depth>32)throw new RangeError('Hierarchy descriptor exceeds its node/depth budget.');
      if(v===null||typeof v==='string'||typeof v==='boolean')continue;
      if(typeof v==='number'){if(!Number.isFinite(v))throw new RangeError('Hierarchy descriptors require finite numbers.');continue;}
      if(typeof v!=='object'||(!Array.isArray(v)&&Object.getPrototypeOf(v)!==Object.prototype&&Object.getPrototypeOf(v)!==null))throw new TypeError('Hierarchy descriptors require plain JSON values; identity fields use decimal strings.');
      if(seen.has(v))throw new TypeError('Hierarchy descriptors must be acyclic.');seen.add(v);pending.push([v,depth,true]);
      const values=Array.isArray(v)?Array.from(v):Object.values(v);if(values.length+pending.length+count>200000)throw new RangeError('Hierarchy descriptor exceeds its node budget.');
      for(const x of values)pending.push([x,depth+1]);
    }
    return JSON.stringify(value);
  }
  class Hierarchy extends Owned{
    constructor(envelope,registry){super(registry===undefined?new native._Hierarchy(encode(envelope)):native._Hierarchy.registered(encode(envelope),registry._inner));}
    static _wrap(inner){const value=Object.create(Hierarchy.prototype);return Object.assign(value,new Owned(inner));}
    static from_json(value,registry){if(registry!==undefined)return Hierarchy._wrap(native._Hierarchy.from_snapshot(value,registry._inner));const empty=new ShapeRegistry();try{return Hierarchy._wrap(native._Hierarchy.from_snapshot(value,empty._inner));}finally{empty.free();}}
    to_json(){return this._inner.to_json();}
    copy(){return Hierarchy._wrap(this._inner.copy());}
    copy_subtree(node,identity){return Hierarchy._wrap(this._inner.copy_subtree(encode(node),encode(String(identity))));}
    apply(change){this._inner.apply(encode(change));return this;}
    query(query){return JSON.parse(this._inner.query(encode(query)));}
    nodes(){return this.query('Nodes');}
    node(node){return this.query({Node:node});}
    ancestors(node){return this.query({Ancestors:node});}
    descendants(node){return this.query({Descendants:node});}
    leaves(node){return this.query({Leaves:node});}
    links(node){return this.query({Links:node});}
    path(start,end){return this.query({Path:{start,end}});}
    visit(root,order='BreadthFirst'){return this.query({Visit:{root,order}});}
    find(root,field,value){return this.query({Find:{root,field,value}});}
    find_registered(root,predicate){return this.query({FindRegistered:{root,predicate}});}
    sum(accessor){return this.apply({Sum:accessor});}
    count(){return this.apply('Count');}
    sort(comparator){return this.apply({Sort:comparator});}
    layout(configuration){return this.apply({Layout:configuration});}
    replace(envelope){return this.apply({Replace:envelope});}
    reset_history(){return this.apply('ResetHistory');}
    configuration(){return this.query('Configuration');}
    tile(request){return JSON.parse(this._inner.tile(encode(request)));}
  }
  const packing=(siblings,circles,limits)=>JSON.parse(native._Hierarchy.packing(encode({version:1,siblings,circles,...(limits===undefined?{}:{limits})})));
  const pack_siblings=(circles,limits)=>packing(true,circles,limits),pack_enclose=(circles,limits)=>packing(false,circles,limits);
  return {Hierarchy,pack_siblings,pack_enclose,packSiblings:pack_siblings,packEnclose:pack_enclose};
};
