import type {Plot, PlotBuilder, Stat, Layer, MappingValue} from './authoring.cjs';
export function withExtensions(builder: PlotBuilder): PlotBuilder;
export function densityHistogram(field: MappingValue, edges: ReadonlyArray<number>): Stat;
export function chamferedBars(native?: boolean): Layer;

export function loadPlot(value: string): Plot;
