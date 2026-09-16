//! Reference facet policy and deterministic source catalog planning.
use super::*;
use crate::{ChartResult, DiagnosticCode};
use std::collections::BTreeSet;
/// Reference facet catalog/population controls; absence on FacetSpec preserves legacy behavior.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FacetPolicy {
    /// Grid rows follow table order; false reverses their physical order.
    pub as_table: bool,
    /// Wrap panel growth and starting corner.
    pub direction: FacetDirection,
    /// Physical size policy over expanded axis spans.
    pub space: FacetSpace,
    /// Wrap strip side.
    pub strip_position: FacetStripPosition,
    /// Grid strip switches.
    pub switch: FacetSwitch,
    /// Interior axis inclusion.
    pub axes: FacetAxes,
    /// Interior tick-label inclusion.
    pub axis_labels: FacetAxes,
    /// Portable strip labeller.
    pub labeller: FacetLabeller,
    /// Preserve an explicit authored panel order instead of recomputing live source catalogs.
    pub fixed_catalog: bool,
    /// Number of leading row variables for a grid; remaining variables form columns.
    pub row_fields: usize,
    /// Keep only observed tuples within each grid side (wrap: observed full tuples).
    pub drop: bool,
    /// Train on statistical output rather than additionally retaining source position ranges.
    pub shrink: bool,
    /// Flattened variable indices to marginalize; outer nested margins include inner variables.
    pub margins: Vec<usize>,
    /// Optional ordered level catalogs per variable; missing identities remain distinct.
    pub levels: Vec<Option<Vec<GroupValue>>>,
    /// Canonical field names resolved by authoring for partial-field layer broadcast.
    pub field_names: Vec<String>,
}
impl Default for FacetPolicy {
    fn default() -> Self {
        Self {
            as_table: true,
            direction: Default::default(),
            space: Default::default(),
            strip_position: Default::default(),
            switch: Default::default(),
            axes: Default::default(),
            axis_labels: FacetAxes::All,
            labeller: Default::default(),
            fixed_catalog: false,
            row_fields: 1,
            drop: true,
            shrink: true,
            margins: vec![],
            levels: vec![],
            field_names: vec![],
        }
    }
}
pub(crate) fn catalog(
    policy: &FacetPolicy,
    layout: &FacetLayout,
    observed: &[Vec<GroupValue>],
    levels: &[Vec<GroupValue>],
) -> ChartResult<Vec<PanelKey>> {
    let count = levels.len();
    if count == 0
        || count > 32
        || matches!(layout, FacetLayout::Grid) && policy.row_fields > count
        || policy.margins.iter().any(|i| *i >= count)
    {
        return Err(error(
            DiagnosticCode::Validation,
            "Reference facet variables and margin indices are invalid.",
        ));
    }
    let side = |start: usize, end: usize| -> ChartResult<Vec<Vec<GroupValue>>> {
        if start == end {
            return Ok(vec![vec![]]);
        }
        let catalogs = &levels[start..end];
        let mut tuples = if policy.drop {
            observed
                .iter()
                .map(|r| r[start..end].to_vec())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
        } else {
            let mut rows = vec![vec![]];
            for catalog in catalogs {
                let mut next = vec![];
                for row in &rows {
                    for value in catalog {
                        let mut row = row.clone();
                        row.push(value.clone());
                        next.push(row);
                        if next.len() > 256 {
                            return Err(error(
                                DiagnosticCode::ResourceLimit,
                                "Facet catalog exceeds 256 panels.",
                            ));
                        }
                    }
                }
                rows = next;
            }
            rows
        };
        tuples.sort_by_key(|row| {
            row.iter()
                .zip(catalogs)
                .map(|(v, c)| c.iter().position(|x| x == v).unwrap_or(c.len()))
                .collect::<Vec<_>>()
        });
        let original = tuples.clone();
        for index in &policy.margins {
            if (*index >= start) && (*index < end) {
                for row in &original {
                    let mut margin = row.clone();
                    for value in &mut margin[*index - start..] {
                        *value = GroupValue::All;
                    }
                    if !tuples.contains(&margin) {
                        tuples.push(margin);
                    }
                }
            }
        }
        tuples.sort_by_key(|row| {
            row.iter()
                .zip(catalogs)
                .map(|(v, c)| c.iter().position(|x| x == v).unwrap_or(c.len()))
                .collect::<Vec<_>>()
        });
        Ok(tuples)
    };
    let (rows, columns) = match layout {
        FacetLayout::Wrap { .. } => (side(0, count)?, vec![vec![]]),
        FacetLayout::Grid => (side(0, policy.row_fields)?, side(policy.row_fields, count)?),
    };
    if rows
        .len()
        .checked_mul(columns.len())
        .is_none_or(|n| n > 256)
    {
        return Err(error(
            DiagnosticCode::ResourceLimit,
            "Facet catalog exceeds 256 panels.",
        ));
    }
    Ok(rows
        .into_iter()
        .flat_map(|row| {
            columns.iter().map(move |column| PanelKey {
                values: row.iter().chain(column).cloned().collect(),
            })
        })
        .collect())
}
pub(crate) fn side_key(key: &PanelKey, start: usize, end: usize) -> GroupValue {
    GroupValue::Interaction(
        key.values[start..end]
            .iter()
            .enumerate()
            .map(|(i, v)| (crate::FieldId::new((start + i) as u64), v.clone()))
            .collect(),
    )
}
/// Axis population identity: fixed figure, reference grid column/row, or individual panel.
pub(crate) fn sharing_group(spec: &FacetSpec, key: &PanelKey, horizontal: bool) -> GroupValue {
    let free = if horizontal {
        spec.scales.free_x
    } else {
        spec.scales.free_y
    };
    if !free {
        return GroupValue::All;
    }
    if let (Some(policy), FacetLayout::Grid) = (&spec.reference, &spec.layout) {
        if horizontal {
            side_key(key, policy.row_fields, spec.fields.len())
        } else {
            side_key(key, 0, policy.row_fields)
        }
    } else {
        side_key(key, 0, spec.fields.len())
    }
}

