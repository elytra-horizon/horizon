//! ely.by accounts (docs/elyby-account.md): OAuth2 in the player's browser, then the account's
//! profile and textures. The access token is the Minecraft session token: ely.by's
//! `minecraft_server_session` scope lets the game join online-mode servers that use ely.by
//! (authlib-injector or the skin system). Tokens stay in the keychain with the Microsoft ones;
//! nothing here is logged.
use super::*;
use base64::Engine;
use sha2::{Digest, Sha256};

/// What an ely.by sign-in says when its refresh token or profile no longer works.
pub(super) const ELY_SIGN_IN_AGAIN: &str = "The ely.by sign-in expired. Sign in again.";

/// The scopes the launcher asks for: who the player is, a refresh token, and the Minecraft
/// session token itself.
pub(super) const SCOPE: &str = "account_info offline_access minecraft_server_session";

/// An OAuth2 token response.
pub(super) struct Tokens {
    pub access: String,
    /// Absent when a refresh token is exchanged: the old one stays.
    pub refresh: Option<String>,
    pub expires_at: u64,
}

impl std::fmt::Debug for Tokens {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ElyTokens(<hidden>)")
    }
}

/// The page the player is sent to: `https://account.ely.by/oauth2/v1?…`.
pub(super) fn authorize_url(
    ep: &Endpoints,
    client_id: &str,
    redirect_uri: &str,
    state: &str,
    code_challenge: &str,
) -> String {
    format!(
        "{}/oauth2/v1?client_id={}&redirect_uri={}&response_type=code&scope={}&state={}&code_challenge={}&code_challenge_method=S256",
        ep.ely_account,
        encode(client_id),
        encode(redirect_uri),
        encode(SCOPE),
        encode(state),
        encode(code_challenge),
    )
}

/// A random `state`: the OAuth2 round trip is only accepted back with it.
pub(super) fn random_state() -> String {
    let mut bytes = [0u8; 16];
    let _ = getrandom::fill(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// A PKCE pair (RFC 7636): 32 random bytes make the verifier, its SHA-256 the challenge. ely.by's
/// applications are public clients with no secret, so the verifier is what ties the code back to
/// this sign-in.
pub(super) fn pkce() -> (String, String) {
    let mut bytes = [0u8; 32];
    let _ = getrandom::fill(&mut bytes);
    let verifier = b64url(bytes);
    let challenge = challenge_of(&verifier);
    (verifier, challenge)
}

/// `BASE64URL(SHA256(verifier))`, as PKCE's `S256` method defines it.
fn challenge_of(verifier: &str) -> String {
    b64url(Sha256::digest(verifier.as_bytes()))
}

fn b64url(bytes: impl AsRef<[u8]>) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

/// `…/api/oauth2/v1/token` for an authorization code, with the PKCE verifier that matches the
/// challenge the browser carried. A secret is only sent for a confidential application.
pub(super) fn exchange(
    ep: &Endpoints,
    client_id: &str,
    client_secret: Option<&str>,
    redirect_uri: &str,
    code: &str,
    code_verifier: &str,
) -> Result<Tokens, String> {
    let (status, v) = send(
        request_to("POST", &format!("{}/api/oauth2/v1/token", ep.ely_account)),
        Some(Body::Form(exchange_form(
            client_id,
            client_secret,
            redirect_uri,
            code,
            code_verifier,
        ))),
        "ely.by",
    )?;
    tokens(status, &v, None)
}

fn exchange_form<'a>(
    client_id: &'a str,
    client_secret: Option<&'a str>,
    redirect_uri: &'a str,
    code: &'a str,
    code_verifier: &'a str,
) -> Vec<(&'a str, &'a str)> {
    let mut form = vec![
        ("client_id", client_id),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
        ("code", code),
        ("code_verifier", code_verifier),
    ];
    if let Some(secret) = client_secret {
        form.push(("client_secret", secret));
    }
    form
}

/// `…/api/oauth2/v1/token` with the refresh token, asked for the same scopes.
pub(super) fn refresh(
    ep: &Endpoints,
    client_id: &str,
    client_secret: Option<&str>,
    refresh_token: &str,
) -> Result<Tokens, String> {
    let (status, v) = send(
        request_to("POST", &format!("{}/api/oauth2/v1/token", ep.ely_account)),
        Some(Body::Form(refresh_form(
            client_id,
            client_secret,
            refresh_token,
        ))),
        "ely.by",
    )?;
    tokens(status, &v, Some(refresh_token.to_owned()))
}

fn refresh_form<'a>(
    client_id: &'a str,
    client_secret: Option<&'a str>,
    refresh_token: &'a str,
) -> Vec<(&'a str, &'a str)> {
    let mut form = vec![
        ("client_id", client_id),
        ("scope", SCOPE),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];
    if let Some(secret) = client_secret {
        form.push(("client_secret", secret));
    }
    form
}

