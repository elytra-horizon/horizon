# Offline accounts

Horizon's launcher plays with an ely.by account ([elyby-account.md](elyby-account.md)) **or an
offline account**: a player name and nothing else. An offline account has no sign-in, no
token and no ownership check. It plays local worlds and offline-mode servers; online-mode servers
refuse it.

## What an offline account is

- **The name follows Mojang's rule**: 3 to 16 letters, digits or `_`
  (`account::offline::validate_name`).
- **The UUID is the game's offline UUID**:
  `UUID.nameUUIDFromBytes("OfflinePlayer:<name>")` — MD5 with the version 3 and IETF variant bits,
  as 32 hex digits. A server that runs in offline mode computes the same UUID for the same name,
  so worlds and player data line up with vanilla and other launchers.
- **The secrets** are a `Secrets` item with `provider: "offline"`, `owns: true`, the token `"0"`
  (never a session any service would accept) and no expiry. `check` returns owns without asking
  anything, so Play works with no network.
- **The session file** handed to the game says `user_type: "legacy"` and `provider: "offline"`
  (see `account::write_session_file`).

## The skin

An offline account borrows the skin and cape of a player with the same name:

1. the launcher looks the name up on ely.by (`GET /api/users/profiles/minecraft/{name}`, then
   `GET /session/profile/{uuid}`) and then Mojang (`GET https://api.mojang.com/users/profiles/minecraft/{name}`,
   then `GET https://sessionserver.mojang.com/session/minecraft/profile/{uuid}`);
2. the client does the same lookup on its own (`skins::offline_textures`), so the in-game skin needs
   no temporary file or environment variable from the launcher;
3. if neither service knows the name (or it has no skin), the game draws the UUID's default skin.

Only unsigned lookups by name are made. No token is sent, and the texture URL must be on
`ely.by` or `textures.minecraft.net`.

## How the client treats it

`AuthSession` has `Provider::Offline`. `authenticated()` is false for it, and every online service
is gated on `auth::current_authenticated()`:

- `join_server` returns `JoinError::InvalidSession` (there is no session service);
- the network login sends no session, exactly like an offline name in vanilla, so online-mode
  servers ask Mojang and find nothing;
- Mojang player attributes/chat actions and the profile key pair are skipped;
- cosmetics, social/Realms, LAN hosting status and the title screen's account calls do nothing;
- `expired()` is false: there is no token to run out.

## Where it lives

| Piece | File |
|---|---|
| Creation, name rule, offline UUID, skin lookup | `launcher/src/account/offline.rs` |
| Provider, owner gate, session file | `launcher/src/account.rs` |
| Status without renewal, join refusal | `launcher/src/account/session.rs` |
| Settings' Offline card, account menu | `launcher/src/glass_account.rs`, `launcher/src/frame.rs`, `launcher/src/main.rs` |
| Client provider and gating | `crates/horizon-client-render/src/auth.rs` |
| Skin lookup and texture domains | `crates/horizon-client-render/src/skins.rs` |
| Login without a session | `crates/horizon-client/src/net.rs` |

## Command line

```sh
horizon-launcher account offline <name>   # add it (prints name, uuid and provider)
horizon-launcher account list             # shows "Offline" beside it
horizon-launcher account select <name>    # play as it
```

`account list --json` includes `provider: "offline"`. Sign-in does not apply to an offline
account: there is nothing to sign in to. Add it again with `account offline <name>` to change the
name.

## Limits

- Online-mode servers (Mojang's session server, or ely.by's) refuse an offline account; use
  offline-mode servers.
- Horizon's cosmetics, Realms, play-with-friends hosted sessions and social features need a real
  account.
- The name is unauthenticated: anyone can take any unused name in offline mode. That is the mode's
  nature, not a Horizon choice.