/// Resolve a reference catalog from the same snapshot used by preparation.
pub(crate) fn resolve<'a>(
    definition: &'a ChartDefinition,
    snapshot: &crate::data::StoreSnapshot,
    limits: CompileLimits,
) -> ChartResult<std::borrow::Cow<'a, ChartDefinition>> {
    let Some(spec) = definition.facets.as_ref() else {
        return Ok(std::borrow::Cow::Borrowed(definition));
    };
    let Some(policy) = spec.reference.as_ref().filter(|p| !p.fixed_catalog) else {
        return Ok(std::borrow::Cow::Borrowed(definition));
    };
    let n = policy.field_names.len();
    if n == 0 || n != spec.fields.len() {
        return Err(error(
            DiagnosticCode::Validation,
            "Reference facet field names must match the field list.",
        ));
    }
    let mut observed = BTreeSet::new();
    let mut partial = BTreeSet::new();
    let mut complete = false;
    let mut levels = vec![vec![]; n];
    let mut categorical = vec![false; n];
    let mut remaining = limits.max_prepared_rows;
    for layer in &definition.layers {
        if layer.facet != FacetTarget::Match {
            continue;
        }
        let mut input = layer.data;
        let mut filters = layer.filters.clone();
        for _ in 0..=definition.transforms.len() {
            let DataRef::Transform(id) = input else { break };
            let node = definition
                .transforms
                .iter()
                .find(|t| t.id == id)
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        "Facet transform input is absent.",
                    )
                })?;
            filters.extend(node.filters.clone());
            input = node.input;
        }
        let DataRef::Dataset(id) = input else {
            return Err(error(
                DiagnosticCode::Validation,
                "Facet input transform graph is cyclic.",
            ));
        };
        let data = snapshot.dataset(id)?;
        let fields = policy
            .field_names
            .iter()
            .map(|name| {
                data.schema()
                    .fields()
                    .iter()
                    .find(|f| &f.name == name)
                    .map(|f| f.id)
            })
            .collect::<Vec<_>>();
        if fields.iter().all(Option::is_none) {
            continue;
        }
        complete |= fields.iter().all(Option::is_some);
        for row in data.rows().filter(|r| {
            filters
                .iter()
                .all(|f| stats::filter_matches(*r, f) == Some(true))
        }) {
            remaining = remaining.checked_sub(1).ok_or_else(|| {
                error(
                    DiagnosticCode::ResourceLimit,
                    "Facet catalog source population exceeds budget.",
                )
            })?;
            let values = fields
                .iter()
                .map(|f| f.map(|f| row_value(row, f)))
                .collect::<Vec<_>>();
            for (i, v) in values.iter().enumerate() {
                if let Some(v) = v
                    && !levels[i].contains(v)
                {
                    levels[i].push(v.clone());
                }
            }
            if let Some(values) = values.iter().cloned().collect::<Option<Vec<_>>>() {
                observed.insert(values);
            } else {
                partial.insert(values);
            }
            if observed.len() > 256 || partial.len() > 256 {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Facet catalog exceeds 256 distinct input tuples.",
                ));
            }
        }
        for (i, field) in fields.iter().enumerate() {
            if let Some(field) = field {
                for chunk in data.chunks() {
                    if let Some(column) = chunk.batch().column(*field)
                        && let crate::data::ColumnValues::Categorical { dictionary, .. } =
                            column.values()
                    {
                        categorical[i] = true;
                        let mut ordered = dictionary
                            .iter()
                            .map(|v| GroupValue::Text(v.clone()))
                            .collect::<Vec<_>>();
                        for v in &levels[i] {
                            if !ordered.contains(v) {
                                ordered.push(v.clone());
                            }
                        }
                        levels[i] = ordered;
                    }
                }
            }
        }
    }
    if !complete || observed.is_empty() && policy.drop {
        return Err(error(
            DiagnosticCode::Validation,
            "At least one layer must contain all facet variables.",
        ));
    }
    let base = observed.clone();
    for row in partial {
        for original in &base {
            let full = row
                .iter()
                .zip(original)
                .map(|(v, o)| v.clone().unwrap_or_else(|| o.clone()))
                .collect::<Vec<_>>();
            observed.insert(full);
            if observed.len() > 256 {
                return Err(error(
                    DiagnosticCode::ResourceLimit,
                    "Facet catalog exceeds 256 expanded tuples.",
                ));
            }
        }
    }
    for (i, level) in levels.iter_mut().enumerate() {
        if let Some(explicit) = policy.levels.get(i).and_then(Option::as_ref) {
            *level = explicit.clone();
            if observed.iter().any(|r| r[i] == GroupValue::Missing)
                && !level.contains(&GroupValue::Missing)
            {
                level.push(GroupValue::Missing);
            }
            if observed.iter().any(|r| !level.contains(&r[i])) {
                return Err(error(
                    DiagnosticCode::Validation,
                    "Observed facet value is absent from declared levels.",
                ));
            }
        } else {
            if !categorical[i] {
                level.sort();
            }
            if let Some(index) = level.iter().position(|v| *v == GroupValue::Missing) {
                let missing = level.remove(index);
                level.push(missing);
            }
        }
    }
    let mut result = definition.clone();
    result.facets.as_mut().expect("facets").order = catalog(
        policy,
        &spec.layout,
        &observed.into_iter().collect::<Vec<_>>(),
        &levels,
    )?;
    Ok(std::borrow::Cow::Owned(result))
}

