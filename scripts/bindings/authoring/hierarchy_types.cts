import * as c from '../../../packages/wasm/authoring.cjs';
function hierarchy(data:c.Data,output:c.Output,previous:c.FigureSnapshot):c.FigureSnapshot {
  const layout:c.HierarchyLayout={Treemap:{options:{tile:{Resquarify:1.6}},history:true,padding_sides:{Top:'Depth'}}};
  const recipe:c.HierarchyRecipe={identity:'991',source:{Paths:'path'},aggregation:{Sum:'v'},label:null,layout};
  const layer=c.hierarchy(recipe).hierarchyValue(data.field('v')).hierarchyLabel(data.field('path')).hierarchyProjection('Cartesian').hierarchyOrder('ValueDescending').hierarchyLimits({max_nodes:1000});
  // @ts-expect-error hierarchy owner is an exact decimal string
  c.hierarchy({...recipe,identity:991});
  // @ts-expect-error ordinary scales cannot substitute a hierarchy layout
  layer.hierarchyLayout(c.scaleLinear());
  // @ts-expect-error explicit registered operations replace executable host callbacks
  layer.hierarchyOrder((a:unknown,b:unknown)=>0);
  return output.request(c.plot(data).layer(layer).build(),c.exportOptions(500,300)).withHierarchyHistory(previous).prepare();
}
function operations(envelope:c.HierarchyEnvelope):c.HierarchyNodeRecord {
  const tree=new c.Hierarchy(envelope).sum({Field:'value'}).layout({Tree:{options:{mode:{NodeSize:[20,40]}}}});
  const root=tree.nodes()[0].handle,copy=tree.copySubtree(root,12345n),restored=c.Hierarchy.fromJson(copy.toJson());
  // @ts-expect-error unscoped node IDs cannot address a hierarchy occurrence
  tree.node('1');
  return restored.node(restored.nodes()[0].handle);
}
