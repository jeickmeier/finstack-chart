use super::*;
use chart_export::{
    ExportJob, ExportLimits, ExportQueue, FigureRequest, FigureSnapshot, Output,
    host::{self, Options},
};
handle!(OptionsHandle, "_ExportOptions", Options);
#[pymethods]
impl OptionsHandle {
    #[new]
    #[pyo3(signature=(width, height, unit="pt"))]
    fn new(width: f64, height: f64, unit: &str) -> PyResult<Self> {
        Options::new(width, height, unit)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn set(&self, name: &str, arguments: &str) -> PyResult<Self> {
        self.get()?
            .set(name, arguments)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn layout(&self, component: &ComponentHandle) -> PyResult<Self> {
        self.get()?
            .layout(component.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(OutputHandle, "_Output", Output);
#[pymethods]
impl OutputHandle {
    #[new]
    fn new(py: Python<'_>, bytes: Vec<u8>) -> PyResult<Self> {
        py.detach(|| Output::new(bytes))
            .map(Self::wrap)
            .map_err(failure)
    }
    fn request(&self, plot: &PlotHandle, options: &OptionsHandle) -> PyResult<RequestHandle> {
        options
            .get()?
            .request(self.get()?, plot.get()?)
            .map(RequestHandle::wrap)
            .map_err(failure)
    }
    fn primary_font(&self) -> PyResult<String> {
        portable::encode(&self.get()?.primary_font()).map_err(failure)
    }
    fn register_font(&mut self, py: Python<'_>, bytes: Vec<u8>) -> PyResult<String> {
        let output = self.get_mut()?;
        py.detach(|| {
            output
                .register_font(bytes)
                .and_then(|d| portable::encode(&d))
        })
        .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(RequestHandle, "_FigureRequest", FigureRequest);
#[pymethods]
impl RequestHandle {
    fn prepare(&self, py: Python<'_>) -> PyResult<FrameHandle> {
        let r = self.get()?;
        py.detach(|| r.prepare())
            .map(FrameHandle::wrap)
            .map_err(failure)
    }
    fn manifest(&self, py: Python<'_>) -> PyResult<String> {
        let r = self.get()?;
        py.detach(|| r.manifest().and_then(|v| portable::encode(&v)))
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(FrameHandle, "_FigureSnapshot", FigureSnapshot);
#[pymethods]
impl FrameHandle {
    fn guides(&self, py: Python<'_>) -> PyResult<String> {
        let f = self.get()?;
        py.detach(|| f.guides_json()).map_err(failure)
    }
    fn scene(&self, py: Python<'_>) -> PyResult<String> {
        let f = self.get()?;
        py.detach(|| f.scene_json()).map_err(failure)
    }
    fn manifest(&self) -> PyResult<String> {
        portable::encode(&self.get()?.metadata().manifest()).map_err(failure)
    }
    fn export(&self, py: Python<'_>, format: &str) -> PyResult<Vec<u8>> {
        let f = self.get()?;
        let format = host::format(format).map_err(failure)?;
        Ok(py.detach(|| f.export(format)).map_err(failure)?.bytes)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(QueueHandle, "_ExportQueue", ExportQueue);
#[pymethods]
impl QueueHandle {
    #[new]
    #[pyo3(signature=(max_jobs=2, max_input_bytes=536870912, max_rows=2000000))]
    fn new(max_jobs: usize, max_input_bytes: usize, max_rows: usize) -> PyResult<Self> {
        ExportQueue::new(ExportLimits {
            max_jobs,
            max_input_bytes,
            max_rows,
        })
        .map(Self::wrap)
        .map_err(failure)
    }
    fn submit(&self, request: &RequestHandle, format: &str) -> PyResult<JobHandle> {
        self.get()?
            .submit(
                request.get()?.clone(),
                host::format(format).map_err(failure)?,
            )
            .map(JobHandle::wrap)
            .map_err(failure)
    }
    fn metrics(&self) -> PyResult<String> {
        portable::encode(&self.get()?.metrics()).map_err(failure)
    }
    fn dispose(&mut self) {
        if let Some(q) = self.inner.take() {
            q.dispose();
        }
    }
}
handle!(JobHandle, "_ExportJob", ExportJob);
#[pymethods]
impl JobHandle {
    fn cancel(&self) -> PyResult<bool> {
        Ok(self.get()?.cancellation().cancel())
    }
    fn run(&mut self, py: Python<'_>) -> PyResult<Vec<u8>> {
        let job = self.inner.take().ok_or_else(disposed)?;
        Ok(py.detach(|| job.run()).map_err(failure)?.bytes)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