/// First direction is growth, second is the starting edge of subsequent rows/columns.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FacetDirection {
    /// Left-to-right, from top.
    #[default]
    Lt,
    /// Top-to-bottom, from left.
    Tl,
    /// Right-to-left, from top.
    Rt,
    /// Top-to-bottom, from right.
    Tr,
    /// Left-to-right, from bottom.
    Lb,
    /// Bottom-to-top, from left.
    Bl,
    /// Right-to-left, from bottom.
    Rb,
    /// Bottom-to-top, from right.
    Br,
}
/// Whether physical panel widths/heights follow expanded scale spans.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FacetSpace {
    /// Equal physical dimensions.
    #[default]
    Fixed,
    /// Proportional widths.
    FreeX,
    /// Proportional heights.
    FreeY,
    /// Proportional widths and heights.
    Free,
}
impl FacetSpace {
    pub(crate) fn free(self, horizontal: bool) -> bool {
        matches!(self, Self::Free)
            || matches!(
                (self, horizontal),
                (Self::FreeX, true) | (Self::FreeY, false)
            )
    }
}
/// Facet wrap strip side.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FacetStripPosition {
    /// Above the panel.
    #[default]
    Top,
    /// Below the panel.
    Bottom,
    /// Before the panel.
    Left,
    /// After the panel.
    Right,
}
/// Grid strips moved from top/right to bottom/left.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FacetSwitch {
    /// Retain top and right strips.
    #[default]
    None,
    /// Move column strips below.
    X,
    /// Move row strips left.
    Y,
    /// Move both strip families.
    Both,
}
/// Additional interior axes or labels; exterior axes always remain eligible.
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum FacetAxes {
    /// Exterior margins only.
    #[default]
    Margins,
    /// Include horizontal interiors.
    AllX,
    /// Include vertical interiors.
    AllY,
    /// Include both interiors.
    All,
}
impl FacetAxes {
    pub(crate) fn includes(self, horizontal: bool) -> bool {
        matches!(self, Self::All)
            || matches!((self, horizontal), (Self::AllX, true) | (Self::AllY, false))
    }
}
/// Portable labeller recipes use typed panel values and canonical variable names.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FacetLabeller {
    /// Reuse a registered vector formatter with logical values and canonical variable names.
    pub registered: Option<FacetLabelOperation>,
    /// Prefix each value with its variable name.
    pub variable_names: bool,
    /// Prefix only when the strip includes multiple variables.
    pub context: bool,
    /// Join name/value components with this separator.
    pub separator: String,
    /// Place variables on separate lines; otherwise join with comma-space.
    pub multiline: bool,
    /// Optional maximum character width for word wrapping (not parsed mathematics).
    pub wrap_width: Option<usize>,
    /// Per-variable, exact logical value-to-label substitutions.
    pub lookup: std::collections::BTreeMap<String, std::collections::BTreeMap<String, String>>,
}
impl Default for FacetLabeller {
    fn default() -> Self {
        Self {
            registered: None,
            variable_names: false,
            context: false,
            separator: ": ".into(),
            multiline: true,
            wrap_width: None,
            lookup: Default::default(),
        }
    }
}

