//! The seed editor uses the same platform text input as Settings search.
use super::{
    input::{type_key, Field, Host, KeyUse},
    panel::Side,
    seams::Ink,
    SettingsView,
};
use gpui::{
    div, prelude::*, px, Context, Div, KeyDownEvent, MouseButton, SharedString, Stateful, Window,
};

impl super::Panel {
    pub fn set_theme_seed(&mut self, seed: &str) {
        self.write(&flea::ui_logic_st1::edit::setting(
            "themeSeed",
            flea::jsondoc::Json::Str(seed.to_owned()),
        ));
    }
}

impl SettingsView {
    pub(super) fn sync_seed(&mut self) {
        if self.panel.section != "theme" || self.panel.query.is_some() {
            self.seed_draft = None;
            self.seed_invalid = false;
        } else if self.seed_draft.is_none() {
            self.seed_draft = Some(
                self.panel
                    .document()
                    .get("themeSeed")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned(),
            );
        }
    }

    pub(super) fn seed_key(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.panel.section != "theme"
            || self.panel.query.is_some()
            || self.panel.side != Side::Pane
        {
            return false;
        }
        self.sync_seed();
        self.sync_editor();
        match self.editor.key(event, cx) {
            KeyUse::Text if !type_key(self, event, window, cx) => return true,
            KeyUse::Text => {}
            KeyUse::Handled => self.edited(cx),
            KeyUse::Command => match event.keystroke.key.as_str() {
                "enter" | "return" | "kp_enter" => {
                    self.apply_seed(cx);
                }
                "tab" | "backtab" => {
                    self.panel.side = Side::Rail;
                    self.sync_editor();
                    cx.notify();
                }
                "escape" => {
                    self.panel.requests.push(super::Request::Close);
                    self.flush(cx);
                }
                _ => {}
            },
        }
        cx.stop_propagation();
        true
    }

    fn apply_seed(&mut self, cx: &mut Context<Self>) {
        let value = self.seed_draft.as_deref().unwrap_or("");
        let seed = if value.is_empty() {
            Some(String::new())
        } else {
            flea::seed_theme::normalize(value)
        };
        if let Some(seed) = seed {
            self.panel.set_theme_seed(&seed);
            self.seed_draft = Some(seed);
            self.seed_invalid = false;
            self.flush(cx);
        } else {
            self.seed_invalid = true;
            cx.notify();
        }
    }

    pub(super) fn theme_pane(&self, ink: &Ink, cx: &mut Context<Self>) -> Stateful<Div> {
        let s = self.panel.scale;
        let active = self.panel.side == Side::Pane;
        let title = self.panel.catalog.tr("settings.theme.seed");
        let hint = if self.seed_invalid {
            self.panel.catalog.tr("settings.theme.invalid")
        } else if let Some(error) = &self.panel.save_error {
            error.resolve(&self.panel.catalog)
        } else {
            self.panel.catalog.tr("settings.theme.hint")
        };
        div()
            .id("settings-pane")
            .flex_1()
            .h_full()
            .px(px(s.of(24.)))
            .pt(px(s.of(24.)))
            .flex()
            .flex_col()
            .gap(px(s.of(12.)))
            .child(div().child(SharedString::from(title)))
            .child(
                div()
                    .id("theme-seed-field")
                    .h(px(s.of(36.)))
                    .px(px(s.of(10.)))
                    .border_1()
                    .border_color(if self.seed_invalid {
                        ink.error
                    } else if active {
                        ink.accent_frame
                    } else {
                        ink.muted_frame
                    })
                    .bg(ink.background)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|view, _, window, cx| {
                            view.panel.side = Side::Pane;
                            view.sync_seed();
                            view.sync_editor();
                            window.focus(&view.focus, cx);
                            cx.notify();
                        }),
                    )
                    .when(active, |field| {
                        field.child(Field::mount(cx.entity(), &self.typeface, true))
                    })
                    .when(!active, |field| {
                        field.child(SharedString::from(
                            self.seed_draft.clone().unwrap_or_default(),
                        ))
                    }),
            )
            .child(
                div()
                    .id("theme-seed-apply")
                    .self_start()
                    .px(px(s.of(14.)))
                    .py(px(s.of(7.)))
                    .border_1()
                    .border_color(ink.accent_frame)
                    .cursor_pointer()
                    .child(SharedString::from(
                        self.panel.catalog.tr("settings.theme.apply"),
                    ))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|view, _, _, cx| view.apply_seed(cx)),
                    ),
            )
            .child(
                div()
                    .text_size(self.typeface.caption)
                    .text_color(if self.seed_invalid || self.panel.save_error.is_some() {
                        ink.error
                    } else {
                        ink.muted
                    })
                    .child(SharedString::from(hint)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        ime_tests::{draw, platform},
        tests::panel,
        SettingsView,
    };
    use gpui::TestAppContext;

    #[gpui::test]
    fn seed_field_validates_saves_and_resets(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|window, cx| {
            let mut view = SettingsView::new(panel("{}"), cx);
            view.panel.show_section("theme");
            window.focus(&view.focus, cx);
            view
        });
        draw(cx);
        platform(cx, |input| input.replace_text_in_range(None, "bad"));
        cx.simulate_keystrokes("enter");
        view.read_with(cx, |view, _| {
            assert!(view.seed_invalid);
            assert!(view.panel.document().get("themeSeed").is_none());
        });
        cx.simulate_keystrokes("ctrl-a");
        platform(cx, |input| input.replace_text_in_range(None, "#E54A21"));
        cx.simulate_keystrokes("enter");
        view.read_with(cx, |view, _| {
            assert_eq!(
                view.panel
                    .document()
                    .get("themeSeed")
                    .and_then(|v| v.as_str()),
                Some("#e54a21")
            );
        });
        cx.simulate_keystrokes("ctrl-a backspace enter");
        view.read_with(cx, |view, _| {
            assert_eq!(
                view.panel
                    .document()
                    .get("themeSeed")
                    .and_then(|v| v.as_str()),
                Some("")
            );
        });
    }
}
