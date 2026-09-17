//! Typed ownership shared by the native-language proof wrappers. No host grammar or source AST.
mod commands;
pub use commands::Editor;
mod options;
mod transaction;
use crate::{FigureRequest, FigureSnapshot, Format, Output, error};
use chart_core::{
    ChartResult, DiagnosticCode, Revision, SceneStamp,
    plot::{AxisHandle, LayerHandle, Plot, host::Component},
    portable::{self, InputOperation, InputQuery, Session},
    state::{ActionOrigin, ChartAction, StateEvent},
};
pub use options::Options;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::BTreeMap;
pub use transaction::Updates;

fn value<T: DeserializeOwned>(value: &Value) -> ChartResult<T> {
    serde_json::from_value(value.clone())
        .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
}
fn exact_u64(value: &Value) -> ChartResult<u64> {
    if let Some(s) = value.as_str() {
        return s.parse().map_err(|_| {
            error(
                DiagnosticCode::Validation,
                "Expected an exact unsigned 64-bit integer.",
            )
        });
    }
    value
        .as_u64()
        .filter(|v| *v <= 9_007_199_254_740_991)
        .ok_or_else(|| {
            error(
                DiagnosticCode::PrecisionLoss,
                "Use an exact integer adapter for large values.",
            )
        })
}
/// Supported byte encoders, without format fallback.
pub fn format(name: &str) -> ChartResult<Format> {
    match name {
        "svg" => Ok(Format::Svg),
        "pdf" => Ok(Format::Pdf),
        "png" => Ok(Format::Png),
        "jpeg" | "jpg" => Ok(Format::Jpeg),
        "tiff" | "tif" => Ok(Format::Tiff),
        "bmp" => Ok(Format::Bmp),
        "ps" | "postscript" => Ok(Format::PostScript),
        "eps" => Ok(Format::Eps),
        "tex" | "pictex" => Ok(Format::PicTeX),
        "emf" | "wmf" => Ok(Format::Emf),
        _ => Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Format must be svg, pdf, png, jpeg, tiff or bmp.",
        )),
    }
}
/// Owned typed runtime for primary Python/WASM authoring; output resources are independent.
pub struct Runtime {
    core: Option<Session>,
    layers: BTreeMap<String, LayerHandle>,
    axes: BTreeMap<String, AxisHandle>,
}
impl Runtime {
    /// Construct through Plot.chart without serializing or copying source columns.
    pub fn new(plot: &Plot) -> ChartResult<Self> {
        Ok(Self {
            core: Some(Session::from_runtime(plot.chart()?)?),
            layers: plot.named_layers().map(|(n, h)| (n.into(), h)).collect(),
            axes: plot.named_axes().map(|(n, h)| (n.into(), h)).collect(),
        })
    }
    /// Share immutable committed inputs with a new independent view and no copied writer.
    pub fn external_view(&self) -> ChartResult<Self> {
        Ok(Self {
            core: Some(Session::from_runtime(self.chart()?.external_view()?)?),
            layers: self.layers.clone(),
            axes: self.axes.clone(),
        })
    }
    /// Explicitly admit the source owner's latest commit into an external view.
    pub fn accept_from(&mut self, source: &Self) -> ChartResult<String> {
        portable::encode(&self.get_mut()?.runtime_mut().accept_from(source.chart()?)?)
    }
    fn get(&self) -> ChartResult<&Session> {
        self.core.as_ref().ok_or_else(disposed)
    }
    fn get_mut(&mut self) -> ChartResult<&mut Session> {
        self.core.as_mut().ok_or_else(disposed)
    }
    /// Read the actual core owner, preserving source and presented lifetimes.
    pub fn chart(&self) -> ChartResult<&chart_core::runtime::Chart> {
        Ok(self.get()?.runtime())
    }
    /// Resolve a named layer's exact handle for state commands and inspection.
    pub fn layer(&self, name: &str) -> ChartResult<LayerHandle> {
        self.get()?;
        self.layers.get(name).copied().ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                format!("No layer named '{name}'."),
            )
        })
    }
    /// Resolve a named axis for exact navigation and links.
    pub fn axis(&self, name: &str) -> ChartResult<AxisHandle> {
        self.get()?;
        self.axes.get(name).copied().ok_or_else(|| {
            error(
                DiagnosticCode::MissingResource,
                format!("No axis named '{name}'."),
            )
        })
    }
    /// Apply a definition-only edit under the explicit expected revision.
    pub fn apply_plot(&mut self, plot: &Plot, expected: u64) -> ChartResult<bool> {
        let changed = self
            .get_mut()?
            .runtime_mut()
            .apply_plot(plot, Revision::new(expected))?;
        if changed {
            self.layers = plot.named_layers().map(|(n, h)| (n.into(), h)).collect();
            self.axes = plot.named_axes().map(|(n, h)| (n.into(), h)).collect();
        }
        Ok(changed)
    }
    /// Exact state/definition/source revisions for typed host receipts and fences.
    pub fn revisions(&self) -> ChartResult<String> {
        let c = self.chart()?;
        portable::encode(
            &json!({"definition":c.definition().revision,"state":c.state().revision(),"epoch":c.source().get()?.epoch(),"store":c.source().get()?.revision()}),
        )
    }
    /// Explicitly request owned full semantic data; ordinary queries use retained indexes instead.
    pub fn semantics(&mut self) -> ChartResult<String> {
        self.get_mut()?.semantics_json()
    }
    /// Capture the portable state snapshot for controlled replies.
    pub fn state(&self) -> ChartResult<String> {
        self.get()?.state_json()
    }
    /// Restore a captured state under its explicit expected-state fence.
    pub fn restore_state(&mut self, input: &str, expected: u64) -> ChartResult<()> {
        self.get_mut()?
            .restore_state(input, Revision::new(expected))
    }
    /// Dispatch a typed action; defaults and scene/revision fences come from the Rust owner.
    pub fn act(&mut self, input: &str, origin: &str, expected: Option<u64>) -> ChartResult<String> {
        let action: ChartAction = portable::decode(input)?;
        let origin: ActionOrigin = portable::decode(origin)?;
        let c = self.get_mut()?.runtime_mut();
        let mut request = c.request(action, origin);
        if let Some(expected) = expected {
            request.expected_state = Revision::new(expected);
        }
        portable::encode(&c.dispatch(request)?)
    }
    /// Run an explicit pure query on the current presented or pinned scene; optional stamp detects stale callers.
    pub fn query(
        &mut self,
        input: &str,
        gesture: bool,
        stamp: Option<SceneStamp>,
    ) -> ChartResult<String> {
        let query: InputOperation = portable::decode(input)?;
        let c = self.get_mut()?;
        let scene = if gesture {
            c.reducer().gesture_basis()
        } else {
            c.reducer().presented()
        }
        .ok_or_else(|| {
            error(
                DiagnosticCode::UnsupportedCapability,
                "Query requires the requested acknowledged scene.",
            )
        })?
        .scene()
        .stamp();
        c.query_input(InputQuery {
            scene: stamp.unwrap_or(scene),
            gesture,
            query,
        })
    }
    /// Acquire cheap coherent publication inputs; the request can outlive this runtime.
    pub fn request(&self, output: &Output, options: &Options) -> ChartResult<FigureRequest> {
        output.live_request(self.chart()?, options.0.clone())
    }
    /// Prepare and acknowledge the headless scene selected by this host for subsequent input.
    pub fn present(&mut self, output: &Output, options: &Options) -> ChartResult<FigureSnapshot> {
        let basis = if self.chart()?.reducer().frozen_scene().is_some() {
            crate::CaptureBasis::Presented
        } else {
            crate::CaptureBasis::Current
        };
        let figure = output
            .live_request(
                self.chart()?,
                options
                    .0
                    .clone()
                    .basis(basis)
                    .interaction(chart_core::state::InteractionCapture::ALL),
            )?
            .prepare()?;
        let c = self.get_mut()?.runtime_mut();
        c.acknowledge_paint_with_layout(
            figure.layout().clone(),
            &figure.metadata().effective_state,
            &figure.metadata().profile.layout,
        )?;
        Ok(figure)
    }
    /// Acknowledge an explicitly rendered immutable frame; existing runtime fences reject stale inputs.
    pub fn acknowledge_frame(&mut self, figure: &FigureSnapshot) -> ChartResult<()> {
        let current = self.chart()?;
        let prepared = figure.layout().prepared();
        let source = prepared.source().get()?;
        let latest = current.source();
        let latest = latest.get()?;
        if prepared.definition() != current.definition()
            || source.epoch() != latest.epoch()
            || source.revision() != latest.revision()
            || prepared.state().viewport_revision() != current.state().viewport_revision()
            || current
                .definition()
                .layers
                .iter()
                .any(|l| prepared.state().is_visible(l.id) != current.state().is_visible(l.id))
            || prepared.state().annotations(current.definition())
                != current.state().annotations(current.definition())
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Sampled frame no longer matches the current definition, source or geometry state.",
            ));
        }
        self.get_mut()?.runtime_mut().acknowledge_paint_with_layout(
            figure.layout().clone(),
            &figure.metadata().effective_state,
            &figure.metadata().profile.layout,
        )
    }
    /// Configure the existing owned ingestion queue without committing it.
    pub fn stream(&mut self, options: &Component) -> ChartResult<()> {
        self.get_mut()?
            .runtime_mut()
            .stream(options.stream_options()?)
    }
    /// Queue occupancy and independent commit accounting.
    pub fn queue_status(&self) -> ChartResult<String> {
        portable::encode(self.chart()?.queue_status())
    }
    /// Current queue limits, accounting, source revisions and last reconciliation.
    pub fn stream_status(&self) -> ChartResult<String> {
        let c = self.chart()?;
        let source = c.source();
        let source = source.get()?;
        portable::encode(
            &json!({"limits":c.queue_limits(),"queue":c.queue_status(),"epoch":source.epoch(),"store_revision":source.revision(),"reconciliation":c.reconciliation()}),
        )
    }
    /// Describe the pinned value, retaining its historical-source label after updates.
    pub fn pinned(&self) -> ChartResult<String> {
        portable::encode(&self.chart()?.reducer().describe_pinned()?)
    }
    /// Drain one transaction with its existing receipt/conflict outcome.
    pub fn commit_next(&mut self) -> ChartResult<String> {
        let result = self.get_mut()?.runtime_mut().commit_next()?;
        let reconciliation = if matches!(
            &result,
            Some((_, chart_core::transaction::CommitOutcome::Applied(_)))
        ) {
            self.chart()?.reconciliation()
        } else {
            None
        };
        portable::encode(&result.map(|(id,outcome)|json!({"id":id.as_str(),"outcome":outcome,"reconciliation":reconciliation})))
    }
    /// Advance the owned epoch while keeping queued stale transactions explicitly observable.
    pub fn reset_epoch(&mut self) -> ChartResult<String> {
        portable::encode(&self.get_mut()?.runtime_mut().reset_epoch()?)
    }
    /// Select exact density policy over a captured scene, preserving source/provenance.
    pub fn dense(&self, figure: &FigureSnapshot, options: &Component) -> ChartResult<String> {
        let density = options.render_options()?.build(self.chart()?)?;
        let dense = chart_core::dense::DenseChart::prepare(
            figure.layout().clone(),
            &density,
            figure.metadata().profile.layout.limits,
        )?;
        portable::encode(
            &json!({"stamp":dense.scene().stamp(),"metrics":dense.metrics(),"candles":dense.candles(),"items":dense.scene().items()}),
        )
    }
    /// Capture a link event with core echo suppression and exact identity matching.
    pub fn link_capture(&mut self, component: &Component, event: &str) -> ChartResult<String> {
        let event: StateEvent = portable::decode(event)?;
        portable::encode(
            &component
                .link_builder()?
                .capture(self.get_mut()?.runtime_mut(), &event)?,
        )
    }
    /// Resolve linked targets/windows, retaining unmatched identities instead of positional substitution.
    pub fn link_resolve(&mut self, component: &Component, message: &str) -> ChartResult<String> {
        let message = portable::decode(message)?;
        let update = component
            .link_builder()?
            .resolve(self.get_mut()?.runtime_mut(), &message)?;
        portable::encode(
            &json!({"action":update.action,"origin":update.origin,"unmatched":update.unmatched}),
        )
    }
    /// Idempotently release this runtime and its indexes/store/queue; captured requests stay valid.
    pub fn dispose(&mut self) {
        self.core.take();
        self.layers.clear();
        self.axes.clear();
    }
}
fn disposed() -> chart_core::Diagnostic {
    error(
        DiagnosticCode::DisposedHandle,
        "This primary host handle has been disposed.",
    )
}
