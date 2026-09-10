//! Exact bigint time handles forwarding all calendar semantics to shared Rust.
use super::{disposed, failure, handle};
use chart_core::{
    portable,
    scales::{CalendarInterval, CalendarTicks, TimeFormat, TimeScale, TimeScaleSpec},
};
use wasm_bindgen::prelude::*;
handle!(_TimeScale, TimeScale);
#[wasm_bindgen]
impl _TimeScale {
    #[wasm_bindgen(constructor)]
    pub fn new(spec: &str) -> Result<Self, JsError> {
        TimeScale::new(portable::decode::<TimeScaleSpec>(spec).map_err(failure)?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn from_json(input: &str) -> Result<Self, JsError> {
        TimeScale::from_json(input).map(Self::wrap).map_err(failure)
    }
    pub fn to_json(&self) -> Result<String, JsError> {
        self.get()?.to_json().map_err(failure)
    }
    pub fn copy(&self) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.clone()))
    }
    pub fn map_json(&self, value: Option<i64>) -> Result<String, JsError> {
        self.get()?
            .map(value)
            .and_then(|v| v.to_json())
            .map_err(failure)
    }
    pub fn invert(&self, position: f64) -> Result<i64, JsError> {
        self.get()?.invert(position).map_err(failure)
    }
    pub fn ticks(&self, selection: &str, budget: usize) -> Result<Vec<i64>, JsError> {
        self.get()?
            .ticks(
                portable::decode::<CalendarTicks>(selection).map_err(failure)?,
                budget,
            )
            .map_err(failure)
    }
    pub fn nice(&self, selection: &str) -> Result<Self, JsError> {
        self.get()?
            .nice(portable::decode::<CalendarTicks>(selection).map_err(failure)?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn format(&self, value: i64, format: &str) -> Result<String, JsError> {
        let s = self.get()?;
        s.tick_format(portable::decode::<TimeFormat>(format).map_err(failure)?)
            .and_then(|f| f.format(value, s.spec().unit))
            .map_err(failure)
    }
    pub fn floor(&self, value: i64, interval: &str) -> Result<i64, JsError> {
        let s = self.get()?;
        s.calendar()
            .floor(
                value,
                s.spec().unit,
                portable::decode::<CalendarInterval>(interval).map_err(failure)?,
            )
            .map_err(failure)
    }
    pub fn ceil(&self, value: i64, interval: &str) -> Result<i64, JsError> {
        let s = self.get()?;
        s.calendar()
            .ceil(
                value,
                s.spec().unit,
                portable::decode::<CalendarInterval>(interval).map_err(failure)?,
            )
            .map_err(failure)
    }
    pub fn round(&self, value: i64, interval: &str) -> Result<i64, JsError> {
        let s = self.get()?;
        s.calendar()
            .round(
                value,
                s.spec().unit,
                portable::decode::<CalendarInterval>(interval).map_err(failure)?,
            )
            .map_err(failure)
    }
    pub fn offset(&self, value: i64, interval: &str, steps: f64) -> Result<i64, JsError> {
        let s = self.get()?;
        s.calendar()
            .offset(
                value,
                s.spec().unit,
                portable::decode::<CalendarInterval>(interval).map_err(failure)?,
                steps,
            )
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
