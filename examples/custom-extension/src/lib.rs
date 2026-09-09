//! FIX-17 external-style implementation using only chart-core's supported public APIs.
//! This example is shared by proof hosts; production core/export do not depend on it.
pub mod authoring;
pub mod shapes;
use chart_core::{data::*, grammar::*, layout::*, provenance::*, state::*, transaction::*, *};
use std::{collections::BTreeMap, sync::Arc};
/// Registered density histogram operation version.
pub const HISTOGRAM: &str = "example.density_histogram";
/// Registered portable chamfered bar geometry.
pub const BARS: &str = "example.chamfered_bars";
/// Registered native-only bar geometry.
pub const NATIVE_BARS: &str = "example.native_bars";
/// Exact native painter identity expected in the GPUI host.
pub const PAINTER: &str = "example.native_bar_paint";
fn error(code: DiagnosticCode, message: impl Into<String>) -> Diagnostic {
    Diagnostic::error(
        code,
        message,
        "Use the documented density histogram fixture inputs and register version one explicitly.",
    )
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameters {
    input: Numeric,
    edges: Vec<f64>,
}
fn params(p: &ExtensionParameters) -> ChartResult<Parameters> {
    serde_json::from_value(p.values.clone())
        .map_err(|e| error(DiagnosticCode::Validation, e.to_string()))
}
fn field(name: &str) -> StatField {
    StatField::Custom(name.into())
}
/// Explicit histogram with density heights and generated left/right/density/count schema.
/// Finite outliers are errors; null/non-finite inputs follow the supplied invalid policy.
pub struct DensityHistogram;
impl CustomStat for DensityHistogram {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(HISTOGRAM, Revision::new(1), true)
    }
    fn schema(
        &self,
        data: &DatasetSnapshot,
        parameters: &ExtensionParameters,
        limits: CompileLimits,
    ) -> ChartResult<Vec<StatColumn>> {
        let p = params(parameters)?;
        if p.edges.len() > limits.max_edges {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Histogram edge budget exceeded.",
            ));
        }
        if p.edges.len() < 2
            || p.edges.iter().any(|x| !x.is_finite())
            || p.edges
                .windows(2)
                .any(|w| w[0] >= w[1] || !(w[1] - w[0]).is_finite())
        {
            return Err(error(
                DiagnosticCode::NumericalDomain,
                "Histogram edges must be finite, strictly increasing and have finite widths.",
            ));
        }
        let space = extension_input_space(data, &p.input, &parameters.space)?;
        Ok(vec![
            StatColumn {
                field: field("left"),
                kind: GeneratedKind::Float64,
                nullable: false,
                space: space.clone(),
            },
            StatColumn {
                field: field("right"),
                kind: GeneratedKind::Float64,
                nullable: false,
                space,
            },
            StatColumn {
                field: field("density"),
                kind: GeneratedKind::Float64,
                nullable: false,
                space: ValueSpace::Data,
            },
            StatColumn {
                field: StatField::Count,
                kind: GeneratedKind::UInt64,
                nullable: false,
                space: ValueSpace::Data,
            },
        ])
    }
    fn evaluate(&self, input: CustomStatInput<'_>) -> ChartResult<CustomStatOutput> {
        let p = params(input.parameters)?;
        let bins = p.edges.len() - 1;
        let mut output = CustomStatOutput::default();
        let mut groups = Vec::<(GroupValue, Vec<Vec<RowKey>>)>::new();
        let mut lookup = BTreeMap::new();
        // An empty population has no density (denominator zero), so produce no fake zero bars.
        for source in input.rows {
            let row = input.data.row(source.key).ok_or_else(|| {
                error(
                    DiagnosticCode::RevisionConflict,
                    "Eligible source row is absent.",
                )
            })?;
            let (Some(value), Some(group)) = (input.number(row, &p.input), input.group(row)) else {
                output.invalid_rows.push(source.key);
                continue;
            };
            if value < p.edges[0] || value > p.edges[bins] {
                return Err(error(
                    DiagnosticCode::NumericalDomain,
                    "The custom histogram declares finite outliers as errors.",
                ));
            }
            let i = if let Some(i) = lookup.get(&group) {
                *i
            } else {
                if groups.len() >= input.limits.max_groups
                    || (groups.len() + 1).saturating_mul(bins).saturating_mul(4)
                        > input.limits.max_prepared_rows
                {
                    return Err(error(
                        DiagnosticCode::ResourceLimit,
                        "Grouped histogram output exceeds remaining budget.",
                    ));
                }
                let i = groups.len();
                lookup.insert(group.clone(), i);
                groups.push((group, vec![vec![]; bins]));
                i
            };
            let bin = if value == p.edges[bins] {
                bins - 1
            } else {
                p.edges.partition_point(|x| *x <= value) - 1
            };
            groups[i].1[bin].push(source.key);
        }
        for (group, bins) in groups {
            let total = bins.iter().map(Vec::len).sum::<usize>();
            for (i, mut members) in bins.into_iter().enumerate() {
                members.sort_unstable();
                let count = members.len() as u64;
                let left = p.edges[i];
                let right = p.edges[i + 1];
                let density = (count as f64) / (total as f64) / (right - left);
                let members: Arc<[RowKey]> = members.into();
                let target = Target::Aggregate {
                    id: AggregateId::new(i as u64),
                    group: format!(
                        "{}/{group:?}/{:016x}:{:016x}",
                        input.scope,
                        left.to_bits(),
                        right.to_bits()
                    ),
                    input: input.data.version(),
                    members: members.clone(),
                };
                output.rows.push(StatisticalRow {
                    group: group.clone(),
                    count,
                    values: vec![
                        StatValue {
                            field: field("left"),
                            value: Some(left),
                        },
                        StatValue {
                            field: field("right"),
                            value: Some(right),
                        },
                        StatValue {
                            field: field("density"),
                            value: Some(density),
                        },
                    ],
                    members,
                    target,
                });
            }
        }
        Ok(output)
    }
}
/// One custom geometry with portable and explicitly native-only paint variants.
pub struct HistogramBars {
    /// Require the host-native painter instead of portable polygons.
    pub native: bool,
}
impl CustomGeom for HistogramBars {
    fn descriptor(&self) -> ExtensionDescriptor {
        ExtensionDescriptor::batch(
            if self.native { NATIVE_BARS } else { BARS },
            Revision::new(1),
            !self.native,
        )
    }
    fn validate(&self, layer: &Layer, parameters: &serde_json::Value) -> ChartResult<()> {
        if layer.geom != Geom::Rectangle
            || layer.position != Position::Identity
            || !parameters.is_null()
        {
            return Err(error(
                DiagnosticCode::UnsupportedCapability,
                "This example custom geom requires interval rectangles, identity position and null parameters.",
            ));
        }
        Ok(())
    }
    fn prepare(&self, input: CustomGeomInput<'_>) -> ChartResult<Vec<CustomMark>> {
        let PreparedRows::Statistical(rows) = input.table.rows() else {
            return Err(error(
                DiagnosticCode::SchemaConflict,
                "Custom bars require the histogram's generated schema.",
            ));
        };
        if input.marks.len().saturating_mul(12) > input.max_vertices {
            return Err(error(
                DiagnosticCode::ResourceLimit,
                "Custom bars exceed the remaining paint/hit vertex budget.",
            ));
        }
        let mut marks = vec![];
        for (i, mark) in input.marks.iter().enumerate() {
            let PreparedGeometry::Rectangle { from, to } = mark.geometry else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Custom histogram input is not an interval.",
                ));
            };
            let row = rows
                .iter()
                .find(|r| mark.targets.first() == Some(&r.target))
                .ok_or_else(|| {
                    error(
                        DiagnosticCode::Validation,
                        "Custom mark lacks matching generated semantics.",
                    )
                })?;
            let (x0, x1, y0, y1) = (
                from.x().min(to.x()),
                from.x().max(to.x()),
                from.y().min(to.y()),
                from.y().max(to.y()),
            );
            let (dx, dy) = ((x1 - x0) * 0.08, (y1 - y0) * 0.08);
            let polygon = vec![
                Point::new(x0, y0)?,
                Point::new(x1, y0)?,
                Point::new(x1, y1 - dy)?,
                Point::new(x1 - dx, y1)?,
                Point::new(x0 + dx, y1)?,
                Point::new(x0, y1 - dy)?,
            ];
            let (geometry, hit) = if self.native {
                (
                    PreparedGeometry::NativePaint {
                        from,
                        to,
                        painter: OperationRef::new(PAINTER, Revision::new(1)),
                        parameters: serde_json::json!({"radius":6.}),
                    },
                    HitGeometry::Rectangle { from, to },
                )
            } else {
                (
                    PreparedGeometry::Polygon(polygon.clone()),
                    HitGeometry::Polygon(polygon),
                )
            };
            marks.push(CustomMark {
                input_mark: i,
                geometry,
                interaction: GeometryInteraction {
                    hit,
                    values: vec![
                        ("Count".into(), SemanticValue::Unsigned(row.count)),
                        (
                            "Density".into(),
                            SemanticValue::Number(row.value(&field("density")).ok_or_else(
                                || error(DiagnosticCode::SchemaConflict, "Missing density output."),
                            )?),
                        ),
                        ("Interval".into(), SemanticValue::Text(format!("{x0}–{x1}"))),
                    ],
                    selection: SelectionPolicy::AtomicTarget,
                    keyboard_order: (input.marks.len() - i) as u64,
                },
            });
        }
        Ok(marks)
    }
}
/// Explicit host installation of known code. JSON alone never performs this registration.
pub fn registry() -> ChartResult<Arc<ExtensionRegistry>> {
    let mut registry = ExtensionRegistry::new();
    registry.register_stat(Arc::new(DensityHistogram))?;
    registry.register_geom(Arc::new(HistogramBars { native: false }))?;
    registry.register_geom(Arc::new(HistogramBars { native: true }))?;
    shapes::register(&mut registry)?;
    Ok(Arc::new(registry))
}
/// Portable/native variants share the exact stat population, scale training and guides.
pub fn definition(native: bool) -> ChartDefinition {
    let statistic = Statistic::custom(
        OperationRef::new(HISTOGRAM, Revision::new(1)),
        ExtensionParameters::new(serde_json::json!({"input":{"Field":"1"},"edges":[0.,1.,2.]})),
    );
    let mut aes = StatAes::new(field("left"), StatNumeric::Literal(0.));
    aes.x2 = Some(field("right").into());
    aes.y2 = Some(field("density").into());
    let mut d = ChartDefinition::new(Revision::new(1))
        .transform(TransformDefinition::new(
            TransformId::new(7),
            DatasetId::new(1),
            statistic,
        ))
        .layer(
            Layer::statistical(
                LayerId::new(1),
                TransformId::new(7),
                Statistic::identity(),
                Geom::Rectangle,
                aes,
            )
            .with_geometry_extension(GeometryExtension {
                operation: OperationRef::new(
                    if native { NATIVE_BARS } else { BARS },
                    Revision::new(1),
                ),
                parameters: serde_json::Value::Null,
            }),
        );
    d = d.layer(
        Layer::statistical(
            LayerId::new(2),
            TransformId::new(7),
            Statistic::identity(),
            Geom::Point,
            StatAes::new(field("left"), field("density")),
        )
        .styled(Style {
            color: chart_core::theme::rgb(210, 95, 40),
            radius: 3.,
            ..Style::default()
        }),
    );
    let mut x = AxisSpec::new(ScaleId::new(0), AxisSide::Bottom);
    x.guide_ticks = Some(vec![
        CustomGuideTick {
            value: chart_core::composition::ScaleValue::Number(0.),
            label: "Lower".into(),
        },
        CustomGuideTick {
            value: chart_core::composition::ScaleValue::Number(1.),
            label: "Boundary".into(),
        },
        CustomGuideTick {
            value: chart_core::composition::ScaleValue::Number(2.),
            label: "Upper".into(),
        },
    ]);
    d.axes = vec![x, AxisSpec::new(ScaleId::new(1), AxisSide::Left)];
    d
}
/// Fixture source with exact keys and one null; finite counts are independently `[3,3]`.
pub fn store() -> ChartResult<DataStore> {
    let fields = vec![Field {
        id: FieldId::new(1),
        name: "observation".into(),
        kind: FieldKind::Float64,
        nullable: true,
        unit: Some("units".into()),
        label: None,
    }];
    let schema = Arc::new(Schema::new(SchemaVersion::new(1), fields)?);
    let keys = (1..=7).map(|i| RowKey::new(9007199254743000 + i)).collect();
    let column = Column::new(
        ColumnValues::Float64(vec![0., 0.25, 0.75, 1., 1.5, 2., 999.]),
        vec![true, true, true, true, true, true, false],
        None,
    );
    let batch = NormalizedBatch::new(schema, keys, vec![column], DataLimits::default())?;
    DataStore::new(
        SourceEpoch::new(1),
        vec![(DatasetId::new(1), batch)],
        DataLimits::default(),
    )
}
/// Public-API batch result for use by a native host or standalone downstream application.
pub fn prepare(native: bool) -> ChartResult<PreparedChart> {
    Compiler::with_extensions(registry()?).prepare(
        &definition(native),
        &store()?.snapshot(),
        &ChartState::default(),
        CompileLimits::default(),
    )
}
