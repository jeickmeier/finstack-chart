use super::*;
use chart_core::plot::{CutOptions, CutResult, CutSpec, ResolutionOptions};
handle!(_Cut, CutResult);
fn nullable(values: Vec<f64>, valid: Vec<u8>) -> Result<Vec<Option<f64>>, JsError> {
    if values.len() != valid.len() || valid.iter().any(|v| *v > 1) {
        return Err(failure(chart_core::Diagnostic::error(
            chart_core::DiagnosticCode::NumericalDomain,
            "Vector validity must contain one boolean per value.",
            "Provide matching values and validity.",
        )));
    }
    Ok(values
        .into_iter()
        .zip(valid)
        .map(|(x, v)| (v == 1).then_some(x))
        .collect())
}
#[wasm_bindgen]
impl _Cut {
    pub fn create(
        values: Vec<f64>,
        valid: Vec<u8>,
        spec: &str,
        options: &str,
    ) -> Result<Self, JsError> {
        plot::cut(
            &nullable(values, valid)?,
            portable::decode::<CutSpec>(spec).map_err(failure)?,
            portable::decode::<CutOptions>(options).map_err(failure)?,
        )
        .map(Self::wrap)
        .map_err(failure)
    }
    pub fn resolution(values: Vec<f64>, valid: Vec<u8>, options: &str) -> Result<f64, JsError> {
        plot::resolution(
            &nullable(values, valid)?,
            portable::decode::<ResolutionOptions>(options).map_err(failure)?,
        )
        .map_err(failure)
    }
    pub fn summarize(values: Vec<f64>, valid: Vec<u8>, helper: &str) -> Result<String, JsError> {
        let value = plot::summarize(
            &nullable(values, valid)?,
            &portable::decode::<chart_core::grammar::SummaryHelper>(helper).map_err(failure)?,
        )
        .map_err(failure)?;
        portable::encode(&value).map_err(failure)
    }
    pub fn to_json(&self) -> Result<String, JsError> {
        portable::encode(self.get()?).map_err(failure)
    }
    pub fn column(&self) -> Result<data::_Column, JsError> {
        Ok(data::_Column::wrap(self.get()?.column()))
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
