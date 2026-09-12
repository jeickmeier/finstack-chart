//! Deterministic initial publication fixture shared by tests and the headless example.
use chart_core::{data::*, grammar::*, scene::*, services::*, transaction::DataStore, *};
use chart_export::{FontResource, FontResources, PageSize, PublicationProfile};
use std::sync::Arc;
pub const DATA: DatasetId = DatasetId::new(1);
pub const X: FieldId = FieldId::new(1);
pub const Y: FieldId = FieldId::new(2);
pub const FONT_BYTES: &[u8] = include_bytes!("../capability/fonts/NotoSans-Regular.ttf");
pub fn font() -> FontResource {
    FontResource::new(
        ResourceDescriptor {
            id: ResourceId::new(0),
            revision: Revision::INITIAL,
            kind: ResourceKind::Font,
            byte_len: FONT_BYTES.len() as u64,
        },
        Arc::from(FONT_BYTES),
    )
    .unwrap()
}
pub fn fonts() -> FontResources {
    FontResources::new(vec![font()]).unwrap()
}
pub fn batch(rows: &[(u64, f64, Option<f64>)]) -> NormalizedBatch {
    let rows = TypedRows::snapshot(
        DATA,
        Revision::INITIAL,
        rows.iter().map(|r| RowKey::new(r.0)).collect(),
        rows.to_vec(),
        100,
    )
    .unwrap();
    TypedDataBuilder::new(rows.get().unwrap(), SchemaVersion::new(1))
        .float(X, "x", |r| Some(r.1))
        .float(Y, "value", |r| r.2)
        .finish(DataLimits::default())
        .unwrap()
}
pub fn store() -> DataStore {
    DataStore::new(
        SourceEpoch::new(7),
        vec![(
            DATA,
            batch(&[
                (1, 0., Some(1.)),
                (2, 0.5, Some(3.)),
                (3, 1., None),
                (4, 1.5, Some(2.)),
                (5, 2., Some(4.)),
            ]),
        )],
        DataLimits::default(),
    )
    .unwrap()
}
pub fn definition() -> ChartDefinition {
    ChartDefinition::new(Revision::new(8))
        .layer(Layer::new(
            LayerId::new(1),
            DATA,
            Geom::line(),
            SourceAes::new().x(X).y(Y),
        ))
        .layer(Layer::new(
            LayerId::new(2),
            DATA,
            Geom::Point,
            SourceAes::new().x(X).y(Y),
        ))
}
pub fn profile() -> PublicationProfile {
    let mut p = PublicationProfile::new(
        PageSize::millimeters(180., 120.).unwrap(),
        font().descriptor(),
    )
    .unwrap();
    p.layout.padding = 30.;
    p.layout.font_size = 10.;
    p.annotation_revision = Revision::new(4);
    p.annotations.push(SceneItem {
        guide: None,
        layer: None,
        clip: None,
        primitive: Primitive::Text {
            origin: Point::new(30., 18.).unwrap(),
            text: "Captured publication - café Ω".into(),
            font: ResourceId::new(0),
            font_size: 12.,
            color: Color {
                red: 25,
                green: 45,
                blue: 65,
                alpha: 255,
            },
        },
    });
    p
}
