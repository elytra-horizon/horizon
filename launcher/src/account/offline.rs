//! Offline accounts (docs/offline-account.md): a player name, and nothing else. No service is
//! asked, no token is kept: the game gets the name and the offline UUID every server makes for
//! it (`OfflinePlayer:<name>`), so it plays offline-mode servers. The name is looked up on
//! ely.by and then Mojang for the skin and cape a player of that name wears; failing that, the
//! game draws the default skin.
use super::*;
use md5::{Digest, Md5};

/// What a name may be: Mojang's player-name rule (3–16 letters, digits or `_`).
pub const NAME_RULE: &str = "Offline names are 3 to 16 letters, digits or underscores.";

/// The player name, trimmed, when it is one the game accepts.
pub fn validate_name(name: &str) -> Result<&str, String> {
    let name = name.trim();
    let count = name.chars().count();
    let shaped =
        (3..=16).contains(&count) && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !shaped {
        return Err(NAME_RULE.into());
    }
    Ok(name)
}

/// The offline UUID of `name`, as `UUID.nameUUIDFromBytes("OfflinePlayer:<name>")` makes it: an
/// MD5 digest with the version 3 and IETF variant bits, in the accounts' 32 hex digits.
pub fn uuid(name: &str) -> String {
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&Md5::digest(format!("OfflinePlayer:{name}").as_bytes()));
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The account and its secrets for `name`: the offline UUID, the game's name, and the skin a
/// player of that name wears if either service knows one. Best effort: no network is not an
/// error, the game just draws the default skin.
pub fn create(ep: &Endpoints, name: &str) -> Result<(Account, Secrets), String> {
    let name = validate_name(name)?.to_owned();
    let uuid = uuid(&name);
    let (skin, cape) = skin(ep, &name);
    Ok((
        Account {
            uuid,
            name,
            skin,
            cape,
        },
        Secrets {
            // No token: the game must never send this anywhere. `0` is what other launchers pass
            // to an offline client, and it is not a session the services would accept.
            refresh_token: String::new(),
            sign_in_required: false,
            minecraft_token: "0".into(),
            // Nothing to renew; the status stays Ready without asking.
            minecraft_expires_at: u64::MAX,
            xuid: None,
            owns: true,
            provider: Provider::Offline,
            ely_refresh: None,
        },
    ))
}

/// The skin and cape of a player named `name`: ely.by first (its skin system is the one this
/// launcher signs in to), then Mojang. Both are unsigned lookups by name.
fn skin(ep: &Endpoints, name: &str) -> Skin {
    elyby_skin(ep, name)
        .or_else(|| mojang_skin(ep, name))
        .unwrap_or((None, None))
}

/// A skin URL and model ("classic" or "slim"), and a cape URL.
type Skin = (Option<(String, String)>, Option<String>);

fn elyby_skin(ep: &Endpoints, name: &str) -> Option<Skin> {
    let (status, v) = send(
        request_to(
            "GET",
            &format!("{}/api/users/profiles/minecraft/{name}", ep.ely_auth),
        ),
        None,
        "ely.by",
    )
    .ok()?;
    if status != 200 {
        return None;
    }
    let uuid = str_field(&v, "id")?.replace('-', "").to_lowercase();
    let (status, profile) = send(
        request_to("GET", &format!("{}/session/profile/{uuid}", ep.ely_auth)),
        None,
        "ely.by",
    )
    .ok()?;
    if status != 200 {
        return None;
    }
    Some(elyby::packed_textures(&profile, elyby::ely_url))
}

fn mojang_skin(ep: &Endpoints, name: &str) -> Option<Skin> {
    let (status, v) = send(
        request_to(
            "GET",
            &format!("{}/users/profiles/minecraft/{name}", ep.mojang),
        ),
        None,
        "Mojang",
    )
    .ok()?;
    if status != 200 {
        return None;
    }
    let uuid = str_field(&v, "id")?.replace('-', "").to_lowercase();
    let (status, profile) = send(
        request_to(
            "GET",
            &format!("{}/session/minecraft/profile/{uuid}", ep.mojang_session),
        ),
        None,
        "Mojang",
    )
    .ok()?;
    if status != 200 {
        return None;
    }
    Some(elyby::packed_textures(&profile, minecraft_url))
}

/// Whether `url` is one of Mojang's public texture URLs (the only host its session server names).
fn minecraft_url(url: &str) -> bool {
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
        .rsplit('@')
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("");
    host == "textures.minecraft.net"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_uuids_match_the_games() {
        // UUID.nameUUIDFromBytes("OfflinePlayer:Notch") in Java.
        assert_eq!(uuid("Notch"), "b50ad385829d3141a2167e7d7539ba7f");
        assert_eq!(uuid("jeb_"), uuid("jeb_"));
        assert_ne!(uuid("Notch"), uuid("notch"));
    }

    #[test]
    fn names_follow_the_games_rule() {
        assert_eq!(validate_name("  Steve "), Ok("Steve"));
        assert!(validate_name("ab").is_err());
        assert!(validate_name("a".repeat(17).as_str()).is_err());
        assert!(validate_name("Steve-2").is_err());
        assert!(validate_name("St ve").is_err());
        // Mojang's rule has no leading-letter requirement: these are names players use.
        assert_eq!(validate_name("_Steve"), Ok("_Steve"));
        assert_eq!(validate_name("123"), Ok("123"));
        assert_eq!(validate_name("a".repeat(16).as_str()).unwrap().len(), 16);
    }
}
