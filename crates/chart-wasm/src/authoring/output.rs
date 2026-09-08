use super::*;
use chart_export::{
    ExportJob, ExportLimits, ExportQueue, FigureRequest, FigureSnapshot, Output,
    host::{self, Options},
};

handle!(_ExportOptions, _ExportOptions, Options);
#[wasm_bindgen]
impl _ExportOptions {
    #[wasm_bindgen(constructor)]
    pub fn new(width: f64, height: f64, unit: &str) -> Result<Self, JsError> {
        Options::new(width, height, unit)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn set(&self, name: &str, arguments: &str) -> Result<Self, JsError> {
        self.get()?
            .set(name, arguments)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn layout(&self, component: &_Component) -> Result<Self, JsError> {
        self.get()?
            .layout(component.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_Output, _Output, Output);
#[wasm_bindgen]
impl _Output {
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: Vec<u8>) -> Result<Self, JsError> {
        (Output::new(bytes)).map(Self::wrap).map_err(failure)
    }
    pub fn request(
        &self,
        plot: &_Plot,
        options: &_ExportOptions,
    ) -> Result<_FigureRequest, JsError> {
        options
            .get()?
            .request(self.get()?, plot.get()?)
            .map(_FigureRequest::wrap)
            .map_err(failure)
    }
    pub fn primary_font(&self) -> Result<String, JsError> {
        portable::encode(&self.get()?.primary_font()).map_err(failure)
    }
    pub fn register_font(&mut self, bytes: Vec<u8>) -> Result<String, JsError> {
        let output = self.inner.as_mut().ok_or_else(disposed)?;
        (output
            .register_font(bytes)
            .and_then(|d| portable::encode(&d)))
        .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_FigureRequest, _FigureRequest, FigureRequest);
#[wasm_bindgen]
impl _FigureRequest {
    pub fn prepare(&self) -> Result<_FigureSnapshot, JsError> {
        let r = self.get()?;
        (r.prepare()).map(_FigureSnapshot::wrap).map_err(failure)
    }
    pub fn manifest(&self) -> Result<String, JsError> {
        let r = self.get()?;
        (r.manifest().and_then(|v| portable::encode(&v))).map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_FigureSnapshot, _FigureSnapshot, FigureSnapshot);
#[wasm_bindgen]
impl _FigureSnapshot {
    pub fn scene(&self) -> Result<String, JsError> {
        let f = self.get()?;
        (f.scene_json()).map_err(failure)
    }
    pub fn manifest(&self) -> Result<String, JsError> {
        portable::encode(&self.get()?.metadata().manifest()).map_err(failure)
    }
    pub fn export(&self, format: &str) -> Result<Vec<u8>, JsError> {
        let f = self.get()?;
        let format = host::format(format).map_err(failure)?;
        let bytes = (f.export(format)).map_err(failure)?.bytes;
        Ok(bytes)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_ExportQueue, _ExportQueue, ExportQueue);
#[wasm_bindgen]
impl _ExportQueue {
    #[wasm_bindgen(constructor)]
    pub fn new(max_jobs: usize, max_input_bytes: usize, max_rows: usize) -> Result<Self, JsError> {
        ExportQueue::new(ExportLimits {
            max_jobs,
            max_input_bytes,
            max_rows,
        })
        .map(Self::wrap)
        .map_err(failure)
    }
    pub fn submit(&self, request: &_FigureRequest, format: &str) -> Result<_ExportJob, JsError> {
        self.get()?
            .submit(
                request.get()?.clone(),
                host::format(format).map_err(failure)?,
            )
            .map(_ExportJob::wrap)
            .map_err(failure)
    }
    pub fn metrics(&self) -> Result<String, JsError> {
        portable::encode(&self.get()?.metrics()).map_err(failure)
    }
    pub fn dispose(&mut self) {
        if let Some(q) = self.inner.take() {
            q.dispose();
        }
    }
}
handle!(_ExportJob, _ExportJob, ExportJob);
#[wasm_bindgen]
impl _ExportJob {
    pub fn cancel(&self) -> Result<bool, JsError> {
        Ok(self.get()?.cancellation().cancel())
    }
    pub fn run(&mut self) -> Result<Vec<u8>, JsError> {
        let job = self.inner.take().ok_or_else(disposed)?;
        let bytes = (job.run()).map_err(failure)?.bytes;
        Ok(bytes)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
