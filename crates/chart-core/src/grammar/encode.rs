//! Aesthetic encoding from prepared statistic tables into positional rows.
use super::compiler::{
    EncodedLayer, EncodedRow, backtransform, bin_binding, bin_number, expression_number,
    source_binding, stat_expression_type, statistical_binding,
};
use super::stats::{group_value, number};
use super::*;
use crate::data::DatasetSnapshot;
use crate::provenance::{SourceRef, Target};
use crate::{ChartResult, DiagnosticCode};
use std::{collections::BTreeMap, sync::Arc};

pub(super) fn encode_layer(
    layer: &Layer,
    table: &PreparedTable,
    data: &DatasetSnapshot,
    inherited: &SourceAes,
    profile: Profile,
    extensions: &Arc<ExtensionRegistry>,
    limits: CompileLimits,
) -> ChartResult<EncodedLayer> {
    let domains;
    let mut hierarchy = None;
    let (mut encoded, mapped_size): (Vec<EncodedRow>, bool) = match (&table.rows, &layer.mappings) {
        (PreparedRows::Source(_), Mappings::Source(_)) if layer.geom == Geom::Hierarchy => {
            let (prepared_hierarchy, rows) =
                super::hierarchy::prepare(layer, table, data, inherited, extensions, limits)?;
            hierarchy = prepared_hierarchy;
            domains = DomainContributions::default();
            (rows, false)
        }
        (PreparedRows::Source(rows), Mappings::Source(authored)) => {
            let (aes, bound_domains) = source_binding(layer, authored, inherited, data, profile)?;
            domains = bound_domains;
            let grouping = aes
                .grouping
                .clone()
                .unwrap_or_else(|| aes.group.map_or(Grouping::All, Grouping::Field));
            let index: BTreeMap<_, _> = data.rows().map(|r| (r.key(), r)).collect();
            let catalogs: BTreeMap<_, BTreeMap<&str, f64>> = [&aes.x, &aes.y, &aes.x2, &aes.y2]
                .into_iter()
                .flatten()
                .filter_map(|v| {
                    if let Numeric::Category(id) = v {
                        Some((
                            *id,
                            data.categories(*id)
                                .unwrap_or_default()
                                .iter()
                                .enumerate()
                                .map(|(i, s)| (s.as_str(), i as f64))
                                .collect(),
                        ))
                    } else {
                        None
                    }
                })
                .collect();
            let coordinate = |row: crate::data::RowView<'_>, value: &Numeric| {
                if let Numeric::Category(id) = value {
                    if let Some(crate::data::ValueRef::Category(label)) = row.value(*id) {
                        catalogs.get(id)?.get(label).copied()
                    } else if profile == Profile::Ggplot2_4_0_3 && row.value(*id).is_none() {
                        Some(catalogs.get(id)?.len() as f64)
                    } else {
                        None
                    }
                } else if matches!(value, Numeric::Scaled { scale, .. }
                    if scale.missing.is_some() || profile == Profile::Ggplot2_4_0_3)
                {
                    super::stats::raw_number(row, value)
                } else {
                    number(row, value)
                }
            };
            let encoded = rows
                .iter()
                .map(|r| {
                    let row = index[&r.key];
                    EncodedRow {
                        geo_feature: None,
                        stat_outliers: vec![],
                        outlier_anchor_y: None,
                        missing_aesthetics: 0,
                        recipe_values: BTreeMap::new(),
                        values: BTreeMap::new(),
                        x: aes.x.as_ref().and_then(|v| coordinate(row, v)),
                        y: aes.y.as_ref().and_then(|v| coordinate(row, v)),
                        x2: aes.x2.as_ref().and_then(|v| coordinate(row, v)),
                        y2: aes.y2.as_ref().and_then(|v| coordinate(row, v)),
                        color: None,
                        fill: None,
                        stroke: None,
                        shape: None,
                        opacity: None,
                        alpha: None,
                        stroke_width: None,
                        low: aes.low.as_ref().and_then(|v| coordinate(row, v)),
                        high: aes.high.as_ref().and_then(|v| coordinate(row, v)),
                        size: aes.size.as_ref().and_then(|v| coordinate(row, v)),
                        group: group_value(row, &grouping),
                        ordinal: r.ordinal,
                        target: Target::Source(SourceRef {
                            dataset: table.input.dataset,
                            key: r.key,
                        }),
                        key: Some(r.key),
                    }
                })
                .collect::<Vec<_>>();
            (encoded, aes.size.is_some())
        }
        (PreparedRows::Statistical(rows), Mappings::Statistical(aes)) => {
            let (OutputSchema::Statistical { fields, .. } | OutputSchema::Custom { fields, .. }) =
                &table.schema
            else {
                unreachable!()
            };
            domains = statistical_binding(layer, aes, fields)?;
            let expressions = [&aes.x, &aes.y]
                .into_iter()
                .map(Some)
                .chain([aes.x2.as_ref(), aes.y2.as_ref(), aes.size.as_ref()])
                .map(|mapping| match mapping {
                    Some(StatNumeric::Expression(expr)) => expr
                        .evaluate(
                            rows.len(),
                            ExpressionLimits::default(),
                            |field| stat_expression_type(fields, field),
                            |field, i| {
                                expression_number(rows[i].value(field).and_then(|v| {
                                    backtransform(
                                        v,
                                        &fields.iter().find(|c| &c.field == field)?.space,
                                    )
                                }))
                            },
                        )
                        .map(Some),
                    _ => Ok(None),
                })
                .collect::<ChartResult<Vec<_>>>()?;
            let value = |i: usize, r: &StatisticalRow, n: &StatNumeric, slot: usize| match n {
                StatNumeric::Expression(_) => {
                    expressions[slot].as_ref().and_then(|v| v[i].number())
                }
                StatNumeric::Literal(v) => Some(*v),
                StatNumeric::Field(StatField::Group) => fields
                    .iter()
                    .find(|c| c.field == StatField::Group)
                    .and_then(|c| {
                        if let ValueSpace::Categorical { categories } = &c.space {
                            categories
                                .iter()
                                .position(|v| v == &super::statistics::group_label(&r.group))
                                .map(|i| i as f64)
                        } else {
                            None
                        }
                    }),
                StatNumeric::Field(f) => r.value(f),
            };
            (
                rows.iter()
                    .enumerate()
                    .map(|(i, r)| EncodedRow {
                        geo_feature: None,
                        stat_outliers: r.outliers.clone(),
                        outlier_anchor_y: None,
                        missing_aesthetics: 0,
                        recipe_values: BTreeMap::new(),
                        values: BTreeMap::new(),
                        x: value(i, r, &aes.x, 0),
                        y: value(i, r, &aes.y, 1),
                        x2: aes.x2.as_ref().and_then(|v| value(i, r, v, 2)),
                        y2: aes.y2.as_ref().and_then(|v| value(i, r, v, 3)),
                        color: None,
                        fill: None,
                        stroke: None,
                        shape: None,
                        opacity: None,
                        alpha: None,
                        stroke_width: None,
                        low: None,
                        high: None,
                        size: aes.size.as_ref().and_then(|v| value(i, r, v, 4)),
                        group: Some(r.group.clone()),
                        ordinal: i as u64,
                        target: r.target.clone(),
                        key: None,
                    })
                    .collect(),
                aes.size.is_some(),
            )
        }
        (PreparedRows::Binned(rows), Mappings::Binned(aes)) => {
            let OutputSchema::Binned { fields, .. } = &table.schema else {
                unreachable!()
            };
            let check = |field: &BinField| -> ChartResult<()> {
                if fields.iter().any(|f| f.field == *field) {
                    Ok(())
                } else {
                    Err(error(
                        DiagnosticCode::SchemaConflict,
                        "Mapped bin field is absent from the statistic output schema.",
                    ))
                }
            };
            for mapping in [
                Some(&aes.x),
                Some(&aes.y),
                aes.x2.as_ref(),
                aes.y2.as_ref(),
                aes.size.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                match mapping {
                    BinNumeric::Field(f) => check(f)?,
                    BinNumeric::Expression(e) => {
                        for node in &e.nodes {
                            if let ExpressionNode::Read(f) = node {
                                check(f)?;
                            }
                        }
                    }
                    _ => {}
                }
            }
            domains = bin_binding(layer, aes, &table.space)?;
            let expressions = [&aes.x, &aes.y]
                .into_iter()
                .map(Some)
                .chain([aes.x2.as_ref(), aes.y2.as_ref(), aes.size.as_ref()])
                .map(|mapping| match mapping {
                    Some(BinNumeric::Expression(expr)) => expr
                        .evaluate(
                            rows.len(),
                            ExpressionLimits::default(),
                            |_| Ok(ExpressionType::Number),
                            |field, i| {
                                expression_number(
                                    bin_number(&rows[i], &BinNumeric::Field(*field)).and_then(
                                        |v| {
                                            if matches!(
                                                field,
                                                BinField::Count
                                                    | BinField::Width
                                                    | BinField::Density
                                                    | BinField::NCount
                                                    | BinField::NDensity
                                            ) {
                                                Some(v)
                                            } else {
                                                backtransform(v, &table.space)
                                            }
                                        },
                                    ),
                                )
                            },
                        )
                        .map(Some),
                    _ => Ok(None),
                })
                .collect::<ChartResult<Vec<_>>>()?;
            let value = |i: usize, r: &BinnedRow, n: &BinNumeric, slot: usize| {
                if matches!(n, BinNumeric::Expression(_)) {
                    expressions[slot].as_ref().and_then(|v| v[i].number())
                } else {
                    bin_number(r, n)
                }
            };
            let encoded = rows
                .iter()
                .enumerate()
                .map(|(i, r)| EncodedRow {
                    geo_feature: None,
                    stat_outliers: vec![],
                    outlier_anchor_y: None,
                    missing_aesthetics: 0,
                    recipe_values: BTreeMap::new(),
                    values: BTreeMap::new(),
                    x: value(i, r, &aes.x, 0),
                    y: value(i, r, &aes.y, 1),
                    x2: aes.x2.as_ref().and_then(|v| value(i, r, v, 2)),
                    y2: aes.y2.as_ref().and_then(|v| value(i, r, v, 3)),
                    color: None,
                    fill: None,
                    stroke: None,
                    shape: None,
                    opacity: None,
                    alpha: None,
                    stroke_width: None,
                    low: None,
                    high: None,
                    size: aes.size.as_ref().and_then(|v| value(i, r, v, 4)),
                    group: Some(r.group.clone()),
                    ordinal: i as u64,
                    target: r.target.clone(),
                    key: None,
                })
                .collect();
            (encoded, aes.size.is_some())
        }
        _ => {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Aesthetic stage does not match the statistic output schema; use BinAes for bins and SourceAes for observations.",
            ));
        }
    };
    if layer.geom.run().is_some() && mapped_size {
        return Err(error(
            DiagnosticCode::UnsupportedCapability,
            "Mapped size currently supports point radius and rule stroke width; use constant styling for lines and rectangles.",
        ));
    }
    if let Geom::Area { baseline, .. } = layer.geom {
        if !baseline.is_finite()
            || matches!(
                domains.y_space,
                Some(
                    ValueSpace::Categorical { .. }
                        | ValueSpace::NullableCategorical { .. }
                        | ValueSpace::Timestamp { .. }
                )
            )
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Area baseline requires a finite numeric calculation space.",
            ));
        }
        for row in &mut encoded {
            row.y2 = Some(baseline);
        }
    }
    if matches!(layer.geom, Geom::Ribbon { .. }) {
        for row in &mut encoded {
            if !matches!((row.y, row.y2), (Some(a), Some(b)) if a <= b) {
                row.y = None;
                row.y2 = None;
            }
        }
    }
    if let Geom::Bar { nonnegative, .. } = layer.geom {
        for row in &mut encoded {
            row.x2 = row.x;
            if nonnegative && row.y.is_some_and(|y| y < 0.) {
                row.y = None;
            }
        }
    }
    if profile == Profile::Ggplot2_4_0_3
        && table
            .population_operation()
            .is_some_and(|op| matches!(op.parameters, StatParameters::Summary(_)))
        && let PreparedRows::Statistical(rows) = &table.rows
    {
        // The inspection table retains empty aggregates; ggplot has no generated
        // positional observation for a summary without any usable input.
        encoded.retain(|row| rows.get(row.ordinal as usize).is_none_or(|r| r.count != 0));
    }
    super::geography_geometry::resolve(layer, data, &mut encoded, limits)?;
    super::recipe_emit::resolve(layer, data, table, &mut encoded, &domains, limits)?;
    Ok(EncodedLayer {
        domains,
        encoded,
        hierarchy,
        mapped_size,
    })
}
