//! GG10 independently authored weighted model and quantile publications.
use chart_core::{grammar::*, prelude::*};
pub fn author(mode: usize) -> chart_core::ChartResult<Plot> {
    let data = Data::columns()
        .column("x", (0..24).map(|i| i as f64).collect::<Vec<_>>())
        .column(
            "y",
            (0..24)
                .map(|i| 1. + (i % 5) as f64 + 0.2 * i as f64)
                .collect::<Vec<_>>(),
        )
        .column("w", (0..24).map(|i| (1 + i % 3) as f64).collect::<Vec<_>>())
        .build()?;
    let method = match mode {
        0 | 7 | 8 | 9 => ModelMethod::Linear,
        1 => ModelMethod::Glm {
            family: ModelFamily::Poisson,
            epsilon: 1e-8,
            iterations: 25,
        },
        2 => ModelMethod::Loess {
            span: 0.75,
            degree: 2,
            cell: 0.2,
            surface: LoessSurface::Interpolate,
            family: LoessFamily::Gaussian,
            iterations: 4,
            normalize: true,
            exact_statistics: false,
            approximate_trace: false,
        },
        3 => ModelMethod::Gam {
            basis_dimension: 10,
            knots: None,
            iterations: 120,
            tolerance: 1e-9,
        },
        4 | 5 => ModelMethod::Quantile {
            solver: if mode == 4 {
                ModelQuantileSolver::Br
            } else {
                ModelQuantileSolver::Fn
            },
            probabilities: vec![0.25, 0.5, 0.75],
            iterations: 10000,
        },
        _ => ModelMethod::Auto,
    };
    let options = ModelOptions {
        method,
        n: 41,
        level: if mode == 7 { 0.5 } else { 0.95 },
        full_range: mode == 9,
        se: mode != 4 && mode != 5,
        terms: if mode == 0 {
            Some(vec![
                ModelTerm::Power(0),
                ModelTerm::Power(1),
                ModelTerm::Power(2),
            ])
        } else {
            None
        },
        ..Default::default()
    };
    let layer = smooth().stat(model_stat(options).x("x").y("y").weight("w"));
    let layer = if mode == 8 {
        layer.orientation(Orientation::Horizontal)
    } else {
        layer
    };
    plot(data)
        .profile(Profile::Ggplot2_4_0_3)
        .layer(layer)
        .x_axis(x_axis().scale(scale_linear().domain(
            if mode == 8 { -2. } else { -1. },
            if mode == 8 { 12. } else { 24. },
        )))
        .y_axis(y_axis().scale(scale_linear().domain(
            if mode == 8 { -1. } else { -2. },
            if mode == 8 { 24. } else { 12. },
        )))
        .build()
}
