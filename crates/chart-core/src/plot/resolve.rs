use super::*;
pub(super) struct LayerContext<'a> {
    pub axes: &'a BTreeMap<String, ScaleId>,
    pub color_ids: &'a mut BTreeMap<String, ScaleId>,
    pub color_scales: &'a BTreeMap<String, ColorScale>,
}
impl LayerContext<'_> {
    pub fn apply(
        &mut self,
        definition: &mut ChartDefinition,
        layer: &mut crate::grammar::Layer,
        builder: &LayerBuilder,
        mapping: &AesBuilder,
        data: &Data,
    ) -> ChartResult<()> {
        if let Some((x, y)) = &builder.axes {
            layer.scales = crate::grammar::ScaleBindings {
                x: *self.axes.get(x).ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        format!("No axis named '{x}'."),
                    )
                })?,
                y: *self.axes.get(y).ok_or_else(|| {
                    error(
                        DiagnosticCode::MissingResource,
                        format!("No axis named '{y}'."),
                    )
                })?,
            };
        }
        if builder.theme.patch != crate::theme::ThemePatch::default() {
            let theme = definition.theme.get_or_insert_with(|| theme().spec);
            theme
                .layers
                .entry(layer.id)
                .or_default()
                .overlay(&builder.theme.patch);
        }
        if let Some(input) = &builder.generated_color {
            let name = builder.generated_color_scale.as_ref().ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    "Generated color requires an explicit color_scale name.",
                )
            })?;
            let scale = self.color_scales.get(name).ok_or_else(|| {
                error(
                    DiagnosticCode::MissingResource,
                    format!("No color scale named '{name}'."),
                )
            })?;
            layer.color = Some(ColorEncoding {
                id: self.color_ids[name],
                title: Some(name.clone()),
                input: input.clone(),
                scale: scale.clone(),
            });
        } else if let Some(color) = &mapping.color {
            let field = color.field(data)?;
            let (_, column) =
                data.batch.schema().field(field).ok_or_else(|| {
                    error(DiagnosticCode::MissingResource, "Color field is absent.")
                })?;
            let scale_name = mapping.color_scale.as_deref().unwrap_or(&column.name);
            let explicit = self.color_scales.get(scale_name);
            if mapping.color_scale.is_some() && explicit.is_none() {
                return Err(error(
                    DiagnosticCode::MissingResource,
                    format!("No color scale named '{scale_name}'."),
                ));
            }
            if explicit.is_none()
                && matches!(
                    column.kind,
                    crate::data::FieldKind::Float64 | crate::data::FieldKind::Timestamp(_)
                )
            {
                return Err(error(
                    DiagnosticCode::UnsupportedCapability,
                    "Numeric color requires an explicit continuous color scale.",
                ));
            }
            let id = if let Some(id) = self.color_ids.get(scale_name) {
                *id
            } else {
                let id = ScaleId::new(fresh_id()?);
                self.color_ids.insert(scale_name.to_owned(), id);
                id
            };
            let scale = explicit.cloned().unwrap_or_else(default_color_scale);
            let input = if matches!(layer.mappings, Mappings::Source(_)) {
                if matches!(scale, ColorScale::Continuous { .. }) {
                    ColorInput::Numeric(color.resolve(data)?)
                } else {
                    ColorInput::Category(field)
                }
            } else if generated_group(definition, layer)
                == Some(crate::grammar::Grouping::Field(field))
            {
                ColorInput::Group
            } else {
                return Err(error(
                    DiagnosticCode::SchemaConflict,
                    "Generated layers must map color to their group or an explicit generated field.",
                ));
            };
            layer.color = Some(ColorEncoding {
                id,
                title: Some(column.name.clone()),
                input,
                scale,
            });
        }
        Ok(())
    }
}