fn tokens(status: u16, v: &Value, kept_refresh: Option<String>) -> Result<Tokens, String> {
    let Some(access) = str_field(v, "access_token").filter(|_| status == 200) else {
        return Err(error(v));
    };
    let refresh = str_field(v, "refresh_token").or(kept_refresh);
    let expires_in = v
        .get("expires_in")
        .and_then(Value::as_u64)
        .unwrap_or(86_400)
        .clamp(60, 7 * 86_400);
    Ok(Tokens {
        access,
        refresh,
        expires_at: now().saturating_add(expires_in),
    })
}

/// The OAuth2 error codes as messages for the player.
fn error(v: &Value) -> String {
    let name = v.get("error").and_then(Value::as_str);
    let detail = v.get("error_description").and_then(Value::as_str);
    match name {
        Some("invalid_client") => {
            "ely.by refused this launcher's application registration (client ID or secret).".into()
        }
        Some("invalid_grant") => ELY_SIGN_IN_AGAIN.into(),
        Some("invalid_scope") => "ely.by refused the permissions this launcher asks for.".into(),
        Some("invalid_request") => match detail {
            Some(d) => format!("ely.by sign-in failed: {d}"),
            None => "ely.by sign-in failed (invalid request).".into(),
        },
        Some(other) => match detail {
            Some(d) => format!("ely.by sign-in failed ({other}): {d}"),
            None => format!("ely.by sign-in failed ({other})."),
        },
        None => "ely.by sign-in failed.".into(),
    }
}

// ----- The browser round trip ------------------------------------------------------------------

/// How long a sign-in waits at the loopback before it gives up.
const BROWSER_WAIT: Duration = Duration::from_secs(15 * 60);

