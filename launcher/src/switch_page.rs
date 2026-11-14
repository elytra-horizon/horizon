//! Who plays: the top bar's quick switch (a menu under the account chip) between the signed-in
//! ely.by and offline accounts.
use super::*;

impl Launcher {
    /// Which service an account signs in through. An offline account is its name's offline UUID,
    /// so no keychain read is needed to say so; the others come from the status loaded off the
    /// UI thread, and stay unknown until it arrives.
    fn provider_of(&self, a: &account::Account) -> Option<account::Provider> {
        if a.uuid == account::offline::uuid(&a.name) {
            return Some(account::Provider::Offline);
        }
        self.auth_status.get(&a.uuid).and_then(|s| s.provider)
    }

    /// The chip's two lines: the name that plays, and how.
    pub(super) fn who_plays(&self) -> (String, &'static str) {
        match self.accounts.selected() {
            Some(a) => (a.name.clone(), tr(kind(self.provider_of(a)))),
            None => (tr("Add an account").to_string(), tr("No account yet")),
        }
    }

    pub(super) fn play_as(&mut self, uuid: &str) {
        let mut accounts = Accounts::load();
        let done = accounts.play_as(uuid).and_then(|()| accounts.save());
        match done {
            Ok(()) => self.accounts = accounts,
            Err(e) => self.sign_in_state = Some(SignInState::Done(Err(e))),
        }
        self.account_menu = false;
    }

    /// Everyone the player can play as (each account's UUID and name), with how each signs in.
    pub(super) fn players(&self) -> Vec<(String, String, &'static str)> {
        self.accounts
            .accounts
            .iter()
            .map(|a| {
                (
                    a.uuid.clone(),
                    a.name.clone(),
                    tr(short(self.provider_of(a))),
                )
            })
            .collect()
    }

    /// The menu under the account chip, while open.
    pub(super) fn account_menu(&self, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        if !self.account_menu {
            return None;
        }
        let t = self.theme;
        let now = self.accounts.selected().map(|a| a.uuid.clone());
        let mut menu = div()
            .id("account-menu")
            .absolute()
            .top(px(46.))
            .right_0()
            .w(px(280.))
            .p(px(6.))
            .border_2()
            .border_color(t.outline)
            .bg(t.surface)
            .flex()
            .flex_col()
            .occlude()
            .on_mouse_down_out(cx.listener(|this, event: &MouseDownEvent, _, cx| {
                this.account_menu = false;
                this.menu_closed_at = Some(event.position);
                cx.notify();
            }));
        for (i, (who, name, how)) in self.players().into_iter().enumerate() {
            let chosen = Some(&who) == now.as_ref();
            menu = menu.child(
                div()
                    .id(SharedString::from(format!("account-pick-{i}")))
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(8.))
                    .h(px(46.))
                    .px(px(10.))
                    .rounded(px(theme::RADIUS))
                    .cursor_pointer()
                    .hover(move |s| s.bg(t.plate_soft))
                    .child(
                        self.profile_heads
                            .draw(Some(who.as_str()), 24., self.scale, &t),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .child(text(theme::LABEL).truncate().text_color(t.ink).child(name))
                            .child(text(theme::CAPTION).text_color(t.muted).child(how)),
                    )
                    .when(chosen, |d| d.child(div().size(px(6.)).bg(t.accent)))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.play_as(&who);
                        cx.notify();
                    })),
            );
        }
        menu = menu.child(div().my(px(6.)).h(px(1.)).bg(t.rule)).child(
            div()
                .id("account-manage")
                .flex()
                .items_center()
                .h(px(32.))
                .px(px(10.))
                .rounded(px(theme::RADIUS))
                .text_size(px(theme::LABEL.px))
                .text_color(t.muted)
                .cursor_pointer()
                .hover(move |s| s.bg(t.plate_soft).text_color(t.ink))
                .child(tr("Manage accounts"))
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.account_menu = false;
                    this.go(Page::Settings);
                    cx.notify();
                })),
        );
        Some(gpui::deferred(menu).with_priority(1))
    }
}

/// How an account signs in, for the chip under the player's name.
fn kind(provider: Option<account::Provider>) -> &'static str {
    match provider {
        Some(account::Provider::ElyBy) => "ely.by account",
        Some(account::Provider::Microsoft) => "Microsoft account",
        Some(account::Provider::Offline) => "Offline account",
        None => "Account",
    }
}

/// How an account signs in, short enough for a menu row.
fn short(provider: Option<account::Provider>) -> &'static str {
    match provider {
        Some(account::Provider::ElyBy) => "ely.by",
        Some(account::Provider::Microsoft) => "Microsoft",
        Some(account::Provider::Offline) => "Offline",
        None => "Account",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_service_has_its_own_name() {
        assert_eq!(kind(Some(account::Provider::ElyBy)), "ely.by account");
        assert_eq!(
            kind(Some(account::Provider::Microsoft)),
            "Microsoft account"
        );
        assert_eq!(kind(Some(account::Provider::Offline)), "Offline account");
        assert_eq!(kind(None), "Account");
        assert_eq!(short(Some(account::Provider::ElyBy)), "ely.by");
        assert_eq!(short(Some(account::Provider::Microsoft)), "Microsoft");
        assert_eq!(short(Some(account::Provider::Offline)), "Offline");
        assert_eq!(short(None), "Account");
    }
}
