//! Settings' Account card in the glass look, as the mockup's: who plays now (a big head, the
//! name, how), and the Microsoft card beside it with its white Sign in with Microsoft button.
//!
//! Not in the mockup, kept in its patterns: the Microsoft accounts already signed in (in the
//! Microsoft card, with Play as, Refresh and Sign out), the sign-in's code, progress and problems.

use super::*;
use crate::glass_ui::{self as ui, white, Tone};
use horizon_theme::glass::{ink, radius};

impl Launcher {
    pub(super) fn glass_account(&mut self, cx: &mut Context<Self>) -> Div {
        let wide = self.window_width >= crate::glass_home::WIDE;
        let left = self.glass_who_plays();
        let right = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(self.glass_microsoft_card(cx))
            .child(self.glass_elyby_card(cx))
            .child(self.glass_offline_card(cx));
        if wide {
            div()
                .flex()
                .flex_row()
                .gap(px(8.))
                .child(left.flex_1().min_w(px(280.)).p(px(20.)))
                .child(right.flex_none().w(px(360.)))
        } else {
            div()
                .flex()
                .flex_col()
                .gap(px(16.))
                .child(left.w_full().min_w_0().p(px(20.)))
                .child(right.w_full())
        }
    }

    /// The player now.
    fn glass_who_plays(&self) -> Div {
        let t = self.theme;
        let (uuid, name, status) = match self.accounts.selected() {
            Some(a) if a.uuid == account::offline::uuid(&a.name) => (
                Some(a.uuid.clone()),
                a.name.clone(),
                (
                    Tone::Live,
                    tr("Offline play · local worlds and offline servers"),
                ),
            ),
            Some(a) => (
                Some(a.uuid.clone()),
                a.name.clone(),
                (Tone::Live, tr("Signed in · online servers too")),
            ),
            None => (
                None,
                tr("Add an account").to_string(),
                (
                    Tone::Off,
                    tr("Microsoft, ely.by, or an offline name: pick one in Settings."),
                ),
            ),
        };
        let head = div()
            .flex_none()
            .size(px(80.))
            .rounded(px(20.))
            .overflow_hidden()
            .child(
                self.profile_heads
                    .draw(uuid.as_deref(), 80., self.scale, &t),
            );
        let header = div().flex().items_center().gap(px(20.)).child(head).child(
            div()
                .flex()
                .flex_col()
                .min_w_0()
                .gap(px(10.))
                .child(
                    div()
                        .font_family(horizon_theme::glass::DISPLAY_FONT)
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(32.))
                        .line_height(px(32.))
                        .text_color(white(ink::FULL))
                        .truncate()
                        .child(name),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap(px(8.))
                        .child(ui::dot(status.0).mt(px(7.)))
                        .child(
                            ui::text(14., FontWeight::NORMAL, 0.65)
                                .flex_1()
                                .min_w_0()
                                .line_height(px(21.))
                                .child(status.1),
                        ),
                ),
        );