/// Listen on the redirect URI's loopback until the browser comes back with the code. The page
/// answers with a short "you can close this" and every request that isn't the redirect (a
/// favicon, a probe) is refused and waited past.
pub(super) fn wait_for_code(
    redirect_uri: &str,
    expected_state: &str,
    cancelled: &dyn Fn() -> bool,
) -> Result<String, String> {
    let (port, path) = listen_address(redirect_uri)?;
    let listener = std::net::TcpListener::bind(("127.0.0.1", port))
        .map_err(|e| format!("Could not wait for the browser at 127.0.0.1:{port}: {e}"))?;
    listener.set_nonblocking(true).map_err(|e| e.to_string())?;
    let deadline = std::time::Instant::now() + BROWSER_WAIT;
    loop {
        if cancelled() {
            return Err("Sign-in cancelled.".into());
        }
        if std::time::Instant::now() >= deadline {
            return Err("The ely.by sign-in timed out. Start again.".into());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let _ = stream.set_nonblocking(false);
                let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                let request = read_request(&mut stream);
                // "GET /oauth/ely?code=… HTTP/1.1": the method, then the target.
                let Some(target) = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                else {
                    respond(&mut stream, 400, "Bad request");
                    continue;
                };
                let (route, query) = target.split_once('?').unwrap_or((target, ""));
                if route != path {
                    respond(&mut stream, 404, "Not found");
                    continue;
                }
                let params = parse_query(query);
                if let Some(error) = params.get("error") {
                    let message = params.get("error_message").cloned().unwrap_or_else(|| {
                        if error == "access_denied" {
                            "Sign-in was declined in the browser.".into()
                        } else {
                            format!("ely.by sign-in failed ({error}).")
                        }
                    });
                    respond(&mut stream, 200, DONE_PAGE);
                    return Err(message);
                }
                if params.get("state").map(String::as_str) != Some(expected_state) {
                    respond(
                        &mut stream,
                        400,
                        "The sign-in didn't come back from ely.by.",
                    );
                    return Err(
                        "The browser answered with the wrong sign-in state. Start again.".into(),
                    );
                }
                match params.get("code") {
                    Some(code) => {
                        respond(&mut stream, 200, DONE_PAGE);
                        return Ok((*code).to_owned());
                    }
                    None => {
                        respond(&mut stream, 400, "The sign-in came back without a code.");
                        return Err(
                            "The browser answered without a sign-in code. Start again.".into()
                        );
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => return Err(format!("Could not wait for the browser: {e}")),
        }
    }
}

const DONE_PAGE: &str =
    "<!doctype html><meta charset=\"utf-8\"><title>Elytra Horizon</title><body \
style=\"font-family:sans-serif;background:#100a1f;color:#fff;text-align:center;padding-top:20vh\">\
<h1>Signed in to ely.by</h1><p>You can close this window and return to Elytra Horizon.</p></body>";

fn respond(stream: &mut std::net::TcpStream, status: u16, body: &str) {
    let _ = std::io::Write::write_all(
        stream,
        format!(
            "HTTP/1.1 {status} X\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .as_bytes(),
    );
}

fn read_request(stream: &mut std::net::TcpStream) -> String {
    use std::io::Read;
    let mut buf = [0u8; 8192];
    let mut read = 0;
    while read < buf.len() {
        match stream.read(&mut buf[read..]) {
            Ok(0) => break,
            Ok(n) => {
                read += n;
                if buf[..read].windows(4).any(|w| w == b"\r\n\r\n") {
                    break;
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => break,
        }
    }
    String::from_utf8_lossy(&buf[..read]).into_owned()
}

fn listen_address(redirect_uri: &str) -> Result<(u16, String), String> {
    let rest = redirect_uri
        .strip_prefix("http://")
        .ok_or("The ely.by redirect must be a plain http loopback address.")?;
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let (host, port) = authority
        .rsplit_once(':')
        .ok_or("The ely.by redirect needs an explicit port.")?;
    if host != "127.0.0.1" && host != "localhost" {
        return Err("The ely.by redirect must point at this computer (127.0.0.1).".into());
    }
    let port = port
        .parse::<u16>()
        .map_err(|_| "The ely.by redirect's port is not a number.".to_string())?;
    Ok((port, format!("/{path}")))
}

fn parse_query(query: &str) -> std::collections::HashMap<String, String> {
    query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|pair| {
            let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
            (decode(k), decode(v))
        })
        .collect()
}

fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            b' ' => out.push_str("%20"),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or("");
                match u8::from_str_radix(hex, 16) {
                    Ok(byte) => {
                        out.push(byte);
                        i += 3;
                    }
                    Err(_) => {
                        out.push(b'%');
                        i += 1;
                    }
                }
            }
            byte => {
                out.push(byte);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

// ----- The profile -----------------------------------------------------------------------------

/// The account's profile: ely.by's account API names it, the session server's textures dress it.
pub(super) fn profile(ep: &Endpoints, token: &str) -> Result<Account, String> {
    let (status, v) = send(
        bearer(
            request_to("GET", &format!("{}/api/account/v1/info", ep.ely_account)),
            token,
        ),
        None,
        "ely.by",
    )?;
    if status == 401 || status == 403 {
        return Err(ELY_SIGN_IN_AGAIN.into());
    }
    let (Some(uuid), Some(name)) = (
        str_field(&v, "uuid").filter(|_| status == 200),
        str_field(&v, "username"),
    ) else {
        return Err("Could not read the ely.by profile.".into());
    };
    let uuid = uuid.replace('-', "").to_lowercase();
    let (skin, cape) = textures(ep, &uuid);
    Ok(Account {
        uuid,
        name,
        skin,
        cape,
    })
}

/// The textures of the session server's profile answer: the skin's URL and model, the cape's URL.
/// The answer is unsigned here (it is only drawn); ely.by's own domains are the only ones taken.
fn textures(ep: &Endpoints, uuid: &str) -> (Option<(String, String)>, Option<String>) {
    let Ok((200, v)) = send(
        request_to("GET", &format!("{}/session/profile/{uuid}", ep.ely_auth)),
        None,
        "ely.by",
    ) else {
        return (None, None);
    };
    packed_textures(&v, ely_url)
}

/// The skin and cape in a session server's `/session/profile/{uuid}` answer, taking only texture
/// URLs `allowed` accepts. Shared with offline accounts, which look a name up on ely.by and
/// Mojang for the skin a player of that name wears.
pub(super) fn packed_textures(
    v: &Value,
    allowed: impl Fn(&str) -> bool,
) -> (Option<(String, String)>, Option<String>) {
    let Some(value) = v
        .get("properties")
        .and_then(Value::as_array)
        .and_then(|all| {
            all.iter()
                .find(|p| p.get("name").and_then(Value::as_str) == Some("textures"))
        })
        .and_then(|p| p.get("value"))
        .and_then(Value::as_str)
    else {
        return (None, None);
    };
    let Ok(packed) = base64::engine::general_purpose::STANDARD.decode(value) else {
        return (None, None);
    };
    let Ok(textures) = serde_json::from_slice::<Value>(&packed) else {
        return (None, None);
    };
    let texture = |kind: &str| {
        textures
            .pointer(&format!("/textures/{kind}/url"))
            .and_then(Value::as_str)
            .filter(|u| allowed(u))
    };
    let skin = texture("SKIN").map(|url| {
        let slim = textures
            .pointer("/textures/SKIN/metadata/model")
            .and_then(Value::as_str)
            .is_some_and(|m| m.eq_ignore_ascii_case("slim"));
        (
            url.to_owned(),
            if slim { "slim" } else { "classic" }.to_owned(),
        )
    });
    (skin, texture("CAPE").map(str::to_owned))
}

/// Whether `url` is on ely.by or under it (the `skinDomains` of its authlib-injector document).
pub(super) fn ely_url(url: &str) -> bool {
    let rest = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    let Some(rest) = rest else {
        return false;
    };
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .split('@')
        .next_back()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");
    host == "ely.by" || host.ends_with(".ely.by")
}

/// The account and its secrets from a finished sign-in.
pub(super) fn finish(ep: &Endpoints, tokens: Tokens) -> Result<(Account, Secrets), String> {
    let account = profile(ep, &tokens.access)?;
    let secrets = Secrets {
        refresh_token: String::new(),
        sign_in_required: false,
        minecraft_token: tokens.access,
        minecraft_expires_at: tokens.expires_at,
        xuid: None,
        owns: true,
        provider: Provider::ElyBy,
        ely_refresh: tokens.refresh,
    };
    Ok((account, secrets))
}

/// [`super::check`] for an ely.by account: the OAuth2 token is renewed near its end, then the
/// profile is asked for again (the name and skin may have changed). No ownership: ely.by accounts
/// are the account, and there is no Mojang entitlement to ask about.
pub(super) fn check(
    ep: &Endpoints,
    account: &Account,
    mut secrets: Secrets,
    force: bool,
    progress: &dyn Fn(session::Phase),
) -> Check {
    if secrets.sign_in_required {
        return Check::NoAnswer(ELY_SIGN_IN_AGAIN.into(), secrets);
    }
    if force || secrets.minecraft_expires_at <= now() + RENEW_BEFORE {
        progress(session::Phase::ElyBy);
        let renewed = (|| {
            let app = ely_client().ok_or_else(|| ELY_NOT_CONFIGURED.to_string())?;
            let refresh_token = secrets
                .ely_refresh
                .clone()
                .ok_or_else(|| ELY_SIGN_IN_AGAIN.to_string())?;
            refresh(
                ep,
                &app.client_id,
                app.client_secret.as_deref(),
                &refresh_token,
            )
        })();
        match renewed {
            Ok(tokens) => {
                secrets.minecraft_token = tokens.access;
                secrets.minecraft_expires_at = tokens.expires_at;
                if let Some(refresh) = tokens.refresh {
                    secrets.ely_refresh = Some(refresh);
                }
            }
            Err(e) => {
                if e == ELY_SIGN_IN_AGAIN {
                    secrets.sign_in_required = true;
                }
                return Check::NoAnswer(e, secrets);
            }
        }
    }
    progress(session::Phase::Profile);
    match profile(ep, &secrets.minecraft_token) {
        Ok(profile) if profile.uuid == account.uuid => Check::Owns(
            profile,
            Secrets {
                owns: true,
                ..secrets
            },
        ),
        Ok(_) => Check::NoAnswer(
            OTHER_PROFILE.into(),
            Secrets {
                sign_in_required: true,
                ..secrets
            },
        ),
        Err(e) => Check::NoAnswer(e, secrets),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_ely_domains_are_texture_hosts() {
        assert!(ely_url("http://ely.by/storage/skins/abc.png"));
        assert!(ely_url("https://skinsystem.ely.by/textures/abc"));
        assert!(!ely_url("https://evil.example/abc.png"));
        assert!(!ely_url("https://ely.by.evil.example/abc.png"));
        assert!(!ely_url("ftp://ely.by/abc.png"));
    }

    #[test]
    fn the_authorize_url_carries_every_parameter() {
        let ep = Endpoints::live();
        let url = authorize_url(
            &ep,
            "id",
            "http://127.0.0.1:1/oauth/ely",
            "state",
            "challenge",
        );
        assert!(url.starts_with("https://account.ely.by/oauth2/v1?"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("scope=account_info%20offline_access%20minecraft_server_session"));
        assert!(url.contains("state=state"));
        assert!(url.contains("code_challenge=challenge"));
        assert!(url.contains("code_challenge_method=S256"));
    }

    #[test]
    fn the_pkce_challenge_follows_rfc_7636() {
        // The worked example in RFC 7636, appendix B.
        assert_eq!(
            challenge_of("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
        let (verifier, challenge) = pkce();
        assert_eq!(verifier.len(), 43, "32 bytes, base64url without padding");
        assert_eq!(challenge, challenge_of(&verifier));
    }

    #[test]
    fn a_public_client_sends_no_secret() {
        let exchange = exchange_form(
            "id",
            None,
            "http://127.0.0.1:1/oauth/ely",
            "code",
            "verifier",
        );
        assert!(exchange.iter().all(|(key, _)| *key != "client_secret"));
        assert_eq!(
            exchange.iter().find(|(key, _)| *key == "code_verifier"),
            Some(&("code_verifier", "verifier"))
        );
        let refresh = refresh_form("id", None, "refresh");
        assert!(refresh.iter().all(|(key, _)| *key != "client_secret"));
        // A confidential application still sends the secret the player configured.
        let confidential = refresh_form("id", Some("secret"), "refresh");
        assert_eq!(
            confidential.iter().find(|(key, _)| *key == "client_secret"),
            Some(&("client_secret", "secret"))
        );
    }

    #[test]
    fn the_browser_round_trip_answers_its_path() {
        use std::io::{Read, Write};
        // A free port, then the launcher binds it.
        let probe = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = probe.local_addr().unwrap().port();
        drop(probe);
        let redirect = format!("http://127.0.0.1:{port}/oauth/ely");
        let browser = std::thread::spawn(move || {
            let mut stream = None;
            for _ in 0..200 {
                match std::net::TcpStream::connect(("127.0.0.1", port)) {
                    Ok(s) => {
                        stream = Some(s);
                        break;
                    }
                    Err(_) => std::thread::sleep(Duration::from_millis(10)),
                }
            }
            let mut stream = stream.expect("the launcher listens");
            stream
                .write_all(
                    b"GET /oauth/ely?code=abc&state=state HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n",
                )
                .unwrap();
            let mut page = String::new();
            stream.read_to_string(&mut page).unwrap();
            page
        });
        let code = wait_for_code(&redirect, "state", &|| false).unwrap();
        assert_eq!(code, "abc");
        let page = browser.join().unwrap();
        assert!(page.starts_with("HTTP/1.1 200"), "{page}");
        assert!(page.contains("Signed in to ely.by"), "{page}");
    }

    #[test]
    fn query_values_are_decoded() {
        let q = parse_query("code=a%20b&state=x&error=access_denied");
        assert_eq!(q["code"], "a b");
        assert_eq!(q["state"], "x");
        assert_eq!(q["error"], "access_denied");
    }
}
