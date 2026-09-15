import * as c from '../../../target/ggplot-theme-palette/wasm-module/authoring.cjs';
const palettes: Record<string,c.JSONValue> = {
  'palette.size.continuous': {operation:{id:'example.scale_palette',version:'1'},parameters:{channel:'size',mode:'first'}}
};
c.theme().scalePalettes(palettes).geometry({point_size:1.5});
c.theme().scale_palettes({});
c.theme().scalePalettes({'palette.colour.continuous': ['red', null, 'blue']});
c.theme().scalePalettes({'palette.colour.continuous': 'viridis'});
