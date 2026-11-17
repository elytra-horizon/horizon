# Forking: Horizon's extras and this fork

Upstream introduced a `horizon-extras` Cargo feature in
horizon-rust/horizon#12055 to group its cosmetics, emotes, presence, server-list
promotions and settings pages. This fork deliberately differs: excluded features
are removed, not compiled behind a switch.

Cosmetics, wardrobe, emotes, partner ads, sponsor cards, telemetry and Discord
presence are not part of this fork. No Cargo feature restores them. The Mods and
Horizon Settings pages remain available independently of the client's default
features; disabling audio or controllers does not hide mod management.

## Build features

The client's default features are `audio`, `controllers` and `friends`.
Play with Friends remains independently selectable. For example, a client without
audio or controllers can be built with:

```sh
cargo build --release -p horizon-game --no-default-features --features friends
```

The dedicated server uses `--no-default-features`. Neither build mode introduces
a telemetry placeholder or an official-build promotion in the options screen.

## Modding and rendering

| Extra | Where it lives | The gate |
|---|---|---|
| Cosmetics, emotes and the emote wheel, "Show me as a Horizon player" | `game/cosmetics.rs` (+ `cosmetics/fx.rs`, `cosmetics/emotes.rs`, `cosmetics/emote_wheel.rs`), crate `horizon-cosmetics` | `game::cosmetics::enabled()`: the service is never asked, nothing is drawn, B opens no wheel |
| The official server added to the server list, the withdrawn partner server taken out; the "Start Horizon from the launcher" screen of a game without its prepared files, with its Download button (`get_the_game` hook) | crate `horizon-official-servers` | registered only by the official executable (`horizon-official`); horizon-client's own executable has none |
| Horizon Settings (Extras, PvP & HUD, Splitscreen, Studio, AI Agent) and Mods (with the registry browser) | `options_ui.rs` (`Page::Horizon` and below), `extras_ui.rs`, `extras.rs`, `pvp.rs`, `game/extras.rs` and `game/extras/`, `extras_hud.rs` | the Options screen shows vanilla's greyed Telemetry Data instead (`options_ui::HORIZON_EXTRAS`, set in `entry.rs`) |
| Discord Rich Presence and its switch | crate `horizon-discord` | registered only by the official executable (`horizon-official`); horizon-client's own executable has none |
| Play with Friends | hosting: `game/friends.rs`, `game/friends/screen.rs`; joining: crate `horizon-friends` (`game`) | hosting: the `friends` feature (`game/friends_off.rs` stands in); joining: only the official executable registers it ([layers.md](layers.md#how-a-horizon-crate-plugs-in)) |
| The anonymous "I'm playing" ping | crate `horizon-heartbeat` | not in horizon-client at all: only the official executable (`horizon-official`) registers it ([layers.md](layers.md#how-a-horizon-crate-plugs-in)) |

Credits & Attribution then says where the official build is (`horizon.plain.official`).

`grep -rn horizon-extras crates/` lists every gate. Apart from the `friends` feature above, the gates
are `cfg!(feature = "horizon-extras")`: both sides compile in every build, so no stub has to follow
the code it replaces. To remove an extra for good, delete its files from the table and the lines its
gate guards.

## Not behind the feature

- The launcher (`launcher/`) holds the Wardrobe, the Shop, the featured servers page, its themes,
  the telemetry consent and updates. A build of the game alone has none of them; without the
  launcher's consent the game keeps no crash or performance report (docs/telemetry.md).
- The graphics screen's fullbright, dynamic lights and hiding block-break particles: renderer
  options, off by default (docs/extras.md). A `horizon-extras.txt` left by an official install is
  still read.
- The mod runtime: nothing installs a mod without the launcher, so it stays idle.
- The `horizon` brand reply to a Horizon server, for its protocol extensions (`net.rs`).
- Agent control (`--control`, `horizon-ctl`), controllers, HDR, shader packs.

## Accounts in a fork

This fork's launcher signs players in with an ely.by account or an offline name, and does not offer
Microsoft sign-in (docs/accounts.md). ely.by's OAuth2 application belongs to the maintainer: this
fork's is the public desktop application `elytra-horizon1` (docs/elyby-account.md). The upstream
Microsoft token code remains only so accounts signed in before this fork can be refreshed or signed
out; nothing starts a new Microsoft sign-in.

- A fork, port or derived launcher that signs players in with ely.by registers its own application,
  or overrides `HORIZON_ELYBY_CLIENT_ID`, and must not ship, embed or fetch this fork's client ID.
- The MIT licence covers the code, not the app registration: copying the code gives no right to use
  another build's OAuth application.
- The maintainer may restrict, rotate or revoke the application at any time, which breaks every
  build that uses it.

A build can disable ely.by sign-in by setting `HORIZON_ELYBY_CLIENT_ID` empty; it then says that
ely.by is not configured.
