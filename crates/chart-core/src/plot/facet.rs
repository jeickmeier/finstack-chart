use super::{Data, error};
use crate::{
    ChartResult, DiagnosticCode,
    data::ValueRef,
    grammar::{EmptyPanels, FacetLayout, FacetScales, FacetSpec, GroupValue, PanelKey},
};
/// Facet layout/catalog policy over the existing shared facet engine.
#[derive(Clone, Debug)]
pub struct FacetBuilder {
    fields: Vec<String>,
    layout: FacetLayout,
    order: Option<Vec<PanelKey>>,
    empty: EmptyPanels,
    scales: FacetScales,
    gap: f64,
    collect_guides: bool,
    failure: Option<crate::Diagnostic>,
}
/// Wrap one field's first-seen exact values across two columns by default.
pub fn facet_wrap(field: impl Into<String>) -> FacetBuilder {
    FacetBuilder {
        fields: vec![field.into()],
        layout: FacetLayout::Wrap { columns: 2 },
        order: None,
        empty: EmptyPanels::Keep,
        scales: FacetScales::default(),
        gap: 12.,
        collect_guides: true,
        failure: None,
    }
}
/// Basic row/column facet grid with exact stable panel identities.
pub fn facet_grid(rows: impl Into<String>, columns: impl Into<String>) -> FacetBuilder {
    FacetBuilder {
        fields: vec![rows.into(), columns.into()],
        layout: FacetLayout::Grid,
        ..facet_wrap("")
    }
}
impl FacetBuilder {
    /// Set wrap column count; grid callers choose row/column fields instead.
    pub fn columns(mut self, columns: usize) -> Self {
        if let FacetLayout::Wrap { columns: value } = &mut self.layout {
            *value = columns;
        } else {
            self.failure = Some(error(
                DiagnosticCode::UnsupportedCapability,
                "Column count applies to facet wrap; a grid uses its row/column fields.",
            ));
        }
        self
    }
    /// Retain an explicit catalog/order, including empty panels.
    pub fn order(mut self, order: Vec<PanelKey>) -> Self {
        self.order = Some(order);
        self
    }
    /// Keep or drop empty panels with the existing grid/wrap semantics.
    pub fn empty(mut self, policy: EmptyPanels) -> Self {
        self.empty = policy;
        self
    }
    /// Train x independently per panel when enabled.
    pub fn free_x(mut self, free: bool) -> Self {
        self.scales.free_x = free;
        self
    }
    /// Train y independently per panel when enabled.
    pub fn free_y(mut self, free: bool) -> Self {
        self.scales.free_y = free;
        self
    }
    /// Set nonnegative panel separation in destination units.
    pub fn gap(mut self, gap: f64) -> Self {
        self.gap = gap;
        self
    }
    /// Collect compatible guides once or show per-panel guides.
    pub fn collect_guides(mut self, collect: bool) -> Self {
        self.collect_guides = collect;
        self
    }
    pub(super) fn lower(&self, data: &Data) -> ChartResult<FacetSpec> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        let fields = self
            .fields
            .iter()
            .map(|f| data.field(f).map(|h| h.id()))
            .collect::<ChartResult<Vec<_>>>()?;
        let order = if let Some(order) = &self.order {
            order.clone()
        } else {
            let columns = fields
                .iter()
                .map(|f| {
                    data.batch.column(*f).ok_or_else(|| {
                        error(DiagnosticCode::MissingResource, "Facet field is absent.")
                    })
                })
                .collect::<ChartResult<Vec<_>>>()?;
            let mut seen = std::collections::BTreeSet::new();
            let mut order = vec![];
            for row in 0..data.batch.len() {
                let values = columns
                    .iter()
                    .map(|c| match c.value(row) {
                        Some(ValueRef::Category(v) | ValueRef::Utf8(v)) => {
                            Ok(Some(GroupValue::Text(v.into())))
                        }
                        Some(ValueRef::Int64(v)) => Ok(Some(GroupValue::Int(v))),
                        Some(ValueRef::UInt64(v)) => Ok(Some(GroupValue::UInt(v))),
                        Some(ValueRef::Boolean(v)) => Ok(Some(GroupValue::Boolean(v))),
                        None => Ok(None),
                        _ => Err(error(
                            DiagnosticCode::SchemaConflict,
                            "Facets require exact categorical/text/integer/bool values.",
                        )),
                    })
                    .collect::<ChartResult<Option<Vec<_>>>>()?;
                if let Some(values) = values {
                    let key = PanelKey { values };
                    if seen.insert(key.clone()) {
                        order.push(key);
                    }
                }
            }
            if matches!(self.layout, FacetLayout::Grid) {
                let mut rows = vec![];
                let mut columns = vec![];
                for key in &order {
                    if !rows.contains(&key.values[0]) {
                        rows.push(key.values[0].clone());
                    }
                    if !columns.contains(&key.values[1]) {
                        columns.push(key.values[1].clone());
                    }
                }
                rows.into_iter()
                    .flat_map(|row| {
                        columns.iter().map(move |column| PanelKey {
                            values: vec![row.clone(), column.clone()],
                        })
                    })
                    .collect()
            } else {
                order
            }
        };
        Ok(FacetSpec {
            fields,
            order,
            layout: self.layout.clone(),
            empty: self.empty,
            scales: self.scales,
            gap: self.gap,
            collect_guides: self.collect_guides,
        })
    }
}

pub(super) fn validate_dataset_fields(
    definition: &crate::grammar::ChartDefinition,
    datasets: &[Data],
) -> ChartResult<()> {
    use crate::grammar::{DataRef, FacetTarget};
    let Some(facet) = &definition.facets else {
        return Ok(());
    };
    let expected = facet
        .fields
        .iter()
        .map(|id| {
            datasets[0]
                .batch
                .schema()
                .field(*id)
                .map(|(_, f)| (*id, f.name.as_str()))
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        "Default data is missing a facet field.",
                    )
                })
        })
        .collect::<ChartResult<Vec<_>>>()?;
    for (mut input, target) in definition
        .layers
        .iter()
        .map(|l| (l.data, &l.facet))
        .chain(definition.transforms.iter().map(|t| (t.input, &t.facet)))
    {
        if *target != FacetTarget::Match {
            continue;
        }
        for _ in 0..=definition.transforms.len() {
            match input {
                DataRef::Dataset(id) => {
                    let data = datasets.iter().find(|d| d.id == id).ok_or_else(|| {
                        error(DiagnosticCode::MissingResource, "Facet source is absent.")
                    })?;
                    for (field, name) in &expected {
                        if !data
                            .batch
                            .schema()
                            .field(*field)
                            .is_some_and(|(_, f)| &f.name == name)
                        {
                            return Err(error(
                                DiagnosticCode::SchemaConflict,
                                format!(
                                    "Dataset '{}' must bind facet field '{name}' at the same schema position as the default data; align the columns or explicitly broadcast/target this layer.",
                                    data.name
                                ),
                            ));
                        }
                    }
                    break;
                }
                DataRef::Transform(id) => {
                    input = definition
                        .transforms
                        .iter()
                        .find(|t| t.id == id)
                        .ok_or_else(|| {
                            error(
                                DiagnosticCode::MissingResource,
                                "Facet transform is absent.",
                            )
                        })?
                        .input
                }
            }
        }
    }
    Ok(())
}
