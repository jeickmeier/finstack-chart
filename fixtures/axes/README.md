# FIX-19 axis reference corpus

`reference.json` is actual d3-axis 3.0.0 SVG DOM output from the pinned Chromium
151.0.7922.34 browser. The 372 static cases cover all four sides, numeric and
categorical scale families, UTC intervals, per-guide selection/format settings,
resets, signed ticks, padding, offset and device scales 1/2. Shared-guide sequences
retain DOM identities through scale-domain changes and replacement of one guide's
scale. `inventory.json` records all four exports, ten methods and observed defaults.

`manifest.json` pins browser/source/lock/generator identities. This is reference
input for WP-AX01–06; it does not establish Rust, native, Python or WASM parity.
Timed transition sampling, explicit local-zone DST, registered provider behavior,
component styling and final native/publication evidence remain open.

Regenerate using the installed tools, without browser downloads:

```sh
PLAYWRIGHT_MODULE=/absolute/path/to/playwright/index.mjs \
CHROMIUM_EXECUTABLE=/absolute/path/to/chrome-headless-shell \
  mise exec -- node tools/reference/node/axis.mjs
```

The generator checks the Chromium version and uses the shared reference lockfile.
D3 selection/transition support packages are development-only. Earlier path/color/
scale/shape oracle manifests retain their original lock identity; the exact prior
lock is archived at `tools/reference/node/locks/foundation-package-lock.json`.
Its SHA-256 is `bcaf52b90527128695304aabf5a959f0b19445715d4d6ca465a0ea6de8b3187e`.
No older reference expected values were regenerated when the DOM dependencies were
added. Production crates retain no JavaScript or browser dependency.
