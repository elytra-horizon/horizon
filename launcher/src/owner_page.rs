//! The account gate in the window (`horizon_launcher::account::launch`): the screen that says no
//! account here can play, and points at Settings, where ely.by and offline accounts are added.
//! It covers the launcher once for players without one, and again whenever Play is refused.
use super::*;

impl Launcher {
    /// Whether the screen covers the window: no account can play, and either Play was refused or
    /// the player hasn't read it yet. The data question comes first.
    pub(crate) fn asking_owner(&self) -> bool {
        self.owner == Some(false)
            && (self.owner_screen_open || !self.settings.owner_notice_seen)
            && !self.asking_consent()
            && self.opening.is_none()
    }

    /// Closes it; it comes back with the next refused Play.
    fn close_owner_screen(&mut self) {
        self.cancel_sign_in();
        self.owner_screen_open = false;
        if !self.settings.owner_notice_seen {
            self.settings.owner_notice_seen = true;
            self.save_settings();
        }
    }

    pub(crate) fn owner_screen(&mut self, cx: &mut Context<Self>) -> Stateful<Div> {
        let t = self.theme;
        let n = t.night;
        if crate::glass_ui::on() {
            return self.glass_owner_screen(cx);
        }
        let line = |s: &str, color: u32| {
            shell::text(theme::night::UI, shell::c(color))
                .relative()
                .child(tr(s).to_string())
        };
        let body = div()
            .relative()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                shell::text(theme::night::CAPTION, shell::c(n.muted))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(tr("Account").to_string()),
            )
            .child(
                shell::text(theme::night::TITLE, shell::c(n.ink))
                    .mb(px(4.))
                    .child(tr("No account can play yet").to_string()),
            )
            .child(line(
                "Horizon plays with an ely.by account or an offline name. Add one in Settings.",
                n.ink_soft,
            ))
            .child(line(
                "Microsoft sign-in is not part of this build.",
                n.ink_soft,
            ));
        let answers = div()
            .relative()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(10.))
            .mt(px(14.))
            .child(
                shell::action("owner-accounts", shell::Action::Main, &t)
                    .child(shell::label(tr("Accounts…").to_string()))
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.close_owner_screen();
                        this.go(Page::Settings);
                        cx.notify();
                    })),
            )
            .child(
                shell::action("owner-later", shell::Action::Quiet, &t)
                    .child(shell::label(tr("Not now").to_string()))
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.close_owner_screen();
                        cx.notify();
                    })),
            );
        // Opaque: it sits over the page's content, not over the bare sky.
        let card = plate::cut(
            theme::night::PIXEL_CORNER,
            2,
            Some(plate::alpha(n.card, 0.98)),
            Some((plate::alpha(n.stroke, 2. * n.stroke_alpha), 1.)),
        )
        .flex()
        .flex_col()
        .gap(px(8.))
        // A set width: text measured at it wraps where it is drawn.
        .w(px(520.))
        .max_w_full()
        .p(px(32.))
        .child(body)
        .child(answers);
        // The sky stays behind, dimmed: the sheet is over the launcher, not instead of it.
        div()
            .id("owner-gate")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .px(px(32.))
            .py(px(24.))
            .bg(plate::alpha(n.page, 0.8))
            .on_scroll(|_, _, cx| cx.stop_propagation())
            .occlude()
            .child(card)
    }

    /// The screen in the glass look: a dialog plate over the dimmed, blurred launcher (the
    /// mockup's modals), its text in the card's inner panel.
    fn glass_owner_screen(&self, cx: &mut Context<Self>) -> Stateful<Div> {
        use crate::glass_ui::{self as ui, white};
        use horizon_theme::glass::{ink, radius};
        let t = self.theme;
        let card = ui::dialog_plate(radius::MODAL)
            .id("owner-card")
            .occlude()
            .flex()
            .flex_col()
            // A set width: text measured at it wraps where it is drawn.
            .w(px(520.))
            .max_w_full()
            .p(px(32.))
            .child(ui::text(13., FontWeight::MEDIUM, ink::QUIET).child(tr("Account")))
            .child(
                div()
                    .mt(px(8.))
                    .font_family(horizon_theme::glass::DISPLAY_FONT)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(28.))
                    .line_height(px(35.))
                    .text_color(white(ink::FULL))
                    .child(tr("No account can play yet")),
            )
            .child(
                div()
                    .mt(px(12.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(ui::text(14., FontWeight::NORMAL, ink::BODY).child(tr(
                        "Horizon plays with an ely.by account or an offline name. Add one in \
                         Settings.",
                    )))
                    .child(
                        ui::text(14., FontWeight::NORMAL, ink::BODY)
                            .child(tr("Microsoft sign-in is not part of this build.")),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(8.))
                    .mt(px(24.))
                    .child(
                        ui::button(
                            "owner-accounts".into(),
                            tr("Accounts…").into(),
                            Kind::Primary,
                            &t,
                        )
                        .h(px(44.))
                        .px(px(20.))
                        .on_click(cx.listener(
                            |this, _: &ClickEvent, _, cx| {
                                this.close_owner_screen();
                                this.go(Page::Settings);
                                cx.notify();
                            },
                        )),
                    )
                    .child(
                        ui::button("owner-later".into(), tr("Not now").into(), Kind::Ghost, &t)
                            .h(px(44.))
                            .px(px(20.))
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.close_owner_screen();
                                cx.notify();
                            })),
                    ),
            );
        div()
            .id("owner-gate")
            .absolute()
            .top_0()
            .left_0()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .p(px(24.))
            .on_scroll(|_, _, cx| cx.stop_propagation())
            .occlude()
            .child(ui::scrim_backdrop())
            .child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .size_full()
                    .bg(ui::scrim()),
            )
            .child(card)
    }
}
