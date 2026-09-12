use super::output::{FrameHandle, OptionsHandle, OutputHandle, RequestHandle};
use super::*;
use chart_core::transaction::Transaction;
use chart_export::host::{Editor, Runtime, Updates};
handle!(RuntimeHandle, "_Runtime", Runtime);
#[pymethods]
impl RuntimeHandle {
    #[new]
    fn new(py: Python<'_>, plot: &PlotHandle) -> PyResult<Self> {
        let p = plot.get()?;
        py.detach(|| Runtime::new(p))
            .map(Self::wrap)
            .map_err(failure)
    }
    fn external_view(&self, py: Python<'_>) -> PyResult<Self> {
        let runtime = self.get()?;
        py.detach(|| runtime.external_view())
            .map(Self::wrap)
            .map_err(failure)
    }
    fn accept_from(&mut self, py: Python<'_>, source: &Self) -> PyResult<String> {
        let source = source.get()?;
        let runtime = self.get_mut()?;
        py.detach(|| runtime.accept_from(source)).map_err(failure)
    }
    #[pyo3(signature=(name, input, expected=None))]
    fn command(
        &mut self,
        py: Python<'_>,
        name: &str,
        input: &str,
        expected: Option<u64>,
    ) -> PyResult<String> {
        let c = self.get_mut()?;
        py.detach(|| c.command(name, input, expected))
            .map_err(failure)
    }
    #[pyo3(signature=(name, input, gesture=false, stamp=None))]
    fn named_query(
        &mut self,
        py: Python<'_>,
        name: &str,
        input: &str,
        gesture: bool,
        stamp: Option<&str>,
    ) -> PyResult<String> {
        let stamp = stamp.map(portable::decode).transpose().map_err(failure)?;
        let c = self.get_mut()?;
        py.detach(|| c.named_query(name, input, gesture, stamp))
            .map_err(failure)
    }
    fn editor(&self, component: &ComponentHandle) -> PyResult<EditorHandle> {
        self.get()?
            .owned_editor(component.get()?)
            .map(EditorHandle::wrap)
            .map_err(failure)
    }
    fn revisions(&self) -> PyResult<String> {
        self.get()?.revisions().map_err(failure)
    }
    fn state(&self) -> PyResult<String> {
        self.get()?.state().map_err(failure)
    }
    fn semantics(&mut self, py: Python<'_>) -> PyResult<String> {
        let c = self.get_mut()?;
        py.detach(|| c.semantics()).map_err(failure)
    }
    fn apply_plot(&mut self, py: Python<'_>, plot: &PlotHandle, expected: u64) -> PyResult<bool> {
        let p = plot.get()?;
        let c = self.get_mut()?;
        py.detach(|| c.apply_plot(p, expected)).map_err(failure)
    }
    fn restore_state(&mut self, py: Python<'_>, input: &str, expected: u64) -> PyResult<()> {
        let c = self.get_mut()?;
        py.detach(|| c.restore_state(input, expected))
            .map_err(failure)
    }
    #[pyo3(signature=(input, origin, expected=None))]
    fn act(
        &mut self,
        py: Python<'_>,
        input: &str,
        origin: &str,
        expected: Option<u64>,
    ) -> PyResult<String> {
        let c = self.get_mut()?;
        py.detach(|| c.act(input, origin, expected))
            .map_err(failure)
    }
    #[pyo3(signature=(input, gesture=false, stamp=None))]
    fn query(
        &mut self,
        py: Python<'_>,
        input: &str,
        gesture: bool,
        stamp: Option<&str>,
    ) -> PyResult<String> {
        let stamp = stamp.map(portable::decode).transpose().map_err(failure)?;
        let c = self.get_mut()?;
        py.detach(|| c.query(input, gesture, stamp))
            .map_err(failure)
    }
    fn request(&self, output: &OutputHandle, options: &OptionsHandle) -> PyResult<RequestHandle> {
        self.get()?
            .request(output.get()?, options.get()?)
            .map(RequestHandle::wrap)
            .map_err(failure)
    }
    fn acknowledge_frame(&mut self, frame: &FrameHandle) -> PyResult<()> {
        self.get_mut()?
            .acknowledge_frame(frame.get()?)
            .map_err(failure)
    }
    fn present(
        &mut self,
        py: Python<'_>,
        output: &OutputHandle,
        options: &OptionsHandle,
    ) -> PyResult<FrameHandle> {
        let o = output.get()?;
        let p = options.get()?;
        let c = self.get_mut()?;
        py.detach(|| c.present(o, p))
            .map(FrameHandle::wrap)
            .map_err(failure)
    }
    fn stream(&mut self, options: &ComponentHandle) -> PyResult<()> {
        self.get_mut()?.stream(options.get()?).map_err(failure)
    }
    fn queue_status(&self) -> PyResult<String> {
        self.get()?.queue_status().map_err(failure)
    }
    fn stream_status(&self) -> PyResult<String> {
        self.get()?.stream_status().map_err(failure)
    }
    fn pinned(&self) -> PyResult<String> {
        self.get()?.pinned().map_err(failure)
    }
    fn commit_next(&mut self, py: Python<'_>) -> PyResult<String> {
        let c = self.get_mut()?;
        py.detach(|| c.commit_next()).map_err(failure)
    }
    fn reset_epoch(&mut self, py: Python<'_>) -> PyResult<String> {
        let c = self.get_mut()?;
        py.detach(|| c.reset_epoch()).map_err(failure)
    }
    fn transaction(&self) -> PyResult<UpdatesHandle> {
        self.get()?
            .transaction()
            .map(UpdatesHandle::wrap)
            .map_err(failure)
    }
    fn commit(&mut self, py: Python<'_>, transaction: &TransactionHandle) -> PyResult<String> {
        let t = transaction.get()?;
        let c = self.get_mut()?;
        py.detach(|| c.commit(t)).map_err(failure)
    }
    fn enqueue(&mut self, transaction: &TransactionHandle) -> PyResult<String> {
        self.get_mut()?.enqueue(transaction.get()?).map_err(failure)
    }
    fn dense(
        &self,
        py: Python<'_>,
        frame: &FrameHandle,
        options: &ComponentHandle,
    ) -> PyResult<String> {
        let c = self.get()?;
        let f = frame.get()?;
        let o = options.get()?;
        py.detach(|| c.dense(f, o)).map_err(failure)
    }
    fn link_capture(&mut self, component: &ComponentHandle, event: &str) -> PyResult<String> {
        self.get_mut()?
            .link_capture(component.get()?, event)
            .map_err(failure)
    }
    fn link_resolve(&mut self, component: &ComponentHandle, message: &str) -> PyResult<String> {
        self.get_mut()?
            .link_resolve(component.get()?, message)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        if let Some(mut c) = self.inner.take() {
            c.dispose();
        }
    }
}
handle!(UpdatesHandle, "_Updates", Updates);
#[pymethods]
impl UpdatesHandle {
    fn id(&self, id: &str) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.id(id)))
    }
    fn data(&self, operation: &str, target: &str, data: &DataHandle) -> PyResult<Self> {
        self.get()?
            .data(operation, target.into(), data.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn remove(&self, target: &str, keys: Vec<u64>) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.remove(target.into(), keys)))
    }
    fn retain_count(&self, target: &str, count: Option<usize>) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.retain_count(target.into(), count)))
    }
    fn retention(&self, target: &str, input: &str) -> PyResult<Self> {
        self.get()?
            .retention(target.into(), input)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn retain_event_time(&self, target: &str, field: &str, input: &str) -> PyResult<Self> {
        self.get()?
            .retain_event_time(target.into(), field, input)
            .map(Self::wrap)
            .map_err(failure)
    }
    fn watermark(&self, target: &str, ticks: i64) -> PyResult<Self> {
        Ok(Self::wrap(self.get()?.watermark(target.into(), ticks)))
    }
    fn reset_categories(&self, target: &str, field: &str) -> PyResult<Self> {
        Ok(Self::wrap(
            self.get()?.reset_categories(target.into(), field),
        ))
    }
    fn build(&self) -> PyResult<TransactionHandle> {
        self.get()?
            .build()
            .map(TransactionHandle::wrap)
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(TransactionHandle, "_Transaction", Transaction);
#[pymethods]
impl TransactionHandle {
    fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(EditorHandle, "_Editor", Editor);
#[pymethods]
impl EditorHandle {
    fn original(&self) -> PyResult<String> {
        self.get()?.original().map_err(failure)
    }
    fn preview(&self, py: Python<'_>, dx: f64, dy: f64) -> PyResult<String> {
        let e = self.get()?;
        py.detach(|| e.preview(dx, dy)).map_err(failure)
    }
    fn nudge(
        &self,
        py: Python<'_>,
        horizontal: bool,
        forward: bool,
        steps: u32,
    ) -> PyResult<String> {
        let e = self.get()?;
        py.detach(|| e.nudge(horizontal, forward, steps))
            .map_err(failure)
    }
    fn dispose(&mut self) {
        self.inner.take();
    }
}
