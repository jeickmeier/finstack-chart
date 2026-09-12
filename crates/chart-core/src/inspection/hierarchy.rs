//! Structural values come from the exact prepared hierarchy behind each painted path.
use super::*;
use crate::{
    grammar::{PanelKey, SemanticValue},
    state::TargetIdentity,
};
use std::collections::BTreeMap;
type Key = (Option<PanelKey>, LayerId, TargetIdentity);
pub(super) struct HierarchyValues(BTreeMap<Key, Vec<(String, SemanticValue)>>);
impl HierarchyValues {
    pub(super) fn new(chart: &LaidOutChart) -> ChartResult<Self> {
        fn visit(
            chart: &LaidOutChart,
            panel: Option<&PanelKey>,
            out: &mut HierarchyValues,
        ) -> ChartResult<()> {
            for (layer, resolved) in chart.hierarchies() {
                let prepared = resolved.prepared();
                for node in prepared.hierarchy().iter()? {
                    let target = prepared.target(node.handle())?;
                    let Target::HierarchyNode { membership, .. } = target else {
                        continue;
                    };
                    let mut values = vec![
                        (
                            "label".into(),
                            SemanticValue::Text(prepared.label(node.handle())?.into()),
                        ),
                        ("depth".into(), SemanticValue::Unsigned(node.depth() as u64)),
                        (
                            "height".into(),
                            SemanticValue::Unsigned(node.height() as u64),
                        ),
                        (
                            "source_members".into(),
                            SemanticValue::Unsigned(membership.members().len() as u64),
                        ),
                    ];
                    if let Some(value) = node.value() {
                        values.push(("value".into(), SemanticValue::Number(value)));
                    }
                    out.0.insert(
                        (panel.cloned(), *layer, TargetIdentity::from(target)),
                        values,
                    );
                }
            }
            for p in chart.panels() {
                visit(&p.chart, Some(&p.key), out)?;
            }
            for p in chart.insets() {
                visit(&p.chart, p.panel.as_ref().or(panel), out)?;
            }
            Ok(())
        }
        let mut out = Self(BTreeMap::new());
        visit(chart, None, &mut out)?;
        Ok(out)
    }
    pub(super) fn get(
        &self,
        panel: &Option<PanelKey>,
        layer: LayerId,
        target: &Target,
    ) -> Vec<(String, SemanticValue)> {
        self.0
            .get(&(panel.clone(), layer, TargetIdentity::from(target)))
            .cloned()
            .unwrap_or_default()
    }
}
