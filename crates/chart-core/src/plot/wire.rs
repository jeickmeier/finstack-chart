use super::*;
use crate::portable::BatchWire;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Dataset {
    id: crate::DatasetId,
    name: String,
    batch: BatchWire,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    version: u32,
    profile: Profile,
    definition: ChartDefinition,
    epoch: SourceEpoch,
    data: Vec<Dataset>,
    layers: BTreeMap<String, LayerId>,
    axes: BTreeMap<String, ScaleId>,
    transforms: BTreeMap<String, crate::TransformId>,
    colors: BTreeMap<String, ScaleId>,
    data_limits: DataLimits,
    compile_limits: CompileLimits,
}
fn names<T: Copy + Ord>(
    names: &BTreeMap<String, T>,
    valid: impl IntoIterator<Item = T>,
) -> ChartResult<()> {
    let valid: std::collections::BTreeSet<_> = valid.into_iter().collect();
    let mut seen = std::collections::BTreeSet::new();
    for (name, id) in names {
        validate_name(name)?;
        if !valid.contains(id) || !seen.insert(id) {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Primary envelope component names have missing or duplicate identities.",
            ));
        }
    }
    if seen.len() != valid.len() {
        return Err(error(
            DiagnosticCode::SchemaConflict,
            "Primary envelope omits component names.",
        ));
    }
    Ok(())
}
impl Plot {
    /// Serialize the primary version-1 interchange, retaining names, profile, exact values and IDs.
    /// Native-only extensions reject; registrations are supplied separately when loading.
    pub fn to_json(&self) -> ChartResult<String> {
        self.extensions.validate_portable(&self.definition)?;
        crate::portable::encode(&Envelope {
            version: 1,
            profile: self.profile,
            definition: self.definition.clone(),
            epoch: self.source.get()?.epoch(),
            data: self
                .data
                .iter()
                .map(|d| Dataset {
                    id: d.id,
                    name: d.name.clone(),
                    batch: BatchWire::from_batch(&d.batch),
                })
                .collect(),
            layers: self.layers.clone(),
            axes: self.axes.clone(),
            transforms: self.transforms.clone(),
            colors: self.colors.clone(),
            data_limits: self.data_limits,
            compile_limits: self.compile_limits,
        })
    }
    /// Load owned portable primary authoring without manually reconstructing IDs or three envelopes.
    pub fn from_json(input: &str) -> ChartResult<Self> {
        Self::from_json_with_extensions(input, Arc::new(ExtensionRegistry::new()))
    }
    /// Load with an explicitly supplied immutable registry; payloads cannot register executable code.
    pub fn from_json_with_extensions(
        input: &str,
        extensions: Arc<ExtensionRegistry>,
    ) -> ChartResult<Self> {
        let value: Envelope = crate::portable::decode(input)?;
        if value.version != 1 {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "Unsupported primary authoring envelope version.",
            ));
        }
        extensions.validate_portable(&value.definition)?;
        if value.data.is_empty() {
            return Err(error(
                DiagnosticCode::MissingResource,
                "A primary plot requires default data.",
            ));
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut data = value
            .data
            .into_iter()
            .map(|d| {
                validate_name(&d.name)?;
                if !seen.insert(d.name.clone()) {
                    return Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Duplicate dataset name.",
                    ));
                }
                Ok(Data {
                    id: d.id,
                    name: d.name,
                    batch: Arc::new(d.batch.into_batch_with_limits(value.data_limits)?),
                    timestamp_origins: vec![],
                })
            })
            .collect::<ChartResult<Vec<_>>>()?;
        data::align_timestamp_origins(&mut data);
        names(&value.layers, value.definition.layers.iter().map(|l| l.id))?;
        names(
            &value.transforms,
            value.definition.transforms.iter().map(|t| t.id),
        )?;
        let axis_ids = if value.definition.axes.is_empty() {
            vec![ScaleId::new(0), ScaleId::new(1)]
        } else {
            value.definition.axes.iter().map(|a| a.id).collect()
        };
        names(&value.axes, axis_ids)?;
        names(
            &value.colors,
            value
                .definition
                .layers
                .iter()
                .filter_map(|l| l.color.as_ref().map(|c| c.id)),
        )?;
        let store = DataStore::new(
            value.epoch,
            data.iter()
                .map(|d| (d.id, d.batch.as_ref().clone()))
                .collect(),
            value.data_limits,
        )?;
        let source = store.snapshot();
        facet::validate_dataset_fields(&value.definition, &data)?;
        Compiler::with_extensions(extensions.clone()).validate(
            &value.definition,
            &source,
            value.compile_limits,
        )?;
        Ok(Self {
            owner: fresh_id()?,
            definition: value.definition,
            source,
            data,
            layers: value.layers,
            axes: value.axes,
            transforms: value.transforms,
            colors: value.colors,
            profile: value.profile,
            data_limits: value.data_limits,
            compile_limits: value.compile_limits,
            extensions,
        })
    }
}
