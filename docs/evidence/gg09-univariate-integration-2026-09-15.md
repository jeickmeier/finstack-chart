# GG09 univariate integration checkpoint

On `96d044d` plus the owned worktree, `mise exec -- cargo test -p chart-core --test ggplot_univariate --locked` passes eleven focused tests (`/private/tmp/gg09-univariate-final-focused.log`). This is integration evidence, not complete GG09 acceptance.

The compiler tests cover all 24 pinned QQ/QQ-line cases (normal, uniform and logistic distributions), all twelve ECDF cases including signed, missing and zero-total weights, two inverse-log function cases, matrix connections and both grouped alignment cases. Numeric comparisons use 1e-10 absolute or 1e-12 relative tolerance; exact membership counts and source/derived distinctions are checked independently of destination ink. Named connections select the existing step-curve owner. Reference areas default to alignment and stacking; explicit statistics retain area stacking unless the layer is a density recipe.

Additional regressions prove:

- Function sampling without a mapped x input uses the unit interval or authored axis limits. Mapped functions evaluate inverse-transformed coordinates before output projection.
- Registered pure functions and QQ distributions require exact installed identities; native-only registrations cannot serialize; malformed result cardinality rejects.
- Population-dependent output transforms receive one complete generated vector, rather than a scalar approximation.
- Distribution/univariate stages retain only source aesthetics constant within each group; varying aesthetics are dropped while retained constants remain usable. Count partitions keep their separate strict contract.
- Unique retains first-row source identities for distinct mapped tuples. Six univariate families match fresh batch values and memberships after append, upsert and removal; previously prepared snapshots remain unchanged.

The shared generated-vector stage now includes dependent recipe columns and outlier values. Recipe inputs are resolved during encoding before this stage; position-dependent setup remains later. Ordinary scale projection applies out-of-bounds policy once to the transformed outputs.

Independent Rust, Python and WASM author files cover eight charts (24 SVG/PDF/PNG publications per runtime): weighted KDE, weighted ECDF, QQ, QQ-line, plain/log-x functions, unique and matrix connections. Native examples use the same core engine and supplied font. These authors and final visuals must be accepted with the coordinated GG09/GG12 runtime proof; their existence alone does not establish host or publication acceptance.

Remaining package acceptance: final density outline/default paint and facet delta integration, actual runtime publication equality, visual inspection, repository checks and a coherent workspace suite. GG10 kernels are separate unwired work and are not certified by this checkpoint.
