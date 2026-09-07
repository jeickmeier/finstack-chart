use super::*;
use crate::{FigureRequest, Format, TextMode, ViewMode};
use chart_core::state::InteractionCapture;

#[derive(Deserialize, Default)]
enum Basis {
    #[default]
    Presented,
    Current,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    operation: Operation,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
enum Operation {
    Begin {
        format: String,
        #[serde(default)]
        basis: Basis,
        #[serde(default)]
        interaction: InteractionCapture,
        #[serde(default)]
        full_domain: Option<bool>,
        #[serde(default)]
        width_pt: Option<f64>,
        #[serde(default)]
        height_pt: Option<f64>,
        #[serde(default)]
        dpi: Option<u32>,
        #[serde(default)]
        outline: Option<bool>,
        #[serde(default)]
        output_theme: Option<Box<chart_core::theme::ThemePatch>>,
    },
    Cancel {
        job: Revision,
    },
    Status,
}
impl PortableChart {
    /// Versioned bounded export capture/cancel/status. Begin retains immutable inputs only;
    /// call export_job later. Default captures the presented source with clean committed state.
    pub fn export_control(&mut self, input: &str) -> ChartResult<String> {
        let envelope: Envelope = portable::decode(input)?;
        if envelope.version != portable::VERSION {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported live-export envelope version.",
            ));
        }
        let i = self.get_mut()?;
        let result = match envelope.operation {
            Operation::Begin {
                format,
                basis,
                interaction,
                full_domain,
                width_pt,
                height_pt,
                dpi,
                outline,
                output_theme,
            } => {
                let format = match format.as_str() {
                    "svg" => Format::Svg,
                    "pdf" => Format::Pdf,
                    "png" => Format::Png,
                    _ => {
                        return Err(error(
                            DiagnosticCode::UnsupportedCapability,
                            "Export format must be svg, pdf or png.",
                        ));
                    }
                };
                let (definition, source, state, origin, mut profile) = match basis {
                    Basis::Current => (
                        i.core.definition().clone(),
                        i.core.source(),
                        i.core.state().clone(),
                        None,
                        i.profile.clone(),
                    ),
                    Basis::Presented => {
                        let figure = i.presented.as_ref().ok_or_else(|| {
                            error(
                                DiagnosticCode::Validation,
                                "No presented figure is available for export capture.",
                            )
                        })?;
                        let p = figure.layout().prepared();
                        (
                            p.definition().clone(),
                            p.source().clone(),
                            p.state().clone(),
                            Some(figure.scene().stamp()),
                            figure.metadata().profile.clone(),
                        )
                    }
                };
                if let Some(full) = full_domain {
                    profile.view = if full {
                        ViewMode::FullDomain
                    } else {
                        ViewMode::VisibleView
                    };
                }
                profile.page = PageSize::points(
                    width_pt.unwrap_or(profile.page.width()),
                    height_pt.unwrap_or(profile.page.height()),
                )?;
                if let Some(dpi) = dpi {
                    profile.dpi = dpi;
                }
                if let Some(outline) = outline {
                    profile.text = if outline {
                        TextMode::Outline
                    } else {
                        TextMode::Preserve
                    };
                }
                if let Some(theme) = output_theme {
                    profile.layout.output_theme = *theme;
                }
                let mut request = FigureRequest::new(
                    definition,
                    source,
                    state,
                    i.fonts.clone(),
                    profile,
                    interaction,
                )?
                .with_extensions(i.core.extensions().clone());
                if let Some(origin) = origin {
                    request = request.with_origin_scene(origin)?;
                }
                let manifest = request.manifest()?;
                let job = i.exports.submit(request, format)?;
                let id = job.id();
                i.export_jobs.insert(id, job);
                json!({"job":id,"captured":manifest,"metrics":i.exports.metrics()})
            }
            Operation::Cancel { job } => {
                let pending = i.export_jobs.remove(&job).ok_or_else(|| {
                    error(
                        DiagnosticCode::DisposedHandle,
                        "Unknown or already finished export job.",
                    )
                })?;
                pending.cancellation().cancel();
                drop(pending);
                json!({"job":job,"cancelled":true,"metrics":i.exports.metrics()})
            }
            Operation::Status => json!({"metrics":i.exports.metrics(),"last_export":i.last_export}),
        };
        portable::encode(&result)
    }
    /// Execute and consume one retained job. Bytes are owned by the caller; metadata-only
    /// last-attempt information is available through export_control Status, with no source pin.
    pub fn export_job(&mut self, id: &str) -> ChartResult<Vec<u8>> {
        let id: Revision = portable::decode(&portable::encode(&id)?)?;
        let i = self.get_mut()?;
        let job = i.export_jobs.remove(&id).ok_or_else(|| {
            error(
                DiagnosticCode::DisposedHandle,
                "Unknown or already finished export job.",
            )
        })?;
        match job.run() {
            Ok(artifact) => {
                i.last_export = Some(
                    json!({"job":id,"metadata":artifact.metadata.manifest(),"bytes":artifact.bytes.len(),"format":artifact.format}),
                );
                Ok(artifact.bytes)
            }
            Err(e) => {
                i.last_export = Some(json!({"job":id,"error":e}));
                Err(e)
            }
        }
    }
}
