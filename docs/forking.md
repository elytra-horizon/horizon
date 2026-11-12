# Forking: Horizon's extras and the vanilla game

Horizon is the vanilla game plus a few things of its own: cosmetics, Play with Friends, the
official server in the list, its settings pages. In the code they sit behind one Cargo feature,
`horizon-extras`, on by default, so a fork can build without them or delete them. Players always
get the full Horizon: there is no setting, flag or variable that turns the extras off.
Optimizations are not extras; they are part of the game in every build.

## Building without Horizon's extras

```sh
cargo build --release -p horizon-client --no-default-features --features audio,controllers
```

Best effort: CI builds and tests the default only. The server build (`--no-default-features`,
docs/server.md) compiles the same gates, so the code keeps compiling, but nobody plays it as a
client before a release. [horizon-rust/horizon-plain](https://github.com/horizon-rust/horizon-plain)
builds it from each official release tag, unsigned.

What changes in that build:

| Extra | Where it lives | The gate |
|---|---|---|
| Cosmetics, emotes and the emote wheel, "Show me as a Horizon player" | `game/cosmetics.rs` (+ `cosmetics/fx.rs`, `cosmetics/emotes.rs`, `cosmetics/emote_wheel.rs`), crate `horizon-cosmetics` | `game::cosmetics::enabled()`: the service is never asked, nothing is drawn, B opens no wheel |
| The official server added to the server list, the withdrawn partner server taken out; the "Start Horizon from the launcher" screen of a game without its prepared files, with its Download button (`get_the_game` hook) | crate `horizon-official-servers` | registered only by the official executable (`horizon-official`); horizon-client's own executable has none |
| Horizon Settings (Extras, PvP & HUD, Splitscreen, Studio, AI Agent) and Mods (with the registry browser) | `options_ui.rs` (`Page::Horizon` and below), `extras_ui.rs`, `extras.rs`, `pvp.rs`, `game/extras.rs`, `game/tweaks.rs`, `extras_hud.rs` | the Options screen shows vanilla's greyed Telemetry Data instead (`options_ui::HORIZON_EXTRAS`, set in `entry.rs`) |
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
