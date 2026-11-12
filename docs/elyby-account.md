# ely.by accounts and online play

Horizon's launcher signs players in with a Microsoft account (docs/microsoft-account.md) **or** an
ely.by account. ely.by is an alternative Minecraft account service: its name, skin and cape come
along, and it joins online-mode servers that accept ely.by (authlib-injector or the skin system)
instead of Mojang's session server.

## What has to be registered

Like Microsoft's Azure application, ely.by's OAuth2 application belongs to the maintainer:

1. https://account.ely.by/dev/applications/new → **Website**.
2. Redirect URI: `http://127.0.0.1:48632/oauth/ely` (the launcher's loopback; a development build
   can use another port with `HORIZON_ELYBY_REDIRECT_URI`).
3. Copy the **clientId** and **clientSecret**.
4. Put them in the release build's repository variables `HORIZON_ELYBY_CLIENT_ID` and
   `HORIZON_ELYBY_CLIENT_SECRET` (read with `option_env!`), or at run time in the same environment
   variables (empty or unset = ely.by sign-in is not configured).

A desktop application cannot keep a secret; the client secret is only as private as the binary, as
with every launcher that ships one. The redirect is a loopback, so nothing listens off this
computer and no other website can receive the code.

## Flow (launcher, `account/elyby.rs`)

1. **Authorize**: open
   `https://account.ely.by/oauth2/v1?client_id=…&redirect_uri=…&response_type=code&scope=account_info%20offline_access%20minecraft_server_session&state=…`.
   `state` is 16 random bytes; the launcher listens on the redirect's `127.0.0.1` port meanwhile.
2. **Code**: the browser comes back to `http://127.0.0.1:48632/oauth/ely?code=…&state=…`; the
   wrong state, an `error`, or no code ends the sign-in. The page answers "you can close this".
3. **Token**: `POST https://account.ely.by/api/oauth2/v1/token`
   (`client_id`, `client_secret`, `redirect_uri`, `grant_type=authorization_code`, `code`). The
   answer's `access_token` is also the Minecraft session token (the `minecraft_server_session`
   scope), and `refresh_token` renews it (`grant_type=refresh_token` with the same scopes).
4. **Profile**: `GET https://account.ely.by/api/account/v1/info` (Bearer) → uuid and name;
   `GET https://authserver.ely.by/session/profile/{uuid}` → the `textures` property (base64 JSON)
   → skin URL and model, cape URL. Only `ely.by` and its subdomains are taken.
5. **Secrets**: the refresh token is kept in the keychain beside the Microsoft ones
   (`Secrets.ely_refresh`, `Secrets.provider = ElyBy`); `minecraft_token` holds the OAuth access
   token and `minecraft_expires_at` its end.

Renewal (`check`) refreshes the OAuth token when it has less than an hour left, then asks for the
profile again (name and skin may have changed). There is no entitlement check: ely.by accounts are
not Mojang accounts, and the owner gate is satisfied by the account itself.

## Handoff to the game

`write_session_file` writes `user_type: "mojang"` and `provider: "elyby"` for an ely.by account
(Microsoft's stay `"msa"`, no `provider`). The game (`crates/horizon-client-render/src/auth.rs`)
reads the provider and:

- joins online-mode servers with `POST https://authserver.ely.by/session/join` instead of the
  endpoints in Mojang's services discovery document;
- takes its skin and cape from `ely.by` URLs (the authlib-injector `skinDomains`);
- skips Mojang-only calls: player attributes (bans/chat actions) and the profile key pair, so its
  chat is unsigned.

`hasJoined` stays the server's business: a server that accepts ely.by players runs authlib-injector
or the ely.by skin system.

## Limits

- Official Horizon servers and Mojang-authenticated servers do not know ely.by accounts; they can
  only be joined in offline mode (or by servers configured for ely.by).
- `hasJoined` on LAN hosts and the dedicated server still asks Mojang's session server; only the
  native client's connection (and the launcher's `account session` handoff) uses ely.by's.
- Chat signing is off for ely.by accounts (no profile key pair service there).

## Command line

`horizon-launcher account elyby` opens the same page and prints its link (with `--json`,
`{"link": "…"}` then `{"signed_in": "<uuid>"}`). `account list` shows each account's provider,
`account sign-in --for <name>` re-signs an ely.by account in through the same browser flow, and
`account list --json` adds `provider` and `elyby_configured`.
