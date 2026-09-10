//! Named host commands resolve identities here and execute the existing reducer/query services.
use super::*;
use chart_core::editing::AnnotationEditor;
use chart_core::state::{FollowMode, SelectionChange};

impl Runtime {
    /// Execute a named primary command with small typed options; never infer source identities.
    pub fn command(
        &mut self,
        name: &str,
        input: &str,
        expected: Option<u64>,
    ) -> ChartResult<String> {
        let args: Value = portable::decode(input)?;
        let required = |key: &str| {
            args.get(key).ok_or_else(|| {
                error(
                    DiagnosticCode::Validation,
                    format!("Missing command option '{key}'."),
                )
            })
        };
        let action = match name {
            "layer_visible" => ChartAction::SetLayerVisible {
                layer: self.layer(&value::<String>(required("layer")?)?)?.id(),
                visible: value(required("visible")?)?,
            },
            "legend_visible" => ChartAction::SetLegendVisible(value(required("visible")?)?),
            "follow" => ChartAction::SetFollow(value::<FollowMode>(required("mode")?)?),
            "resume" => ChartAction::ResumeLatest,
            "reset" => ChartAction::Reset,
            "undo" => ChartAction::Undo,
            "redo" => ChartAction::Redo,
            "clear_inspection" => ChartAction::ClearInspection,
            "select" => ChartAction::Select {
                change: value::<SelectionChange>(required("change")?)?,
                targets: value(required("targets")?)?,
            },
            "hover" => ChartAction::SetHover(value(required("targets")?)?),
            "focus" => ChartAction::SetFocus(value(required("target")?)?),
            "pin" => ChartAction::SetPinned(value(required("target")?)?),
            "annotation" => ChartAction::SetAnnotation(value(required("annotation")?)?),
            "remove_annotation" => ChartAction::RemoveAnnotation(value(required("id")?)?),
            "windows" => ChartAction::SetAxisWindows(value(required("windows")?)?),
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    format!("Unknown primary command '{name}'."),
                ));
            }
        };
        let c = self.get_mut()?.runtime_mut();
        let mut request = c.request(action, ActionOrigin::Programmatic);
        if let Some(expected) = expected {
            request.expected_state = Revision::new(expected);
        }
        portable::encode(&c.dispatch(request)?)
    }
    /// Query named axes or layers over the acknowledged scene, retaining optional stale-scene fences.
    pub fn named_query(
        &mut self,
        name: &str,
        input: &str,
        gesture: bool,
        stamp: Option<SceneStamp>,
    ) -> ChartResult<String> {
        let mut args: Value = portable::decode(input)?;
        if args.get("panel").is_some_and(Value::is_array) {
            args["panel"] =
                serde_json::to_value(chart_core::plot::host::panel_key(args["panel"].clone())?)
                    .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?;
        }
        match name {
            "Navigate" => {
                let names: Vec<String> = value(args.get("axes").ok_or_else(|| {
                    error(
                        DiagnosticCode::Validation,
                        "Navigation requires axis names.",
                    )
                })?)?;
                args["axes"] = serde_json::to_value(
                    names
                        .iter()
                        .map(|n| self.axis(n).map(|a| a.id()))
                        .collect::<ChartResult<Vec<_>>>()?,
                )
                .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))?;
            }
            "SetRange" => {
                let axis: String = value(args.get("axis").ok_or_else(|| {
                    error(DiagnosticCode::Validation, "Range requires an axis name.")
                })?)?;
                args["axis"] = json!(self.axis(&axis)?.id());
            }
            "Series" => {
                let layer: String = value(args.get("layer").ok_or_else(|| {
                    error(
                        DiagnosticCode::Validation,
                        "Series selection requires a layer name.",
                    )
                })?)?;
                let query = json!({"Select":{"region":{"Series":{"layer":self.layer(&layer)?.id(),"panel":args.get("panel")}},"limit":args.get("limit")}});
                return self.query(&portable::encode(&query)?, gesture, stamp);
            }
            _ => {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Unknown named query.",
                ));
            }
        }
        self.query(&portable::encode(&json!({name:args}))?, gesture, stamp)
    }
}
/// Owned editor pins its original scene, independently of host runtime/output disposal.
pub struct Editor(AnnotationEditor);
impl Runtime {
    /// Capture an owned annotation editor through the shared primary policy builder.
    pub fn owned_editor(&self, component: &Component) -> ChartResult<Editor> {
        component.annotation_editor(self.chart()?).map(Editor)
    }
}
impl Editor {
    /// Captured scene fence and original annotation.
    pub fn original(&self) -> ChartResult<String> {
        portable::encode(
            &json!({"stamp":self.0.presented().scene().stamp(),"annotation":self.0.original()}),
        )
    }
    /// Produce a proposal from total displacement in the pinned destination frame.
    pub fn preview(&self, dx: f64, dy: f64) -> ChartResult<String> {
        portable::encode(&self.0.preview(self.0.presented().scene().stamp(), dx, dy)?)
    }
    /// Produce one bounded keyboard-equivalent proposal.
    pub fn nudge(&self, horizontal: bool, forward: bool, steps: u32) -> ChartResult<String> {
        portable::encode(&self.0.nudge(
            self.0.presented().scene().stamp(),
            horizontal,
            forward,
            steps,
        )?)
    }
}
