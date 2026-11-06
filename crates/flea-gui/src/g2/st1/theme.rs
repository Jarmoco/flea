//! RGB seed and window opacity controls.
use super::{panel::Side, seams::Ink, SettingsView};
use flea::{jsondoc::Json, ui_logic_st1::edit};
use gpui::{
    div, prelude::*, px, Context, Div, KeyDownEvent, MouseButton, SharedString, Stateful, Window,
};

impl SettingsView {
    fn values(&self) -> [u8; 4] {
        let seed = self
            .panel
            .document()
            .get("themeSeed")
            .and_then(Json::as_str)
            .and_then(flea::seed_theme::normalize)
            .unwrap_or_else(|| "#638be8".into());
        let channel = |at| u8::from_str_radix(&seed[at..at + 2], 16).unwrap_or(0);
        let opacity = self
            .panel
            .document()
            .get("themeOpacity")
            .and_then(Json::as_f64)
            .filter(|n| *n >= 0.)
            .unwrap_or(100.) as u8;
        [channel(1), channel(3), channel(5), opacity]
    }

    fn set_channel(&mut self, channel: usize, value: u8, cx: &mut Context<Self>) {
        let mut values = self.values();
        values[channel] = value;
        let patch = if channel == 3 {
            edit::setting("themeOpacity", Json::Num(value.to_string()))
        } else {
            edit::setting(
                "themeSeed",
                Json::Str(format!(
                    "#{:02x}{:02x}{:02x}",
                    values[0], values[1], values[2]
                )),
            )
        };
        self.panel.write(&patch);
        self.theme_channel = channel;
        self.flush(cx);
    }

    fn reset_theme(&mut self, cx: &mut Context<Self>) {
        self.panel.write(&Json::Obj(vec![
            ("themeSeed".into(), Json::Str(String::new())),
            ("themeOpacity".into(), Json::Num("-1".into())),
        ]));
        self.flush(cx);
    }

    pub(super) fn seed_key(
        &mut self,
        event: &KeyDownEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.panel.section != "theme"
            || self.panel.query.is_some()
            || self.panel.side != Side::Pane
        {
            return false;
        }
        match event.keystroke.key.as_str() {
            "up" | "k" => self.theme_channel = self.theme_channel.saturating_sub(1),
            "down" | "j" => self.theme_channel = (self.theme_channel + 1).min(4),
            "enter" | "space" if self.theme_channel == 4 => self.reset_theme(cx),
            "left" | "h" if self.theme_channel < 4 => {
                let v = self.values()[self.theme_channel].saturating_sub(1);
                self.set_channel(self.theme_channel, v, cx);
            }
            "right" | "l" if self.theme_channel < 4 => {
                let v = self.values()[self.theme_channel]
                    .saturating_add(1)
                    .min(if self.theme_channel == 3 { 100 } else { 255 });
                self.set_channel(self.theme_channel, v, cx);
            }
            "tab" => self.panel.side = Side::Rail,
            "escape" => self.panel.requests.push(super::Request::Close),
            _ => return false,
        }
        self.flush(cx);
        cx.stop_propagation();
        true
    }

    pub(super) fn theme_pane(&self, ink: &Ink, cx: &mut Context<Self>) -> Stateful<Div> {
        let s = self.panel.scale;
        let values = self.values();
        let mut unfilled = ink.muted_frame;
        unfilled.l *= 0.35;
        unfilled.a *= 0.7;
        let mut pane = div()
            .id("settings-pane")
            .flex_1()
            .min_w_0()
            .h_full()
            .px(px(s.of(24.)))
            .pt(px(s.of(24.)))
            .flex()
            .flex_col()
            .gap(px(s.of(18.)));
        for (channel, label) in [
            "settings.theme.red",
            "settings.theme.green",
            "settings.theme.blue",
            "settings.theme.opacity",
        ]
        .into_iter()
        .enumerate()
        {
            let max = if channel == 3 { 100 } else { 255 };
            let current = values[channel] as usize;
            let mut track = div()
                .id(("theme-track", channel))
                .w_full()
                .h(px(s.of(24.)))
                .flex()
                .items_center()
                .cursor_pointer();
            for value in 0..=max {
                track = track.child(
                    div()
                        .id(("theme-stop", channel * 256 + value))
                        .flex_1()
                        .min_w_0()
                        .h(px(s.of(16.)))
                        .bg(if value <= current {
                            ink.accent
                        } else {
                            unfilled
                        })
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |view, _, _, cx| {
                                view.panel.side = Side::Pane;
                                view.set_channel(channel, value as u8, cx);
                            }),
                        )
                        .on_mouse_move(cx.listener(
                            move |view, event: &gpui::MouseMoveEvent, _, cx| {
                                if event.pressed_button == Some(MouseButton::Left)
                                    && view.values()[channel] != value as u8
                                {
                                    view.set_channel(channel, value as u8, cx);
                                }
                            },
                        )),
                );
            }
            let title = format!(
                "{}   {}{}",
                self.panel.catalog.tr(label),
                current,
                if channel == 3 { "%" } else { "" }
            );
            pane = pane.child(
                div()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap(px(s.of(3.)))
                    .text_color(
                        if self.panel.side == Side::Pane && self.theme_channel == channel {
                            ink.accent
                        } else {
                            ink.foreground
                        },
                    )
                    .child(SharedString::from(title))
                    .child(track),
            );
        }
        if let Some(error) = &self.panel.save_error {
            pane = pane.child(
                div()
                    .text_color(ink.error)
                    .child(SharedString::from(error.resolve(&self.panel.catalog))),
            );
        }
        pane.child(
            div()
                .id("theme-reset")
                .self_start()
                .cursor_pointer()
                .border_1()
                .border_color(
                    if self.panel.side == Side::Pane && self.theme_channel == 4 {
                        ink.accent_frame
                    } else {
                        ink.muted_frame
                    },
                )
                .px(px(s.of(12.)))
                .py(px(s.of(6.)))
                .child(SharedString::from(
                    self.panel.catalog.tr("settings.theme.reset"),
                ))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|view, _, _, cx| {
                        view.reset_theme(cx);
                    }),
                ),
        )
        .child(
            div()
                .min_w_0()
                .text_size(self.typeface.caption)
                .text_color(ink.muted)
                .child(SharedString::from(
                    self.panel.catalog.tr("settings.theme.reset_description"),
                )),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::{tests::panel, SettingsView};
    use gpui::TestAppContext;

    #[gpui::test]
    fn keyboard_changes_rgb_and_opacity(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|window, cx| {
            let mut view = SettingsView::new(panel("{}"), cx);
            view.panel.show_section("theme");
            window.focus(&view.focus, cx);
            view
        });
        cx.update(|window, cx| window.draw(cx).clear(cx));
        cx.simulate_keystrokes("right down right down right down left");
        view.read_with(cx, |view, _| {
            assert_eq!(
                view.panel
                    .document()
                    .get("themeSeed")
                    .and_then(|v| v.as_str()),
                Some("#648ce9")
            );
            assert_eq!(
                view.panel
                    .document()
                    .get("themeOpacity")
                    .and_then(|v| v.as_f64()),
                Some(99.)
            );
        });
    }
}
