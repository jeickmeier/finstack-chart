//! Registered guides consume trained metadata and lower to the existing vector guide.
use super::{CustomLegend, ExtensionDescriptor, ExtensionRegistry, LegendOptions, OperationRef};
use crate::{ChartResult, DiagnosticCode, Limits, ScaleId, scene::Color, services::Units};
use std::{collections::BTreeMap, sync::Arc};
/// Exact installed guide implementation and bounded declarative parameters.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuideDrawingSelection {
    /// Registered identity and version.
    pub operation: OperationRef,
    /// Implementation-specific checked schema.
    pub parameters: serde_json::Value,
}
/// Trained, ordered metadata at the common destination boundary.
pub struct GuideDrawingInput<'a> {
    /// Stable scale identity.
    pub scale: ScaleId,
    /// Title after explicit guide overrides.
    pub title: &'a str,
    /// Final ordered labels, including duplicates.
    pub labels: &'a [String],
    /// Original semantic values when retained by key training.
    pub values: &'a [crate::composition::ScaleValue],
    /// Resolved primary colors for key guides.
    pub colors: &'a [Color],
    /// Complete continuous/discrete color scale guide, including gradient samples.
    pub color_guide: Option<&'a crate::scales::ColorLegend>,
    /// Checked parameter payload.
    pub parameters: &'a serde_json::Value,
    /// Explicit destination units.
    pub units: Units,
    /// Destination font size in the same units as output paths.
    pub font_size: f64,
    /// Destination resource budgets.
    pub limits: Limits,
}
/// Immutable drawing callback; text uses explicitly provided glyph outlines.
pub trait CustomGuideDrawing: Send + Sync {
    /// Stable identity and portability/invalidation contract.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate bounded parameters before preparing the chart.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Emit intrinsic dimensions and portable paths through the common guide renderer.
    fn draw(&self, input: GuideDrawingInput<'_>) -> ChartResult<super::KeyGlyphOutput>;
}
#[derive(Clone)]
struct Entry {
    descriptor: ExtensionDescriptor,
    implementation: Arc<dyn CustomGuideDrawing>,
}
impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.descriptor.fmt(f)
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct GuideDrawingRegistrations {
    entries: BTreeMap<(String, u64), Entry>,
}
impl GuideDrawingRegistrations {
    fn get(&self, s: &GuideDrawingSelection) -> ChartResult<&Entry> {
        self.entries
            .get(&(s.operation.id.clone(), s.operation.version.get()))
            .ok_or_else(|| {
                super::error(
                    DiagnosticCode::UnsupportedCapability,
                    "Selected guide drawing is not registered.",
                )
            })
    }
    pub(crate) fn validate(&self, s: &GuideDrawingSelection, portable: bool) -> ChartResult<()> {
        super::extensions::parameter_size(&s.parameters)?;
        let e = self.get(s)?;
        if portable && !e.descriptor.portable {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "Native-only guide drawing cannot execute in a portable chart.",
            ));
        }
        e.implementation.validate(&s.parameters)
    }
    pub(crate) fn draw(
        &self,
        s: &GuideDrawingSelection,
        input: GuideDrawingInput<'_>,
        mut options: LegendOptions,
    ) -> ChartResult<CustomLegend> {
        self.validate(s, false)?;
        let id = input.scale;
        let limits = input.limits;
        let output = self.get(s)?.implementation.draw(input)?;
        // Drawing owns its title; retain only box placement controls outside its bounds.
        options.title = None;
        options.registered = None;
        let guide = CustomLegend {
            id,
            bounds: [0., 0., output.size[0], output.size[1]],
            paths: output.paths,
            options,
        };
        guide.validate(limits)?;
        for path in &guide.paths {
            if let Some(b) = path.geometry.bounds(0.01, limits.max_path_commands)? {
                // Path bounds deliberately include the 0.01 lowering-error envelope.
                let tolerance = 0.01 + 1e-8 * output.size[0].max(output.size[1]).max(1.);
                if b.origin().x() < -tolerance
                    || b.origin().y() < -tolerance
                    || b.max_x() > output.size[0] + tolerance
                    || b.max_y() > output.size[1] + tolerance
                {
                    return Err(super::error(
                        DiagnosticCode::Validation,
                        "Registered guide path exceeds declared bounds.",
                    ));
                }
            }
        }
        Ok(guide)
    }
}
impl ExtensionRegistry {
    /// Install an immutable drawing implementation, without loading executable content.
    pub fn register_guide_drawing(
        &mut self,
        implementation: Arc<dyn CustomGuideDrawing>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        super::extensions::validate_descriptor(&descriptor)?;
        let entries = &mut Arc::make_mut(&mut self.guide_drawing).entries;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        if entries.contains_key(&key) {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "Guide drawing version is already registered.",
            ));
        }
        crate::limits::require_within(entries.len() < 64, "guide drawing registrations")?;
        entries.insert(
            key,
            Entry {
                descriptor,
                implementation,
            },
        );
        Ok(())
    }
    pub(crate) fn validate_guide_drawing(
        &self,
        definition: &super::ChartDefinition,
        portable: bool,
    ) -> ChartResult<()> {
        for selection in definition
            .legends
            .values()
            .filter_map(|o| o.registered.as_ref())
            .chain(
                definition
                    .custom_legends
                    .iter()
                    .filter_map(|l| l.options.registered.as_ref()),
            )
        {
            self.guide_drawing.validate(selection, portable)?;
        }
        if definition
            .custom_legends
            .iter()
            .any(|l| l.options.registered.is_some())
        {
            return Err(super::error(
                DiagnosticCode::Validation,
                "Registered drawing selects a trained guide, not static custom content.",
            ));
        }
        Ok(())
    }
}
