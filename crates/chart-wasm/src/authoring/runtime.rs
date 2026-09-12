use super::output::{_ExportOptions, _FigureRequest, _FigureSnapshot, _Output};
use super::*;
use chart_core::transaction::Transaction;
use chart_export::host::{Editor, Runtime, Updates};
handle!(_Runtime, Runtime);
#[wasm_bindgen]
impl _Runtime {
    #[wasm_bindgen(constructor)]
    pub fn new(plot: &_Plot) -> Result<Self, JsError> {
        Runtime::new(plot.get()?).map(Self::wrap).map_err(failure)
    }
    pub fn external_view(&self) -> Result<Self, JsError> {
        self.get()?.external_view().map(Self::wrap).map_err(failure)
    }
    pub fn accept_from(&mut self, source: &Self) -> Result<String, JsError> {
        self.get_mut()?.accept_from(source.get()?).map_err(failure)
    }
    pub fn command(
        &mut self,
        name: &str,
        input: &str,
        expected: Option<u64>,
    ) -> Result<String, JsError> {
        self.get_mut()?
            .command(name, input, expected)
            .map_err(failure)
    }
    pub fn named_query(
        &mut self,
        name: &str,
        input: &str,
        gesture: bool,
        stamp: Option<String>,
    ) -> Result<String, JsError> {
        let stamp = stamp
            .as_deref()
            .map(portable::decode)
            .transpose()
            .map_err(failure)?;
        self.get_mut()?
            .named_query(name, input, gesture, stamp)
            .map_err(failure)
    }
    pub fn editor(&self, component: &_Component) -> Result<_Editor, JsError> {
        self.get()?
            .owned_editor(component.get()?)
            .map(_Editor::wrap)
            .map_err(failure)
    }
    pub fn revisions(&self) -> Result<String, JsError> {
        self.get()?.revisions().map_err(failure)
    }
    pub fn state(&self) -> Result<String, JsError> {
        self.get()?.state().map_err(failure)
    }
    pub fn semantics(&mut self) -> Result<String, JsError> {
        self.get_mut()?.semantics().map_err(failure)
    }
    pub fn apply_plot(&mut self, plot: &_Plot, expected: u64) -> Result<bool, JsError> {
        let p = plot.get()?;
        self.get_mut()?.apply_plot(p, expected).map_err(failure)
    }
    pub fn restore_state(&mut self, input: &str, expected: u64) -> Result<(), JsError> {
        self.get_mut()?
            .restore_state(input, expected)
            .map_err(failure)
    }
    pub fn act(
        &mut self,
        input: &str,
        origin: &str,
        expected: Option<u64>,
    ) -> Result<String, JsError> {
        self.get_mut()?
            .act(input, origin, expected)
            .map_err(failure)
    }
    pub fn query(
        &mut self,
        input: &str,
        gesture: bool,
        stamp: Option<String>,
    ) -> Result<String, JsError> {
        let stamp = stamp
            .as_deref()
            .map(portable::decode)
            .transpose()
            .map_err(failure)?;
        self.get_mut()?
            .query(input, gesture, stamp)
            .map_err(failure)
    }
    pub fn request(
        &self,
        output: &_Output,
        options: &_ExportOptions,
    ) -> Result<_FigureRequest, JsError> {
        self.get()?
            .request(output.get()?, options.get()?)
            .map(_FigureRequest::wrap)
            .map_err(failure)
    }
    pub fn acknowledge_frame(&mut self, frame: &_FigureSnapshot) -> Result<(), JsError> {
        self.get_mut()?
            .acknowledge_frame(frame.get()?)
            .map_err(failure)
    }
    pub fn present(
        &mut self,
        output: &_Output,
        options: &_ExportOptions,
    ) -> Result<_FigureSnapshot, JsError> {
        let o = output.get()?;
        let p = options.get()?;
        self.get_mut()?
            .present(o, p)
            .map(_FigureSnapshot::wrap)
            .map_err(failure)
    }
    pub fn stream(&mut self, options: &_Component) -> Result<(), JsError> {
        self.get_mut()?.stream(options.get()?).map_err(failure)
    }
    pub fn queue_status(&self) -> Result<String, JsError> {
        self.get()?.queue_status().map_err(failure)
    }
    pub fn stream_status(&self) -> Result<String, JsError> {
        self.get()?.stream_status().map_err(failure)
    }
    pub fn pinned(&self) -> Result<String, JsError> {
        self.get()?.pinned().map_err(failure)
    }
    pub fn commit_next(&mut self) -> Result<String, JsError> {
        self.get_mut()?.commit_next().map_err(failure)
    }
    pub fn reset_epoch(&mut self) -> Result<String, JsError> {
        self.get_mut()?.reset_epoch().map_err(failure)
    }
    pub fn transaction(&self) -> Result<_Updates, JsError> {
        self.get()?
            .transaction()
            .map(_Updates::wrap)
            .map_err(failure)
    }
    pub fn commit(&mut self, transaction: &_Transaction) -> Result<String, JsError> {
        let t = transaction.get()?;
        self.get_mut()?.commit(t).map_err(failure)
    }
    pub fn enqueue(&mut self, transaction: &_Transaction) -> Result<String, JsError> {
        self.get_mut()?.enqueue(transaction.get()?).map_err(failure)
    }
    pub fn dense(&self, frame: &_FigureSnapshot, options: &_Component) -> Result<String, JsError> {
        self.get()?
            .dense(frame.get()?, options.get()?)
            .map_err(failure)
    }
    pub fn link_capture(&mut self, component: &_Component, event: &str) -> Result<String, JsError> {
        self.get_mut()?
            .link_capture(component.get()?, event)
            .map_err(failure)
    }
    pub fn link_resolve(
        &mut self,
        component: &_Component,
        message: &str,
    ) -> Result<String, JsError> {
        self.get_mut()?
            .link_resolve(component.get()?, message)
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        if let Some(mut c) = self.inner.take() {
            c.dispose();
        }
    }
}
handle!(_Updates, Updates);
#[wasm_bindgen]
impl _Updates {
    pub fn id(&self, id: &str) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.id(id)))
    }
    pub fn data(&self, operation: &str, target: &str, data: &_Data) -> Result<Self, JsError> {
        self.get()?
            .data(operation, target.into(), data.get()?)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn remove(&self, target: &str, keys: Vec<u64>) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.remove(target.into(), keys)))
    }
    pub fn retain_count(&self, target: &str, count: Option<usize>) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.retain_count(target.into(), count)))
    }
    pub fn retention(&self, target: &str, input: &str) -> Result<Self, JsError> {
        self.get()?
            .retention(target.into(), input)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn retain_event_time(
        &self,
        target: &str,
        field: &str,
        input: &str,
    ) -> Result<Self, JsError> {
        self.get()?
            .retain_event_time(target.into(), field, input)
            .map(Self::wrap)
            .map_err(failure)
    }
    pub fn watermark(&self, target: &str, ticks: i64) -> Result<Self, JsError> {
        Ok(Self::wrap(self.get()?.watermark(target.into(), ticks)))
    }
    pub fn reset_categories(&self, target: &str, field: &str) -> Result<Self, JsError> {
        Ok(Self::wrap(
            self.get()?.reset_categories(target.into(), field),
        ))
    }
    pub fn build(&self) -> Result<_Transaction, JsError> {
        self.get()?.build().map(_Transaction::wrap).map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
handle!(_Transaction, Transaction);
#[wasm_bindgen]
impl _Transaction {
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}

handle!(_Editor, Editor);
#[wasm_bindgen]
impl _Editor {
    pub fn original(&self) -> Result<String, JsError> {
        self.get()?.original().map_err(failure)
    }
    pub fn preview(&self, dx: f64, dy: f64) -> Result<String, JsError> {
        self.get()?.preview(dx, dy).map_err(failure)
    }
    pub fn nudge(&self, horizontal: bool, forward: bool, steps: u32) -> Result<String, JsError> {
        self.get()?
            .nudge(horizontal, forward, steps)
            .map_err(failure)
    }
    pub fn dispose(&mut self) {
        self.inner.take();
    }
}
