import {Owned,ShapeRegistry} from './authoring.cjs';
export type HierarchyJSON = null|boolean|number|string|readonly HierarchyJSON[]|{readonly [key:string]:HierarchyJSON};
export interface HierarchyLimits {max_nodes?:number;max_depth?:number;max_work?:number;max_payload_bytes?:number;}
export interface HierarchyHandle {hierarchy:string;node:string;}
export interface HierarchyOperation {operation:{id:string;version:string};parameters?:HierarchyJSON;}
export type HierarchyScalar = {Constant:number}|{Field:string}|'Value'|'Depth'|{Registered:HierarchyOperation};
export type HierarchyComparator = {Scalar:{accessor:HierarchyScalar;descending:boolean}}|{Text:{field:string;descending:boolean}}|{Registered:HierarchyOperation};
export interface HierarchyPayload {key:string;data:HierarchyJSON;}
export interface HierarchyRow extends HierarchyPayload {parent:string|null;id?:string|null;synthetic?:boolean;}
export interface HierarchyNested extends HierarchyPayload {children:readonly HierarchyNested[];}
export interface HierarchyGroup {key:string;label:HierarchyJSON;value:HierarchyJSON;children:readonly HierarchyGroup[];}
export interface StratifyOptions {id_field?:string|null;parent_field?:string|null;path_field?:string|null;}
export type HierarchyInput = {Rows:readonly HierarchyRow[]}|{Nested:HierarchyNested}|{Node:HierarchyPayload}|{Grouped:{root:string;entries:readonly HierarchyGroup[]}}|{Stratify:{rows:readonly HierarchyPayload[];options:StratifyOptions}}|{Children:{root:HierarchyPayload;accessor:HierarchyOperation}};
export interface HierarchyEnvelope {version:1;identity:string;limits?:HierarchyLimits;input:HierarchyInput;}
export type HierarchyTiler = 'Binary'|'Dice'|'Slice'|'SliceDice'|'Custom'|{Squarify:number}|{Resquarify:number};
export interface HierarchyTreeOptions {mode?:{Extent:readonly [number,number]}|{NodeSize:readonly [number,number]};separation?:'Default'|'Depth'|{Constant:number};}
export interface HierarchyPartitionOptions {size?:readonly [number,number];round?:boolean;padding?:number;}
export type HierarchyPaddingSides = Partial<Record<'Inner'|'Top'|'Right'|'Bottom'|'Left',HierarchyScalar>>;
export interface HierarchyTreemapOptions {size?:readonly [number,number];round?:boolean;tile?:HierarchyTiler;padding_inner?:number;padding_top?:number;padding_right?:number;padding_bottom?:number;padding_left?:number;}
export interface HierarchyPackOptions {size?:readonly [number,number];radius?:'Fitted'|'Explicit';padding?:number;}
export type HierarchyLayout = {Tree:{options:HierarchyTreeOptions;separation?:HierarchyOperation|null}}|{Cluster:{options:HierarchyTreeOptions;separation?:HierarchyOperation|null}}|{Partition:HierarchyPartitionOptions}|{Treemap:{options:HierarchyTreemapOptions;history:boolean;padding?:HierarchyScalar|null;tiler?:HierarchyOperation|null;padding_sides?:HierarchyPaddingSides}}|{Pack:{options:HierarchyPackOptions;radius?:HierarchyScalar|null;padding?:HierarchyScalar|null}};
export interface HierarchyCircle {x:number;y:number;r:number;}
export type HierarchyGeometry = {Point:{x:number;y:number}}|{Rectangle:{x0:number;y0:number;x1:number;y1:number}}|{Circle:HierarchyCircle};
export interface HierarchyNodeRecord {handle:HierarchyHandle;data:HierarchyJSON;id:string|null;synthetic:boolean;parent:HierarchyHandle|null;children:HierarchyHandle[];depth:number;height:number;value:number|null;geometry:HierarchyGeometry|null;}
export type HierarchyVisitOrder = 'BreadthFirst'|'PreOrder'|'PostOrder';
export interface HierarchyTilingRequest {version:1;parent:HierarchyHandle;bounds:readonly [number,number,number,number];tiler:HierarchyTiler;history:boolean;operation?:HierarchyOperation|null;}
export interface HierarchyConfiguration {version:1;identity:string;limits:Required<HierarchyLimits>;layout:HierarchyLayout|null;effective_ratio:number|null;history_rows:number;history_members:number;}
export type HierarchyChange = 'Count'|'ResetHistory'|{Replace:HierarchyEnvelope}|{Sum:HierarchyScalar}|{Sort:HierarchyComparator}|{Layout:HierarchyLayout};
export class Hierarchy extends Owned {
  constructor(envelope:HierarchyEnvelope,registry?:ShapeRegistry);
  static from_json(value:string,registry?:ShapeRegistry):Hierarchy;
  static fromJson(value:string,registry?:ShapeRegistry):Hierarchy;
  to_json():string;
  toJson():string;
  copy():Hierarchy;
  copy_subtree(node:HierarchyHandle,identity:string|bigint):Hierarchy;
  copySubtree(node:HierarchyHandle,identity:string|bigint):Hierarchy;
  apply(change:HierarchyChange):this;
  query(query:HierarchyJSON):HierarchyJSON;
  nodes():HierarchyNodeRecord[];
  node(node:HierarchyHandle):HierarchyNodeRecord;
  ancestors(node:HierarchyHandle):HierarchyHandle[];
  descendants(node:HierarchyHandle):HierarchyHandle[];
  leaves(node:HierarchyHandle):HierarchyHandle[];
  links(node:HierarchyHandle):[HierarchyHandle,HierarchyHandle][];
  path(start:HierarchyHandle,end:HierarchyHandle):HierarchyHandle[];
  visit(root:HierarchyHandle,order?:HierarchyVisitOrder):[HierarchyNodeRecord,number,HierarchyHandle][];
  find(root:HierarchyHandle,field:string,value:HierarchyJSON):HierarchyHandle|null;
  find_registered(root:HierarchyHandle,predicate:HierarchyOperation):HierarchyHandle|null;
  findRegistered(root:HierarchyHandle,predicate:HierarchyOperation):HierarchyHandle|null;
  sum(accessor:HierarchyScalar):this;
  count():this;
  sort(comparator:HierarchyComparator):this;
  layout(configuration:HierarchyLayout):this;
  replace(envelope:HierarchyEnvelope):this;
  reset_history():this;
  resetHistory():this;
  configuration():HierarchyConfiguration;
  tile(request:HierarchyTilingRequest):[HierarchyHandle,[number,number,number,number]][];
}
export function pack_siblings(circles:readonly HierarchyCircle[],limits?:HierarchyLimits):HierarchyCircle[];
export function pack_enclose(circles:readonly HierarchyCircle[],limits?:HierarchyLimits):HierarchyCircle|null;
export const packSiblings:typeof pack_siblings;
export const packEnclose:typeof pack_enclose;
