//! Versioned key glyphs lower to the same validated portable paths as custom guides.
use super::{CustomGuidePath, ExtensionDescriptor, ExtensionRegistry, OperationRef};
use crate::{ChartResult, DiagnosticCode, LayerId, Limits, Rect, scene::Color, services::Units};
use std::{collections::BTreeMap, sync::Arc};
/// Portable selection of an explicitly registered legend-key implementation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyGlyphSelection {
    /// Exact registered identity and version.
    pub operation: OperationRef,
    /// Bounded declarative parameters, validated before drawing.
    pub parameters: serde_json::Value,
}
/// Resolved destination-space key context, after guide-only aesthetic overrides.
pub struct KeyGlyphInput<'a> {
    /// Original layer identity.
    pub layer: LayerId,
    /// Original semantic guide value.
    pub value: &'a crate::composition::ScaleValue,
    /// Selected guide occurrence, including repeated values.
    pub index: usize,
    /// Explicit parameter value.
    pub parameters: &'a serde_json::Value,
    /// Destination units, with no implicit DPI lookup.
    pub units: Units,
    /// Suggested local key box.
    pub bounds: Rect,
    /// Resolved point radius.
    pub radius: f64,
    /// Resolved key color.
    pub color: Color,
    /// Optional resolved interior color.
    pub fill: Option<Color>,
    /// Resolved outline color.
    pub stroke: Color,
    /// Resolved outline width.
    pub stroke_width: f64,
    /// Remaining destination resource limits.
    pub limits: Limits,
}
/// Bounded vector-only output; supplied glyph outlines may also be retained paths.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyGlyphOutput {
    /// Positive local dimensions; origin is zero in each direction.
    pub size: [f64; 2],
    /// Ordered ink using existing portable path/paint types.
    pub paths: Vec<CustomGuidePath>,
}
/// Trusted, immutable key implementation. No dynamic code or host objects are loaded.
pub trait CustomKeyGlyph: Send + Sync {
    /// Identity, portability and exact batch invalidation policy.
    fn descriptor(&self) -> ExtensionDescriptor;
    /// Validate the parameter schema before any render callback.
    fn validate(&self, parameters: &serde_json::Value) -> ChartResult<()>;
    /// Emit one key for a complete resolved guide context.
    fn draw(&self, input: KeyGlyphInput<'_>) -> ChartResult<KeyGlyphOutput>;
}
#[derive(Clone)]
struct Entry {
    descriptor: ExtensionDescriptor,
    implementation: Arc<dyn CustomKeyGlyph>,
}
impl std::fmt::Debug for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.descriptor.fmt(f)
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct KeyRegistrations {
    entries: BTreeMap<(String, u64), Entry>,
}
impl KeyRegistrations {
    fn get(&self, s: &KeyGlyphSelection) -> ChartResult<&Entry> {
        self.entries
            .get(&(s.operation.id.clone(), s.operation.version.get()))
            .ok_or_else(|| {
                super::error(
                    DiagnosticCode::UnsupportedCapability,
                    "Selected key glyph is not registered.",
                )
            })
    }
    pub(crate) fn validate(&self, s: &KeyGlyphSelection, portable: bool) -> ChartResult<()> {
        super::extensions::parameter_size(&s.parameters)?;
        let entry = self.get(s)?;
        if portable && !entry.descriptor.portable {
            return Err(super::error(
                DiagnosticCode::UnsupportedCapability,
                "A native-only key glyph cannot execute in a portable chart.",
            ));
        }
        entry.implementation.validate(&s.parameters)
    }
    pub(crate) fn draw(
        &self,
        s: &KeyGlyphSelection,
        input: KeyGlyphInput<'_>,
    ) -> ChartResult<KeyGlyphOutput> {
        self.validate(s, false)?;
        let limits = input.limits;
        let out = self.get(s)?.implementation.draw(input)?;
        // Reuse the existing complete vector guide resource/geometry validator.
        super::CustomLegend {
            id: crate::ScaleId::new(1),
            bounds: [0., 0., out.size[0], out.size[1]],
            paths: out.paths.clone(),
            options: Default::default(),
        }
        .validate(limits)?;
        for p in &out.paths {
            if let Some(b) = p.geometry.bounds(0.01, limits.max_path_commands)? {
                // Path bounds deliberately include the 0.01 lowering-error envelope.
                let tolerance = 0.01 + 1e-8 * out.size[0].max(out.size[1]).max(1.);
                if b.origin().x() < -tolerance
                    || b.origin().y() < -tolerance
                    || b.max_x() > out.size[0] + tolerance
                    || b.max_y() > out.size[1] + tolerance
                {
                    return Err(super::error(
                        DiagnosticCode::Validation,
                        "Custom key paths exceed their declared local bounds.",
                    ));
                }
            }
        }
        Ok(out)
    }
}
impl ExtensionRegistry {
    /// Install one immutable version; duplicate identities and excess registrations reject.
    pub fn register_key_glyph(
        &mut self,
        implementation: Arc<dyn CustomKeyGlyph>,
    ) -> ChartResult<()> {
        let descriptor = implementation.descriptor();
        super::extensions::validate_descriptor(&descriptor)?;
        let entries = &mut Arc::make_mut(&mut self.keys).entries;
        let key = (
            descriptor.operation.id.clone(),
            descriptor.operation.version.get(),
        );
        if entries.contains_key(&key) {
            return Err(super::error(
                DiagnosticCode::SchemaConflict,
                "Key glyph version is already registered.",
            ));
        }
        crate::limits::require_within(entries.len() < 64, "key registrations")?;
        entries.insert(
            key,
            Entry {
                descriptor,
                implementation,
            },
        );
        Ok(())
    }
    pub(crate) fn validate_key_selections(
        &self,
        definition: &super::ChartDefinition,
        portable: bool,
    ) -> ChartResult<()> {
        for selection in definition
            .layers
            .iter()
            .filter_map(|l| l.legend.as_ref()?.registered_key.as_ref())
        {
            self.keys.validate(selection, portable)?;
        }
        Ok(())
    }
}
