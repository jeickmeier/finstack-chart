import {ShapeLineRadial,ShapeAreaRadial,ShapeLink,ShapeLinkRadial,pointRadial,point_radial,ShapeLineRadialConfig,ShapeAreaRadialConfig,ShapeLinkConfig,ShapeLinkRadialConfig,LinkDatum,Path} from '../../../packages/wasm/authoring.cjs';
const lineConfig: ShapeLineRadialConfig={angle:{Column:2},radius:{Constant:4},defined:[true,false],curve:{kind:'Cardinal',tension:.2},digits:null};
const areaConfig: ShapeAreaRadialConfig={start_angle:{Column:0},end_angle:null,inner_radius:{Constant:2},outer_radius:{Column:1}};
const linkConfig: ShapeLinkConfig={source:'Target',target:{Constant:[1,2]},curve:{kind:'BumpY'}};
const radialConfig: ShapeLinkRadialConfig={angle:{Column:1},radius:{Column:0},digits:12};
const datum: LinkDatum={source:[0,10],target:[1,30]};
const pair: [number,number]=pointRadial(1,20);point_radial(...pair);
const line=new ShapeLineRadial(lineConfig),area=new ShapeAreaRadial(areaConfig);
const paths: Path[]=[line.generate([[0,1,2],[2,3,4]]),area.boundary('OuterRadius').generate([[0,1],[2,3]]),new ShapeLink(linkConfig).generate(datum),new ShapeLinkRadial(radialConfig).generate(datum)];
line.copy().config();paths.forEach(p=>p.free());
// @ts-expect-error radial selectors use angle/radius
new ShapeLineRadial({x:{Column:0}});
// @ts-expect-error named radial boundary
area.boundary('X1');
// @ts-expect-error radial links have a fixed radial tangent protocol
new ShapeLinkRadial({curve:{kind:'BumpX'}});
// @ts-expect-error links consume a source/target datum
new ShapeLink().generate([[1,2],[3,4]]);
// @ts-expect-error endpoints are source/target or constant numeric rows
new ShapeLink({source:'Node'});
// @ts-expect-error angle must be numeric
pointRadial('zero',10);
import {shapeLineRadial,shapeAreaRadial,shapeLink,shapeLinkHorizontal,shapeLinkVertical,shapeLinkRadial,RadialParameters,Layer} from '../../../packages/wasm/authoring.cjs';
const parameters:RadialParameters={start_angle:0,end_angle:1,inner_radius:4,outer_radius:20};
const layers:Layer[]=[shapeLineRadial().radialParameters(parameters).shapeValue('Angle',1).shapeValue('Radius',20),shapeAreaRadial().curve({kind:'Basis'}),shapeLink({kind:'BumpX'}),shapeLinkHorizontal(),shapeLinkVertical(),shapeLinkRadial().radialParameters(parameters)];
// @ts-expect-error chart radial constants use named radii
shapeLineRadial().radialParameters({radius:20});
// @ts-expect-error curve selection is typed
shapeLink({kind:'Unknown'});
