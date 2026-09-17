//! Bounded output sinks and the shared retained-vector tree walker for the headless devices.
use crate::{VectorAlphaPolicy, error};
use chart_core::{ChartResult, DiagnosticCode};
use std::fmt::Write as _;

pub(crate) struct BoundedString {
    pub(crate) text: String,
    pub(crate) limit: usize,
}
impl BoundedString {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            text: String::new(),
            limit,
        }
    }
    pub(crate) fn remaining(&self) -> usize {
        self.limit - self.text.len()
    }
    /// Append or fail with ResourceLimit and the device's budget message.
    pub(crate) fn add(
        &mut self,
        args: std::fmt::Arguments<'_>,
        exceeded: &'static str,
    ) -> ChartResult<()> {
        self.write_fmt(args)
            .map_err(|_| error(DiagnosticCode::ResourceLimit, exceeded))
    }
}
impl std::fmt::Write for BoundedString {
    // Each invocation is bounded independently of the final output allocation.
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if self
            .text
            .len()
            .checked_add(s.len())
            .is_none_or(|n| n > self.limit)
        {
            return Err(std::fmt::Error);
        }
        self.text.push_str(s);
        Ok(())
    }
}
pub(crate) struct BoundedBytes {
    pub(crate) bytes: Vec<u8>,
    pub(crate) limit: usize,
}
impl BoundedBytes {
    pub(crate) fn new(limit: usize) -> Self {
        Self {
            bytes: vec![],
            limit,
        }
    }
    /// True when `additional` more bytes still fit the budget (checked arithmetic).
    pub(crate) fn fits(&self, additional: usize) -> bool {
        self.bytes
            .len()
            .checked_add(additional)
            .is_some_and(|n| n <= self.limit)
    }
}

/// Shared partial-transparency policy for the retained-vector devices.
pub(crate) struct AlphaPolicy {
    pub(crate) policy: VectorAlphaPolicy,
    pub(crate) omitted: usize,
    device: &'static str,
}
impl AlphaPolicy {
    pub(crate) fn new(policy: VectorAlphaPolicy, device: &'static str) -> Self {
        Self {
            policy,
            omitted: 0,
            device,
        }
    }
    /// Ok(true) paints; Ok(false) skips (fully transparent, or translucent under
    /// OmitTranslucent — counted); Err under Reject.
    pub(crate) fn admit(&mut self, alpha: f32) -> ChartResult<bool> {
        if alpha == 0. {
            return Ok(false);
        }
        if alpha == 1. {
            return Ok(true);
        }
        match self.policy {
            VectorAlphaPolicy::OmitTranslucent => {
                self.omitted += 1;
                Ok(false)
            }
            VectorAlphaPolicy::Reject => Err(error(
                DiagnosticCode::ExportFidelity,
                format!(
                    "{} does not support partial transparency; select an explicit vector alpha policy.",
                    self.device
                ),
            )),
        }
    }
}
pub(crate) enum LeafPaint<'a> {
    Fill { even_odd: bool },
    Stroke(&'a usvg::Stroke),
}
pub(crate) struct Leaf<'a> {
    pub(crate) path: &'a usvg::tiny_skia_path::Path,
    pub(crate) transform: usvg::Transform,
    pub(crate) paint: &'a usvg::Paint,
    pub(crate) kind: LeafPaint<'a>,
}
/// Walk a usvg subtree in paint order, applying the alpha policy to groups and paints,
/// rejecting masks/filters/images, and flattening text.
pub(crate) fn walk(
    node: &usvg::Node,
    alpha: &mut AlphaPolicy,
    emit: &mut dyn FnMut(Leaf<'_>, &mut AlphaPolicy) -> ChartResult<()>,
) -> ChartResult<()> {
    match node {
        usvg::Node::Group(g) => group(g, alpha, emit),
        usvg::Node::Text(t) => group(t.flattened(), alpha, emit),
        usvg::Node::Image(_) => Err(error(
            DiagnosticCode::ExportFidelity,
            format!(
                "{} image must use its retained raster scene item.",
                alpha.device
            ),
        )),
        usvg::Node::Path(p) => {
            if !p.is_visible() {
                return Ok(());
            }
            let order = if p.paint_order() == usvg::PaintOrder::FillAndStroke {
                [false, true]
            } else {
                [true, false]
            };
            for stroke in order {
                let (paint, opacity, kind) = if stroke {
                    let Some(s) = p.stroke() else { continue };
                    (s.paint(), s.opacity().get(), LeafPaint::Stroke(s))
                } else {
                    let Some(f) = p.fill() else { continue };
                    (
                        f.paint(),
                        f.opacity().get(),
                        LeafPaint::Fill {
                            even_odd: f.rule() == usvg::FillRule::EvenOdd,
                        },
                    )
                };
                if !alpha.admit(opacity)? {
                    continue;
                }
                emit(
                    Leaf {
                        path: p.data(),
                        transform: p.abs_transform(),
                        paint,
                        kind,
                    },
                    alpha,
                )?;
            }
            Ok(())
        }
    }
}
/// Same walk rooted at a group; the tree root is a `usvg::Group`, not a `Node`.
#[cfg(test)]
pub(crate) fn walk_group(
    g: &usvg::Group,
    alpha: &mut AlphaPolicy,
    emit: &mut dyn FnMut(Leaf<'_>, &mut AlphaPolicy) -> ChartResult<()>,
) -> ChartResult<()> {
    group(g, alpha, emit)
}
fn group(
    g: &usvg::Group,
    alpha: &mut AlphaPolicy,
    emit: &mut dyn FnMut(Leaf<'_>, &mut AlphaPolicy) -> ChartResult<()>,
) -> ChartResult<()> {
    if !alpha.admit(g.opacity().get())? {
        return Ok(());
    }
    // Scene item clips are resolved independently before traversing this tree.
    if g.mask().is_some() || !g.filters().is_empty() {
        return Err(error(
            DiagnosticCode::ExportFidelity,
            format!("{} cannot silently flatten masks or filters.", alpha.device),
        ));
    }
    for node in g.children() {
        walk(node, alpha, emit)?;
    }
    Ok(())
}
/// Cubic control points equivalent to a quadratic segment from `current` through `control` to `end`.
pub(crate) fn quad_to_cubic(
    current: (f32, f32),
    control: (f32, f32),
    end: (f32, f32),
) -> [(f32, f32); 2] {
    [
        (
            current.0 + 2. / 3. * (control.0 - current.0),
            current.1 + 2. / 3. * (control.1 - current.1),
        ),
        (
            end.0 + 2. / 3. * (control.0 - end.0),
            end.1 + 2. / 3. * (control.1 - end.1),
        ),
    ]
}
