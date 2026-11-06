//! Icon pack preview and selection, with one pack visible at a time.
use super::{seams::Ink, SettingsView};
use flea::{jsondoc::Json, ui_logic_st1::edit};
use gpui::{
    canvas, div, prelude::*, px, Context, Div, KeyDownEvent, MouseButton, SharedString, Stateful,
    Window,
};

impl SettingsView {
    fn choose_icon_pack(&mut self, cx: &mut Context<Self>) {
        let name = self
            .icon_catalog
            .as_ref()
            .and_then(|packs| self.icon_index.checked_sub(1).and_then(|i| packs.get(i)))
            .map_or("", |pack| &pack.pack.name)
            .to_owned();
        self.panel
            .write(&edit::setting("iconTheme", Json::Str(name)));
        self.flush(cx);
    }

    pub(super) fn icon_picker_key(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) -> bool {
        let max = self.icon_catalog.as_ref().map_or(0, Vec::len);
        match event.keystroke.key.as_str() {
            "left" | "up" | "h" | "k" => self.icon_index = self.icon_index.saturating_sub(1),
            "right" | "down" | "l" | "j" => self.icon_index = (self.icon_index + 1).min(max),
            "enter" | "space" => self.choose_icon_pack(cx),
            "escape" | "backspace" => self.icon_page = false,
            _ => return false,
        }
        cx.notify();
        cx.stop_propagation();
        true
    }

    pub(super) fn icon_picker(
        &self,
        ink: &Ink,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> Stateful<Div> {
        let s = self.panel.scale;
        let side = s.of(28.).round().max(1.) as u32;
        let max = self.icon_catalog.as_ref().map_or(0, Vec::len);
        let start = self.icon_index.saturating_sub(3);
        let end = (start + 4).min(max + 1);
        let selected = self
            .panel
            .document()
            .get("iconTheme")
            .and_then(Json::as_str)
            .unwrap_or("");
        let mut list = div().flex().flex_col().gap(px(s.of(8.)));
        for index in start..end {
            let pack = self
                .icon_catalog
                .as_ref()
                .and_then(|packs| index.checked_sub(1).and_then(|i| packs.get(i)));
            let name = pack.map_or_else(
                || self.panel.catalog.tr("settings.theme.builtin"),
                |p| p.pack.name.clone(),
            );
            let is_selected = if index == 0 {
                selected.is_empty()
            } else {
                pack.is_some_and(|p| p.pack.name == selected)
            };
            let mut preview = div().flex().gap(px(s.of(4.)));
            for (i, role) in flea::icon_packs::PREVIEW_ICONS.iter().enumerate() {
                let theme = pack.map(|p| p.pack.name.clone());
                if let Some(theme) = &theme {
                    crate::icon_themes::request(theme, role, side, window, cx);
                }
                let role = role.to_string();
                let fallback = [
                    crate::row::marks::Mark::Folder,
                    crate::row::marks::Mark::FileText,
                    crate::row::marks::Mark::Image,
                    crate::row::marks::Mark::Film,
                    crate::row::marks::Mark::Music,
                    crate::row::marks::Mark::Terminal,
                ][i];
                let color = ink.foreground;
                preview = preview.child(
                    canvas(
                        |_, _, _| (),
                        move |bounds, _, window, cx| {
                            let painted = theme.as_ref().is_some_and(|theme| {
                                crate::icon_themes::paint(theme, &role, side, bounds, window, cx)
                            });
                            if !painted {
                                let (key, data) = fallback.svg();
                                let _ = window.paint_svg(
                                    bounds,
                                    key,
                                    Some(data),
                                    gpui::TransformationMatrix::unit(),
                                    color,
                                    cx,
                                );
                            }
                        },
                    )
                    .size(px(side as f32)),
                );
            }
            list = list.child(
                div()
                    .id(("icon-pack", index))
                    .min_w_0()
                    .p(px(s.of(8.)))
                    .cursor_pointer()
                    .border_1()
                    .border_color(if self.icon_index == index {
                        ink.accent_frame
                    } else {
                        ink.muted_frame
                    })
                    .flex()
                    .flex_col()
                    .gap(px(s.of(5.)))
                    .child(SharedString::from(format!(
                        "{}{}",
                        if is_selected { "✓  " } else { "" },
                        name
                    )))
                    .child(preview)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |view, _, _, cx| {
                            view.icon_index = index;
                            view.choose_icon_pack(cx);
                        }),
                    ),
            );
        }
        let button = |id: &'static str, label: String| {
            div()
                .id(id)
                .px(px(s.of(12.)))
                .py(px(s.of(7.)))
                .border_1()
                .border_color(ink.muted_frame)
                .cursor_pointer()
                .child(SharedString::from(label))
        };
        div()
            .id("settings-pane")
            .flex_1()
            .min_w_0()
            .h_full()
            .px(px(s.of(24.)))
            .pt(px(s.of(12.)))
            .flex()
            .flex_col()
            .gap(px(s.of(8.)))
            .child(list)
            .child(
                div()
                    .flex()
                    .gap(px(s.of(8.)))
                    .child(button("icon-prev", "‹".into()).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|view, _, _, cx| {
                            view.icon_index = view.icon_index.saturating_sub(1);
                            cx.notify();
                        }),
                    ))
                    .child(button("icon-next", "›".into()).on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|view, _, _, cx| {
                            let max = view.icon_catalog.as_ref().map_or(0, Vec::len);
                            view.icon_index = (view.icon_index + 1).min(max);
                            cx.notify();
                        }),
                    ))
                    .child(
                        button("icon-back", self.panel.catalog.tr("settings.theme.back"))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(|view, _, _, cx| {
                                    view.icon_page = false;
                                    cx.notify();
                                }),
                            ),
                    ),
            )
    }
}
