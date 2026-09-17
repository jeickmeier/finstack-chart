//! Versioned key glyphs lower to the same validated portable paths as custom guides.
use super::{
    CustomGuidePath, ExtensionDescriptor, ExtensionRegistry, OperationRef,
    registry::{VersionedMap, reject_native_only},
};
use crate::{ChartResult, LayerId, Limits, Rect, scene::Color, services::Units};
use std::sync::Arc;
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
#[derive(Clone, Debug, Default)]
pub(crate) struct KeyRegistrations(VersionedMap<Arc<dyn CustomKeyGlyph>>);
impl KeyRegistrations {
    fn get(
        &self,
        s: &KeyGlyphSelection,
    ) -> crate::ChartResult<&super::registry::VersionedEntry<Arc<dyn CustomKeyGlyph>>> {
        self.0
            .get(&s.operation, "Selected key glyph is not registered.")
    }
    pub(crate) fn validate(&self, s: &KeyGlyphSelection, portable: bool) -> crate::ChartResult<()> {
        super::extensions::parameter_size(&s.parameters)?;
        let entry = self.get(s)?;
        reject_native_only(
            portable,
            entry.descriptor.portable,
            "A native-only key glyph cannot execute in a portable chart.",
        )?;
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
        super::CustomLegend {
            id: crate::ScaleId::new(1),
            bounds: [0., 0., out.size[0], out.size[1]],
            paths: out.paths.clone(),
            options: Default::default(),
        }
        .validate_local(
            limits,
            "Custom key paths exceed their declared local bounds.",
        )?;
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
        Arc::make_mut(&mut self.keys).0.insert(
            descriptor,
            implementation,
            "Key glyph version is already registered.",
            "key registrations",
        )
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
