//! Reference palette batches; scale owners retain domains and normalization.
use super::*;
use crate::{
    ChartResult,
    grammar::{ScalePaletteOperation, scale_palette_extensions::ScalePaletteRegistrations},
    interpolate::{Number, Value},
};

#[derive(Default)]
pub(super) struct DiscretePaletteValues {
    pub values: Vec<Value>,
    pub fallback_indices: Vec<usize>,
}
pub(super) fn discrete_values(
    registry: &ScalePaletteRegistrations,
    call: &ScalePaletteOperation,
    domain: &[ScaleKey],
) -> ChartResult<DiscretePaletteValues> {
    if domain.is_empty() {
        return Ok(DiscretePaletteValues::default());
    }
    let output = registry.evaluate(
        call,
        crate::grammar::ScalePaletteDomain::Count(domain.len()),
    )?;
    let values = output.values.unwrap_or_default();
    if let Some(names) = output.names {
        let mut indices = std::collections::BTreeMap::new();
        for (i, name) in names.iter().enumerate() {
            indices.entry(name.as_str()).or_insert(i);
        }
        let mut result = DiscretePaletteValues::default();
        for (i, key) in domain.iter().enumerate() {
            let found = text_key(key)
                .and_then(|text| indices.get(text.as_str()).copied())
                .and_then(|i| values.get(i));
            if found.is_none() {
                result.fallback_indices.push(i);
            }
            result.values.push(found.cloned().unwrap_or(Value::Missing));
        }
        return Ok(result);
    }
    // Source appends one NA slot, without recycling. Preserve a short range;
    // only an actual mark/guide lookup past that slot raises the reference error.
    let mut values = values;
    let mut fallback_indices = vec![];
    if values.len() < domain.len() {
        fallback_indices.push(values.len());
        values.push(Value::Missing);
    }
    values.truncate(domain.len());
    Ok(DiscretePaletteValues {
        values,
        fallback_indices,
    })
}

pub(super) fn text_key(key: &ScaleKey) -> Option<String> {
    match key {
        ScaleKey::Text(s) => Some(s.clone()),
        ScaleKey::Timestamp(v) => Some(v.to_string()),
        _ => super::mapped::discrete_text_key(key),
    }
}

/// One palette evaluation with shared output slots and original observation order.
/// NULL has no mapped aesthetic; an empty vector has missing mapped aesthetics.
pub(crate) struct PaletteBatch<T> {
    pub values: Option<Vec<T>>,
    pub indices: Vec<usize>,
}
// Preserve R's NA/NaN distinction during reference palette pooling. This quiet
// NaN payload is internal mapping state; ordinary numeric transport stays canonical.
pub(super) fn missing_number() -> f64 {
    f64::from_bits(0x7ff8_0000_0000_07a2)
}
fn is_missing_number(value: f64) -> bool {
    value.to_bits() & 0x7fff_ffff_ffff_ffff == missing_number().to_bits()
}
pub(super) fn continuous_values(
    registry: &ScalePaletteRegistrations,
    call: &ScalePaletteOperation,
    inputs: impl Iterator<Item = Number>,
    unknown: &Value,
) -> ChartResult<PaletteBatch<Value>> {
    let mut unique = Vec::new();
    let mut lookup = std::collections::BTreeMap::new();
    let mut indices = Vec::new();
    for value in inputs {
        // R match pools signed zeros, but keeps NA separate from arithmetic NaN.
        let key = if is_missing_number(value.0) {
            missing_number().to_bits()
        } else if value.0.is_nan() {
            f64::NAN.to_bits()
        } else if value.0 == 0. {
            0
        } else {
            value.0.to_bits()
        };
        let index = *lookup.entry(key).or_insert_with(|| {
            unique.push(value);
            unique.len() - 1
        });
        crate::limits::require_within(
            unique.len() <= crate::interpolate::MAX_VALUES,
            "palette input count",
        )?;
        indices.push(index);
    }
    let output = registry.evaluate(
        call,
        crate::grammar::ScalePaletteDomain::Normalized(&unique),
    )?;
    let values = output.values.map(|values| {
        (0..unique.len().max(1))
            .map(|i| match values.get(i) {
                None | Some(Value::Missing | Value::Null) => unknown.clone(),
                Some(Value::Number(Number(v))) if v.is_nan() => unknown.clone(),
                Some(value) => value.clone(),
            })
            .collect()
    });
    Ok(PaletteBatch { values, indices })
}

impl<T> PaletteBatch<T> {
    /// Data frame assignment recycles one value; empty vectors use a missing slot.
    pub(crate) fn for_rows(mut self, count: usize) -> ChartResult<Self> {
        if self.values.is_none() {
            return Ok(self);
        }
        if self.indices.is_empty() && count != 0 {
            self.indices.resize(count, 0);
        } else if self.indices.len() == 1 {
            self.indices.resize(count, self.indices[0]);
        } else if self.indices.len() != count {
            return Err(crate::scales::error(
                crate::DiagnosticCode::Validation,
                "Scale pipeline output must have one value or match the observation count.",
            ));
        }
        Ok(self)
    }
}
