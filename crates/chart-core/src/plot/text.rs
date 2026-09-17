use crate::ChartResult;
use crate::services::ResourceDescriptor;
use crate::typography::{RichRun, RichText, TextDirection};

/// Reusable typography options shared by titles, annotations, axes and legends.
#[derive(Clone, Debug)]
pub struct TextStyle {
    pub(super) run: RichRun,
}
/// Default regular typography in the destination's supplied font.
pub fn text_style() -> TextStyle {
    TextStyle {
        run: RichRun::new(""),
    }
}
impl TextStyle {
    /// Parse replacement labels through this mathematical typography template.
    pub fn math(mut self, fonts: crate::typography::MathFonts) -> ChartResult<Self> {
        self.run.math = Some(crate::typography::MathExpression::parse(
            "x",
            fonts,
            crate::Limits::default(),
        )?);
        self.run.text = "x".into();
        Ok(self)
    }

    /// Relative size multiplier on the destination label size.
    pub fn size(mut self, size: f64) -> Self {
        self.run.size = size;
        self
    }
    /// Requested weight, which must match the supplied face.
    pub fn weight(mut self, weight: u16) -> Self {
        self.run.weight = weight;
        self
    }
    /// Explicit primary face identity, resolved by the destination.
    pub fn font(mut self, font: ResourceDescriptor) -> Self {
        self.run.font = Some(font);
        self
    }
    /// Append a deliberate fallback face.
    pub fn fallback(mut self, font: ResourceDescriptor) -> Self {
        self.run.fallback.push(font);
        self
    }
    /// Override inherited text color.
    pub fn color(mut self, color: impl Into<crate::color::Paint>) -> Self {
        self.run.color = Some(color.into());
        self
    }
    /// Explicit shaping language tag.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.run.language = language.into();
        self
    }
    /// Explicit shaping direction.
    pub fn direction(mut self, direction: TextDirection) -> Self {
        self.run.direction = direction;
        self
    }
    /// Request OpenType tabular numerals where available.
    pub fn tabular(mut self, enabled: bool) -> Self {
        self.run.tabular = enabled;
        self
    }
    pub(super) fn apply(&self, text: &mut RichText) {
        for line in &mut text.lines {
            for run in line {
                let value = std::mem::take(&mut run.text);
                let math = run.math.take();
                *run = self.run.clone();
                run.math = math;
                run.text = value;
            }
        }
    }
}
pub(super) fn plain(text: impl Into<String>) -> RichText {
    RichText {
        lines: text
            .into()
            .split('\n')
            .map(|line| vec![RichRun::new(line)])
            .collect(),
        line_spacing: 1.2,
        rotation: 0.,
    }
}
/// An independently styled run within a rich text block.
#[derive(Clone, Debug)]
pub struct TextRunBuilder(RichRun);
/// Start one logical run; line breaks belong in the block's line structure.
pub fn text_run(text: impl Into<String>) -> TextRunBuilder {
    TextRunBuilder(RichRun::new(text))
}
impl TextRunBuilder {
    /// Apply shared typography while retaining the logical text.
    pub fn style(mut self, style: TextStyle) -> Self {
        let text = self.0.text;
        let math = self.0.math;
        self.0 = style.run;
        self.0.math = math;
        self.0.text = text;
        self
    }
}
/// Logical rich text with explicit lines and shared destination measurement.
#[derive(Clone, Debug)]
pub struct RichTextBuilder(RichText);
/// Start a rich block, supporting newlines without embedding controls in a run.
pub fn rich_text(text: impl Into<String>) -> RichTextBuilder {
    RichTextBuilder(plain(text))
}
/// Parse one mathematical text block using explicit supplied font resources.
pub fn math_text(
    source: impl Into<String>,
    fonts: crate::typography::MathFonts,
) -> ChartResult<RichTextBuilder> {
    Ok(RichTextBuilder(RichText::math(source, fonts)?))
}
impl RichTextBuilder {
    /// Parse each current logical run without executing expressions.
    pub fn math(mut self, fonts: crate::typography::MathFonts) -> ChartResult<Self> {
        for run in self.0.lines.iter_mut().flatten() {
            run.math = Some(crate::typography::MathExpression::parse(
                &run.text,
                fonts.clone(),
                crate::Limits::default(),
            )?);
        }
        Ok(self)
    }

    /// Append a styled run to the last line.
    pub fn run(mut self, run: TextRunBuilder) -> Self {
        self.0.lines.last_mut().expect("initial line").push(run.0);
        self
    }
    /// Append a line made of explicitly styled runs.
    pub fn line(mut self, runs: impl IntoIterator<Item = TextRunBuilder>) -> Self {
        self.0.lines.push(runs.into_iter().map(|r| r.0).collect());
        self
    }
    /// Apply shared typography to all current runs.
    pub fn style(mut self, style: TextStyle) -> Self {
        style.apply(&mut self.0);
        self
    }
    /// Set baseline spacing multiplier.
    pub fn line_spacing(mut self, spacing: f64) -> Self {
        self.0.line_spacing = spacing;
        self
    }
    /// Clockwise rotation in degrees.
    pub fn rotation(mut self, degrees: f64) -> Self {
        self.0.rotation = degrees;
        self
    }
}
impl From<RichTextBuilder> for RichText {
    fn from(value: RichTextBuilder) -> Self {
        value.0
    }
}
macro_rules! components {
    ($($name:ident, $function:ident, $doc:literal);* $(;)?) => {$(
        #[doc = $doc]
        #[derive(Clone, Debug)]
        pub struct $name { pub(super) text: RichText }
        #[doc = $doc]
        pub fn $function(text: impl Into<String>) -> $name { $name { text: plain(text) } }
        impl $name {
            /// Set independently styled multiline content.
            pub fn rich(mut self, text: impl Into<RichText>) -> Self { self.text = text.into(); self }
            /// Apply reusable typography options.
            pub fn style(mut self, style: TextStyle) -> Self { style.apply(&mut self.text); self }
            /// Set clockwise block rotation.
            pub fn rotation(mut self, degrees: f64) -> Self { self.text.rotation = degrees; self }
            /// Set line baseline spacing.
            pub fn line_spacing(mut self, spacing: f64) -> Self { self.text.line_spacing = spacing; self }
        }
    )*};
}
components!(TitleBuilder, title, "Plot title component, separate from x/y annotations.";
SubtitleBuilder, subtitle, "Plot subtitle component.";
CaptionBuilder, caption, "Caption placed below panels.";
SourceNoteBuilder, source_note, "Ordered figure source note.";
FootnoteBuilder, footnote, "Ordered figure footnote.");