pub(crate) fn coordinates(spec: &FacetSpec, index: usize, total: usize) -> (usize, usize) {
    let FacetLayout::Wrap { columns } = spec.layout else {
        unreachable!()
    };
    let Some(policy) = &spec.reference else {
        return (index / columns, index % columns);
    };
    let columns = if policy.space.free(true) {
        total
    } else if policy.space.free(false) {
        1
    } else {
        columns.min(total).max(1)
    };
    let rows = total.div_ceil(columns);
    let vertical = matches!(
        policy.direction,
        FacetDirection::Tl | FacetDirection::Tr | FacetDirection::Bl | FacetDirection::Br
    );
    let (mut row, mut col) = if vertical {
        (index % rows, index / rows)
    } else {
        (index / columns, index % columns)
    };
    if matches!(
        policy.direction,
        FacetDirection::Lb | FacetDirection::Bl | FacetDirection::Rb | FacetDirection::Br
    ) {
        row = rows - 1 - row;
    }
    if matches!(
        policy.direction,
        FacetDirection::Rt | FacetDirection::Tr | FacetDirection::Rb | FacetDirection::Br
    ) {
        col = columns - 1 - col;
    }
    (row, col)
}
pub(crate) fn label(policy: &FacetPolicy, key: &PanelKey, start: usize, end: usize) -> String {
    let labels = (start..end)
        .map(|i| {
            let name = &policy.field_names[i];
            let raw = if key.values[i] == GroupValue::All {
                "(all)".into()
            } else {
                key.values[i].label()
            };
            let value = policy
                .labeller
                .lookup
                .get(name)
                .and_then(|map| map.get(&raw))
                .cloned()
                .unwrap_or(raw);
            if policy.labeller.variable_names || policy.labeller.context && end - start > 1 {
                format!("{name}{}{value}", policy.labeller.separator)
            } else {
                value
            }
        })
        .collect::<Vec<_>>();
    let text = labels.join(if policy.labeller.multiline {
        "\n"
    } else {
        ", "
    });
    if let Some(width) = policy.labeller.wrap_width.filter(|n| *n > 0) {
        text.lines()
            .map(|line| {
                let mut result = String::new();
                let mut n = 0;
                for word in line.split_whitespace() {
                    let len = word.chars().count();
                    if n > 0 {
                        if n + 1 + len > width {
                            result.push('\n');
                            n = 0;
                        } else {
                            result.push(' ');
                            n += 1;
                        }
                    }
                    result.push_str(word);
                    n += len;
                }
                result
            })
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        text
    }
}

/// Existing formatter registration invoked once for a complete strip-variable vector.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FacetLabelOperation {
    /// Installed versioned guide formatter identity.
    pub operation: OperationRef,
    /// Bounded declarative formatter settings.
    pub parameters: serde_json::Value,
}

/// Reference grid scales share all panels on their row or column.
pub(crate) fn axis_scope(
    definition: &ChartDefinition,
    scope: Option<&super::facets::PanelScope>,
    horizontal: bool,
) -> Option<super::facets::PanelScope> {
    let mut scope = scope?.clone();
    scope.axis_group = true;
    if let Some(spec) = &definition.facets
        && let (Some(policy), FacetLayout::Grid) = (&spec.reference, &spec.layout)
    {
        let range = if horizontal {
            0..policy.row_fields
        } else {
            policy.row_fields..spec.fields.len()
        };
        for i in range {
            scope.key.values[i] = GroupValue::All;
        }
    }
    Some(scope)
}

pub(crate) fn row_value(row: crate::data::RowView<'_>, field: crate::FieldId) -> GroupValue {
    stats::group_value(row, &Grouping::Interaction(vec![field]))
        .and_then(|group| group.component(field).cloned())
        .unwrap_or(GroupValue::Missing)
}
