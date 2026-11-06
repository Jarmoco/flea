//! Load icon-theme metadata and preview paths away from the UI thread.
use super::SettingsView;
use gpui::Context;

impl SettingsView {
    pub(super) fn load_icon_catalog(&mut self, cx: &mut Context<Self>) {
        if self.icon_catalog.is_some() || self.icon_catalog_loading {
            return;
        }
        self.icon_catalog_loading = true;
        let work = cx.background_executor().spawn(async {
            let roots = flea::icon_packs::session_roots();
            flea::icon_packs::catalog(&roots, 24, 1)
        });
        cx.spawn(async move |view, cx| {
            let catalog = work.await;
            let _ = view.update(cx, |view, cx| {
                view.icon_catalog = Some(catalog);
                view.icon_catalog_loading = false;
                cx.notify();
            });
        })
        .detach();
    }
}

