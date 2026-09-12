use super::*;
#[derive(Clone)]
enum Target {
    New(PlotBuilder),
    Edit(PlotEditBuilder),
}
/// A host-owned primary draft containing the actual typed builder, with no stored command AST.
#[derive(Clone)]
pub struct Draft(Target);
impl Draft {
    /// Begin a primary plot around already normalized owned data.
    pub fn new(data: &Data) -> Self {
        Self(Target::New(plot(data.clone())))
    }
    /// Begin a definition-only edit of one immutable primary plot.
    pub fn edit(plot: &Plot) -> Self {
        Self(Target::Edit(plot.edit()))
    }
    /// Append one correctly typed component to the native Rust authoring path.
    pub fn with(&self, slot: &str, component: &Component) -> ChartResult<Self> {
        Ok(Self(match &self.0 {
            Target::New(b) => Target::New(component.attach(slot, b.clone())?),
            Target::Edit(b) => Target::Edit(component.attach_edit(slot, b.clone())?),
        }))
    }
    /// Install an explicitly compiled immutable registry; parameter payloads cannot install code.
    pub fn extensions(&self, registry: Arc<ExtensionRegistry>) -> ChartResult<Self> {
        let Target::New(b) = &self.0 else {
            return Err(unsupported("registry replacement in a definition edit"));
        };
        Ok(Self(Target::New(b.clone().extensions(registry))))
    }
    /// Register another immutable dataset with its existing owner-scoped identity.
    pub fn dataset(&self, data: &Data) -> ChartResult<Self> {
        let Target::New(b) = &self.0 else {
            return Err(unsupported("dataset replacement in a definition edit"));
        };
        Ok(Self(Target::New(b.clone().data(data.clone()))))
    }
    /// Replace/add a named layer without overwriting live data or unrelated components.
    pub fn layer(&self, name: &str, component: &Component) -> ChartResult<Self> {
        let Target::Edit(b) = &self.0 else {
            return Err(unsupported("named edit outside Plot.edit"));
        };
        Ok(Self(Target::Edit(component.edit_layer(name, b.clone())?)))
    }
    /// Configure core-owned authoring budgets/profile or remove an explicit edit component.
    pub fn set(&self, method: &str, arguments: &str) -> ChartResult<Self> {
        let a = Args::parse(arguments)?;
        Ok(Self(match &self.0 {
            Target::New(b) => Target::New(match method {
                "profile" => b.clone().profile(a.one()?),
                "compile_limits" => b.clone().compile_limits(options(a.one()?)?),
                "data_limits" => b.clone().data_limits(options(a.one()?)?),
                _ => return Err(unsupported(method)),
            }),
            Target::Edit(b) => Target::Edit(match method {
                "profile" => b.clone().profile(a.one()?),
                "remove_layer" => b.clone().remove_layer(&a.string()?),
                "remove_annotation" => b.clone().remove_annotation(&a.string()?),
                "clear_facets" => {
                    a.count(0)?;
                    b.clone().clear_facets()
                }
                _ => return Err(unsupported(method)),
            }),
        }))
    }
    /// Build structurally through Rust, preserving component handles and no-op edit revisions.
    pub fn build(&self) -> ChartResult<Plot> {
        let plot = match &self.0 {
            Target::New(b) => b.clone().build(),
            Target::Edit(b) => b.clone().build(),
        }?;
        plot.extensions()
            .validate_portable_hierarchies(plot.definition())?;
        Ok(plot)
    }
}
