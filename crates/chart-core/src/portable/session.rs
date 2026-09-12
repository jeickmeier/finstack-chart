use super::error;
use super::*;
use crate::{grammar::*, runtime::Chart, state::*, transaction::*, *};
use serde_json::json;
use std::sync::Arc;

/// Versioned JSON compatibility adapter over the shared typed [`Chart`] runtime.
/// Construction preserves the legacy eager preparation and portable capability checks.
pub struct Session {
    chart: Chart,
}
impl std::ops::Deref for Session {
    type Target = Chart;
    fn deref(&self) -> &Chart {
        &self.chart
    }
}
impl std::ops::DerefMut for Session {
    fn deref_mut(&mut self) -> &mut Chart {
        &mut self.chart
    }
}
impl Session {
    /// Adopt the primary typed runtime without encoding/decoding or rebuilding its source store.
    pub fn from_runtime(chart: Chart) -> ChartResult<Self> {
        chart.extensions().validate_portable(chart.definition())?;
        Ok(Self { chart })
    }
    /// Decode and validate versioned portable chart/data inputs.
    pub fn new(chart: &str, data: &str) -> ChartResult<Self> {
        Self::with_extensions(chart, data, Arc::new(ExtensionRegistry::new()))
    }
    /// Execute only explicitly registered portable extensions.
    pub fn with_extensions(
        chart: &str,
        data: &str,
        extensions: Arc<ExtensionRegistry>,
    ) -> ChartResult<Self> {
        let definition: ChartEnvelope = decode(chart)?;
        extensions.validate_portable(&definition.definition)?;
        definition.validate()?;
        let data: DataEnvelope = decode(data)?;
        let mut chart = Chart::from_store(definition.definition, data.into_store()?, extensions)?;
        chart.prepare()?;
        Ok(Self { chart })
    }
    /// Access typed execution without serializing operations.
    pub fn runtime(&self) -> &Chart {
        &self.chart
    }
    /// Access typed execution without serializing operations.
    pub fn runtime_mut(&mut self) -> &mut Chart {
        &mut self.chart
    }
    /// Serialize the definition, applying the portable capability contract.
    pub fn chart_json(&self) -> ChartResult<String> {
        self.extensions().validate_portable(self.definition())?;
        encode(&ChartEnvelope {
            version: self.definition().wire_version(),
            definition: self.definition().clone(),
        })
    }
    /// Serialize exact state/revisions in their separate envelope.
    pub fn state_json(&self) -> ChartResult<String> {
        encode(&StateEnvelope::capture(self.definition(), self.state()))
    }
    /// Decode controlled state, then use the typed revision-fenced restore.
    pub fn restore_state(&mut self, input: &str, expected: Revision) -> ChartResult<()> {
        if expected != self.state().revision() {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Stale state restore",
            ));
        }
        let state: StateEnvelope = decode(input)?;
        let next = state.into_state(self.definition())?;
        self.chart.restore_state(next, expected)
    }
    /// Decode an atomic transaction, preserving typed receipts and replay outcomes.
    pub fn apply_transaction(&mut self, input: &str) -> ChartResult<CommitOutcome> {
        let envelope: TransactionEnvelope = decode(input)?;
        self.chart.apply_transaction(envelope.into_transaction()?)
    }
    /// Decode bounded queue operations and encode the existing version 1 results.
    pub fn stream(&mut self, input: &str) -> ChartResult<String> {
        let envelope: StreamEnvelope = decode(input)?;
        version(envelope.version)?;
        match envelope.operation {
            StreamOperation::ConfigureQueue(limits) => {
                self.chart.configure_queue(limits)?;
                encode(&self.queue_limits())
            }
            StreamOperation::Enqueue(transaction) => {
                encode(&self.chart.enqueue(transaction.into_transaction()?)?)
            }
            StreamOperation::CommitNext => {
                let result = self.chart.commit_next()?;
                let reconciliation = if matches!(&result, Some((_, CommitOutcome::Applied(_)))) {
                    self.reconciliation().cloned()
                } else {
                    None
                };
                encode(&result.map(|(id, outcome)| json!({"id":id.as_str(),"outcome":outcome,"reconciliation":reconciliation})))
            }
            StreamOperation::Status => encode(
                &json!({"limits":self.queue_limits(),"queue":self.queue_status(),"epoch":self.source().get()?.epoch(),"store_revision":self.source().get()?.revision(),"reconciliation":self.reconciliation()}),
            ),
            StreamOperation::Pinned => encode(&self.reducer().describe_pinned()?),
        }
    }
    /// Decode exact definition/state fences, then dispatch through the typed reducer.
    pub fn apply_action(&mut self, input: &str) -> ChartResult<ActionOutcome> {
        let envelope: ActionEnvelope = decode(input)?;
        version(envelope.version)?;
        if envelope.definition_revision != self.definition().revision
            || envelope.expected_state != self.state().revision()
        {
            return Err(error(
                DiagnosticCode::RevisionConflict,
                "Action definition/state revision is stale",
            ));
        }
        Ok(self.chart.act(envelope.action)?.outcome)
    }
    /// Decode a full typed action request with explicit origin/state/scene fences.
    pub fn dispatch(&mut self, input: &str) -> ChartResult<DispatchOutcome> {
        self.chart.dispatch(decode(input)?)
    }
    /// Decode pure presented-scene queries; the runtime owns and reuses their geometry indexes.
    pub fn query(&mut self, input: &str) -> ChartResult<String> {
        let request: InputQuery = decode(input)?;
        self.query_input(request)
    }
    /// Run a typed query through the same retained presented/gesture inspection path.
    pub fn query_input(&mut self, request: InputQuery) -> ChartResult<String> {
        use crate::navigation::Navigator;
        let inspector = self.chart.inspector(request.scene, request.gesture)?;
        let scene = inspector.presented().clone();
        match request.query {
            InputOperation::Describe { offset, limit } => {
                encode(&inspector.accessible_page(self.state(), offset, limit)?)
            }
            InputOperation::EditAnnotation {
                id,
                part,
                constraints,
                delta,
            } => {
                let editor = crate::editing::AnnotationEditor::new(scene, &id, part, constraints)?;
                encode(&json!({"annotation": editor.preview(request.scene, delta[0], delta[1])?}))
            }
            InputOperation::LinkCapture {
                origin,
                event,
                axes,
                panel,
                selection,
            } => encode(
                &json!({"message":crate::linking::LinkMessage::from_event(&origin,&event,&inspector,self.state(),&axes,panel.as_ref(),selection)?}),
            ),
            InputOperation::LinkResolve {
                message,
                mappings,
                panel,
                missing,
            } => {
                let update = message.resolve(&inspector, &mappings, panel.as_ref(), missing)?;
                encode(
                    &json!({"action":update.action,"origin":update.origin,"unmatched":update.unmatched}),
                )
            }
            InputOperation::Inspect {
                point,
                radius,
                max_grouped,
                mode,
            } => {
                let result = inspector
                    .with_options(radius, max_grouped)?
                    .query(super::input::point(point)?, mode);
                let epoch = scene.prepared().source().get()?.epoch();
                let targets: Vec<_> = result
                    .hits
                    .iter()
                    .map(|h| MarkTarget::from_inspected(h, epoch))
                    .collect();
                encode(
                    &json!({"hits":result.hits,"targets":targets,"examined":result.examined,"nodes":result.nodes}),
                )
            }
            InputOperation::Select { region, limit } => {
                encode(&json!({"targets":inspector.select(request.scene,&region.region()?,limit)?}))
            }
            InputOperation::Navigate {
                axes,
                panel,
                action,
                boundary,
            } => encode(
                &json!({"windows":Navigator::new(scene).navigate(request.scene,&axes,panel.as_ref(),action.navigation()?,boundary)?}),
            ),
            InputOperation::SetRange {
                axis,
                panel,
                window,
            } => encode(
                &json!({"windows":Navigator::new(scene).set_range(request.scene,axis,panel.as_ref(),window)?}),
            ),
        }
    }
    /// Semantic result DTO for runtime comparison: domains, generated rows, targets, exact sources.
    pub fn semantics_json(&mut self) -> ChartResult<String> {
        let prepared = self.prepare()?;
        let source = self.source();
        let data = source.get()?;
        let datasets=data.datasets().map(|d| json!({
            "version":d.version(),"schema":d.schema(),"retention":d.retention(),
            "chunks":d.chunks().iter().map(|c|BatchWire::from_batch(c.batch())).collect::<Vec<_>>(),
        })).collect::<Vec<_>>();
        let layer_semantics =
            |l: &crate::grammar::PreparedLayer| -> ChartResult<serde_json::Value> {
                let mut value = json!({
                    "id":l.id(),"visible":l.visible(),"domains":l.domains(),"rows":l.table().rows(),
                    "schema":l.table().schema(),"operations":l.table().operations(),
                    "color_legend":l.color_legend(),"invalid_geometry":l.invalid_geometry(),
                    "targets":l.marks().iter().flat_map(|m|m.targets.iter()).collect::<Vec<_>>(),
                });
                if !l.paint_legends().is_empty()
                    || l.marks().iter().any(|m| {
                        !m.aesthetics.is_empty()
                            || m.style.fill.is_some()
                            || m.style.stroke.is_some()
                            || m.style.alpha.is_some()
                            || m.style.line_type.is_some()
                            || m.style.units.is_some()
                    })
                {
                    value["paint_legends"] = json!(l.paint_legends());
                    value["styles"] = json!(l.marks().iter().map(|m| &m.style).collect::<Vec<_>>());
                    value["aesthetics"] =
                        json!(l.marks().iter().map(|m| &m.aesthetics).collect::<Vec<_>>());
                    value["numeric_scales"] = json!(l.numeric_scales());
                    value["value_scales"] = json!(l.value_scales());
                }
                if let Some(h) = l.hierarchy() {
                    let nodes = h
                        .hierarchy()
                        .iter()?
                        .map(|n| crate::hierarchy::NodeRecord::from_node(n, None))
                        .collect::<ChartResult<Vec<_>>>()?;
                    value["hierarchy"] = json!({"version":1,"recipe":h.recipe(),"source_keys":h.source_keys(),"nodes":nodes});
                }
                Ok(value)
            };
        let layers = prepared
            .layers()
            .iter()
            .map(&layer_semantics)
            .collect::<ChartResult<Vec<_>>>()?;
        let panels = prepared.panels().iter().map(|p| {
            let layers = p.chart.layers().iter().map(&layer_semantics).collect::<ChartResult<Vec<_>>>()?;
            Ok(json!({"key":p.key,"row":p.row,"column":p.column,"scale_domains":p.chart.scale_domains(),"layers":layers}))
        }).collect::<ChartResult<Vec<_>>>()?;
        let transforms=self.definition().transforms.iter().filter_map(|node|prepared.transform(node.id).map(|table|json!({"id":node.id,"rows":table.rows(),"schema":table.schema(),"operations":table.operations(),"space":table.space()}))).collect::<Vec<_>>();
        encode(
            &json!({"version":VERSION,"definition_revision":prepared.definition_revision(),"store_revision":data.revision(),"state":StateEnvelope::capture(self.definition(),self.state()),"datasets":datasets,"layers":layers,"panels":panels,"transforms":transforms}),
        )
    }
}
