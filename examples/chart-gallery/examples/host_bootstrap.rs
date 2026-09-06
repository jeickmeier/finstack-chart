//! WP-01 dependency/link proof. This is a host window, not a chart or renderer fixture.

use gpui::{AppContext, Context, IntoElement, ParentElement, Render, Window, WindowOptions, div};

struct HostBootstrap;

impl Render for HostBootstrap {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let content = div().child("WP-01 host dependency proof; chart implementation is pending.");
        #[cfg(feature = "kit")]
        let content = content.child(
            gpui_kit::component::button::Button::new("dependency-proof").label("Kit compatible"),
        );
        content
    }
}

fn main() {
    gpui_platform::application().run(|cx| {
        #[cfg(feature = "kit")]
        {
            // This call and the Render implementation must use the same GPUI identity.
            let init: fn(&mut gpui::App) = gpui_kit::init;
            init(cx);
        }
        cx.spawn(async move |cx| {
            let result = cx.open_window(WindowOptions::default(), |_window, cx| {
                let view = cx.new(|_| HostBootstrap);
                #[cfg(feature = "kit")]
                let view = cx.new(|cx| gpui_kit::component::Root::new(view, _window, cx));
                view
            });
            if let Err(error) = result {
                eprintln!("Host bootstrap window failed: {error}");
            }
        })
        .detach();
    });
}