        div().flex().flex_col().child(header)
    }

    /// The Microsoft card: the accounts signed in here, then adding one (the mockup's card).
    fn glass_microsoft_card(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let has_accounts = !self.accounts.accounts.is_empty();
        let (rows, _) = if has_accounts {
            ui::flat_rows(|| self.microsoft_rows(cx))
        } else {
            (div(), 0)
        };
        let (title, detail, action) = self.sign_in_parts(cx);
        let configured = account::client_id().is_some();
        let idle = self.sign_in_state.is_none()
            || matches!(self.sign_in_state, Some(SignInState::Done(_)));
        let first = !has_accounts && self.sign_in_target.is_none();
        let heading = if first {
            tr("Play online with your Microsoft account").to_string()
        } else {
            title
        };
        let note: AnyElement = if first && configured && self.sign_in_state.is_none() {
            div()
                .child(tr(
                    "Sign in with the account that owns Minecraft: Java Edition. Your name and \
                     skin come along to online servers, like in other launchers.",
                ))
                .into_any_element()
        } else {
            detail.into_any_element()
        };
        // Adding an account is the mockup's white button; signing one in again, the code and
        // the waiting keep their own buttons.
        let button: Option<AnyElement> = if configured && idle && self.sign_in_target.is_none() {
            Some(
                keyboard(
                    div()
                        .id("glass-sign-in")
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap(px(10.))
                        .h(px(48.))
                        .w_full()
                        .mt(px(24.))
                        .rounded(px(radius::ROW))
                        .bg(gpui::white())
                        // On a light plate, Microsoft's white button keeps an edge.
                        .when(ui::light(), |d| d.border_1().border_color(ui::white(0.15)))
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.9))
                        .font_family(horizon_theme::glass::FONT)
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(gpui::rgb(0x100a1f))
                        .child(microsoft::logo(18.))
                        .child(tr("Sign in with Microsoft")),
                    &t,
                )
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.start_sign_in();
                    cx.notify();
                }))
                .into_any_element(),
            )
        } else {
            action.map(|a| div().mt(px(20.)).child(a).into_any_element())
        };
        div()
            .flex()
            .flex_col()
            .rounded(px(16.))
            .bg(white(0.045))
            .p(px(24.))
            .when(has_accounts, |d| {
                d.child(rows.mx(px(-16.)).mt(px(-16.)).mb(px(16.)))
            })
            .child(microsoft::logo(36.))
            .child(
                div()
                    .mt(px(20.))
                    .font_family(horizon_theme::glass::DISPLAY_FONT)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(22.))
                    .line_height(px(27.5))
                    .text_color(white(ink::FULL))
                    .child(heading),
            )
            .child(
                ui::text(14., FontWeight::NORMAL, 0.65)
                    .mt(px(8.))
                    .line_height(px(21.7))
                    .child(note),
            )
            .children(button)
            .when(configured, |d| {
                d.child(
                    ui::text(12., FontWeight::NORMAL, 0.45)
                        .mt(px(12.))
                        .line_height(px(18.))
                        .child(tr(
                            "Microsoft's own page opens in your browser. Horizon never sees your \
                             password.",
                        )),
                )
            })
    }

    /// The ely.by card: its OAuth2 page in the browser brings the account back to this computer.
    fn glass_elyby_card(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let configured = account::ely_client().is_some();
        let mine = self.sign_in_provider == Some(account::Provider::ElyBy);
        let busy = mine
            && matches!(
                self.sign_in_state,
                Some(SignInState::Starting | SignInState::Browser(_) | SignInState::Working(_))
            );
        let detail: AnyElement = if !configured {
            div()
                .child(tr(account::ELY_NOT_CONFIGURED))
                .into_any_element()
        } else if mine {
            match &self.sign_in_state {
                Some(SignInState::Starting) => {
                    div().child(tr("Opening ely.by…")).into_any_element()
                }
                Some(SignInState::Browser(_)) => div()
                    .child(tr("Finish signing in to ely.by in your browser."))
                    .into_any_element(),
                Some(SignInState::Working(phase)) => {
                    div().child(tr(phase.label())).into_any_element()
                }
                Some(SignInState::Done(Ok(a))) => div()
                    .child(trf("Signed in as {name}.", &[("name", &a.name)]))
                    .into_any_element(),
                Some(SignInState::Done(Err(e))) => div()
                    .text_color(t.danger)
                    .child(e.clone())
                    .into_any_element(),
                _ => div().child(tr("Add an ely.by account.")).into_any_element(),
            }
        } else {
            div()
                .child(tr(
                    "ely.by is another Minecraft account service: its skins, capes and the online \
                     servers that accept it.",
                ))
                .into_any_element()
        };
        let action: Option<AnyElement> = if !configured {
            None
        } else if let Some(SignInState::Browser(url)) = &self.sign_in_state {
            if !mine {
                None
            } else {
                let uri = url.clone();
                Some(
                    div()
                        .flex()
                        .gap(px(4.))
                        .child(
                            keyboard(
                                div()
                                    .id("glass-elyby-open")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .h(px(44.))
                                    .flex_1()
                                    .rounded(px(radius::ROW))
                                    .bg(white(0.08))
                                    .cursor_pointer()
                                    .hover(|s| s.opacity(0.9))
                                    .font_family(horizon_theme::glass::FONT)
                                    .text_size(px(14.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(white(ink::FULL))
                                    .child(tr("Open ely.by")),
                                &t,
                            )
                            .on_click(move |_, _, cx| cx.open_url(&uri)),
                        )
                        .child(
                            keyboard(
                                div()
                                    .id("glass-elyby-cancel")
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .h(px(44.))
                                    .px(px(16.))
                                    .rounded(px(radius::ROW))
                                    .cursor_pointer()
                                    .hover(|s| s.opacity(0.9))
                                    .font_family(horizon_theme::glass::FONT)
                                    .text_size(px(14.))
                                    .text_color(white(0.65))
                                    .child(tr("Cancel")),
                                &t,
                            )
                            .on_click(cx.listener(
                                |this, _: &ClickEvent, _, cx| {
                                    this.cancel_sign_in();
                                    cx.notify();
                                },
                            )),
                        )
                        .into_any_element(),
                )
            }
        } else if busy {
            Some(
                keyboard(
                    div()
                        .id("glass-elyby-working-cancel")
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(px(48.))
                        .w_full()
                        .rounded(px(radius::ROW))
                        .bg(white(0.08))
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.9))
                        .font_family(horizon_theme::glass::FONT)
                        .text_size(px(15.))
                        .text_color(white(ink::FULL))
                        .child(tr("Cancel")),
                    &t,
                )
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.cancel_sign_in();
                    cx.notify();
                }))
                .into_any_element(),
            )
        } else {
            Some(
                keyboard(
                    div()
                        .id("glass-elyby-sign-in")
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(px(48.))
                        .w_full()
                        .rounded(px(radius::ROW))
                        .bg(white(0.10))
                        .border_1()
                        .border_color(white(0.16))
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.9))
                        .font_family(horizon_theme::glass::FONT)
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(white(ink::FULL))
                        .child(tr("Sign in with ely.by")),
                    &t,
                )
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.start_elyby_sign_in();
                    cx.notify();
                }))
                .into_any_element(),
            )
        };
        div()
            .flex()
            .flex_col()
            .rounded(px(16.))
            .bg(white(0.045))
            .p(px(24.))
            .child(
                div()
                    .font_family(horizon_theme::glass::DISPLAY_FONT)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(20.))
                    .line_height(px(25.))
                    .text_color(white(ink::FULL))
                    .child("ely.by"),
            )
            .child(
                ui::text(14., FontWeight::NORMAL, 0.65)
                    .mt(px(8.))
                    .line_height(px(21.7))
                    .child(detail),
            )
            .children(action.map(|a| div().mt(px(20.)).child(a)))
    }

    /// The offline card: a name, then Play. The skin of a player of that name is looked up when
    /// the game starts, and in the launcher for the head.
    fn glass_offline_card(&mut self, cx: &mut Context<Self>) -> Div {
        let t = self.theme;
        let mine = self.sign_in_provider == Some(account::Provider::Offline);
        let busy = mine
            && matches!(
                self.sign_in_state,
                Some(SignInState::Starting | SignInState::Working(_))
            );
        let detail: AnyElement = if mine {
            match &self.sign_in_state {
                Some(SignInState::Starting) => div()
                    .child(tr("Adding the offline account…"))
                    .into_any_element(),
                Some(SignInState::Working(_)) => div()
                    .child(tr("Looking for this player's skin…"))
                    .into_any_element(),
                Some(SignInState::Done(Ok(a))) => div()
                    .child(trf("Playing as {name}.", &[("name", &a.name)]))
                    .into_any_element(),
                Some(SignInState::Done(Err(e))) => div()
                    .text_color(t.danger)
                    .child(e.clone())
                    .into_any_element(),
                _ => div()
                    .child(tr("Enter a player name and play, no account needed."))
                    .into_any_element(),
            }
        } else {
            div()
                .child(tr(
                    "Play without an account: a name, and the skin of a player who has it. \
                     Offline-mode servers only.",
                ))
                .into_any_element()
        };
        let focused = self.offline_focused;
        let field = div()
            .id("glass-offline-name")
            .track_focus(&self.offline_focus)
            .flex()
            .items_center()
            .w_full()
            .h(px(44.))
            .px(px(16.))
            .rounded(px(radius::ROW))
            .bg(white(if focused { 0.11 } else { 0.07 }))
            .font_family(horizon_theme::glass::FONT)
            .text_size(px(14.))
            .text_color(white(ink::FULL))
            .cursor_text()
            .overflow_hidden()
            .when(focused, |d| d.child(ui::focus_ring(radius::ROW)))
            .child(line_edit::content(
                &self.offline_input,
                &t,
                focused,
                false,
                false,
                tr("Player name"),
            ))
            .on_click(cx.listener(|this, _: &ClickEvent, window, cx| {
                window.focus(&this.offline_focus, cx);
                cx.notify();
            }))
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, window, cx| {
                match line_edit::handle(&mut this.offline_input, ev, cx) {
                    line_edit::Key::Pass => return,
                    line_edit::Key::Enter => this.begin_offline_sign_in(),
                    line_edit::Key::Escape => window.blur(cx),
                    _ => {}
                }
                cx.stop_propagation();
                cx.notify();
            }));
        let field = line_edit::mouse(field, cx, |this| &mut this.offline_input);
        let action: Option<AnyElement> = if busy {
            Some(
                keyboard(
                    div()
                        .id("glass-offline-cancel")
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(px(44.))
                        .w_full()
                        .rounded(px(radius::ROW))
                        .bg(white(0.08))
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.9))
                        .font_family(horizon_theme::glass::FONT)
                        .text_size(px(14.))
                        .text_color(white(ink::FULL))
                        .child(tr("Cancel")),
                    &t,
                )
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.cancel_sign_in();
                    cx.notify();
                }))
                .into_any_element(),
            )
        } else {
            Some(
                keyboard(
                    div()
                        .id("glass-offline-add")
                        .flex()
                        .items_center()
                        .justify_center()
                        .h(px(48.))
                        .w_full()
                        .rounded(px(radius::ROW))
                        .bg(white(0.10))
                        .border_1()
                        .border_color(white(0.16))
                        .cursor_pointer()
                        .hover(|s| s.opacity(0.9))
                        .font_family(horizon_theme::glass::FONT)
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(white(ink::FULL))
                        .child(tr("Add offline account")),
                    &t,
                )
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.begin_offline_sign_in();
                    cx.notify();
                }))
                .into_any_element(),
            )
        };
        div()
            .flex()
            .flex_col()
            .rounded(px(16.))
            .bg(white(0.045))
            .p(px(24.))
            .child(
                div()
                    .font_family(horizon_theme::glass::DISPLAY_FONT)
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(20.))
                    .line_height(px(25.))
                    .text_color(white(ink::FULL))
                    .child(tr("Offline account")),
            )
            .child(
                ui::text(14., FontWeight::NORMAL, 0.65)
                    .mt(px(8.))
                    .line_height(px(21.7))
                    .child(detail),
            )
            .child(div().mt(px(16.)).child(field))
            .children(action.map(|a| div().mt(px(12.)).child(a)))
    }
}
