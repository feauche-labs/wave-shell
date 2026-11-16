//! Input validation helpers.

use crate::{Result, StoreError, KNOWN_FLAGS};

fn bad<T>(msg: &str) -> Result<T> {
    Err(StoreError::Validation(msg.to_string()))
}

/// Name / label: 1..=128 chars, not blank.
pub(crate) fn name(field: &str, v: &str) -> Result<()> {
    let n = v.chars().count();
    if n == 0 || n > 128 || v.trim().is_empty() {
        return bad(&format!("{field}: must be 1..=128 characters"));
    }
    Ok(())
}

/// Hostname: non-empty, no whitespace, no brackets.
pub(crate) fn hostname(v: &str) -> Result<()> {
    if v.is_empty()
        || v.len() > 255
        || v.chars()
            .any(|c| c.is_whitespace() || c == '[' || c == ']' || c.is_control())
    {
        return bad("hostname: invalid");
    }
    Ok(())
}

/// Port 1..=65535 (0 is rejected).
pub(crate) fn port(field: &str, v: Option<u16>) -> Result<()> {
    if v == Some(0) {
        return bad(&format!("{field}: must be 1..=65535"));
    }
    Ok(())
}

/// Lowercases and checks a flag code. When `KNOWN_FLAGS` is still the empty placeholder only the
/// shape (2..=6 ASCII letters) is checked.
pub(crate) fn country(v: Option<String>) -> Result<Option<String>> {
    let Some(v) = v else { return Ok(None) };
    // Wave: the same field also holds an icon or an emoji instead of a flag (`icon:server`, `emoji:🏠`).
    if let Some(look) = wave_look(&v) {
        return look.map(Some);
    }
    let v = v.to_ascii_lowercase();
    let shape = (2..=6).contains(&v.len()) && v.bytes().all(|b| b.is_ascii_lowercase());
    if !shape || (!KNOWN_FLAGS.is_empty() && !KNOWN_FLAGS.contains(&v.as_str())) {
        return bad("country_code: unknown flag");
    }
    Ok(Some(v))
}

/// Wave: `icon:<id>` (lowercase letters, digits, dashes) or `emoji:<one emoji>`; None = not a look.
fn wave_look(v: &str) -> Option<Result<String>> {
    if let Some(id) = v.strip_prefix("icon:") {
        let ok = (1..=40).contains(&id.len())
            && id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
        return Some(if ok {
            Ok(v.to_owned())
        } else {
            bad("country_code: bad icon")
        });
    }
    if let Some(e) = v.strip_prefix("emoji:") {
        let n = e.chars().count();
        let ok = (1..=12).contains(&n)
            && !e
                .chars()
                .any(|c| c.is_control() || c.is_whitespace() || c.is_ascii());
        return Some(if ok {
            Ok(v.to_owned())
        } else {
            bad("country_code: bad emoji")
        });
    }
    None
}

#[cfg(test)]
mod wave_look_tests {
    use super::country;

    #[test]
    fn icons_and_emoji_pass_flags_still_checked() {
        assert_eq!(
            country(Some("icon:server-cog".into())).unwrap().as_deref(),
            Some("icon:server-cog")
        );
        assert_eq!(
            country(Some("emoji:🏠".into())).unwrap().as_deref(),
            Some("emoji:🏠")
        );
        assert_eq!(
            country(Some("emoji:🛡️".into())).unwrap().as_deref(),
            Some("emoji:🛡️")
        );
        assert_eq!(country(Some("DE".into())).unwrap().as_deref(), Some("de"));
        for bad in [
            "icon:",
            "icon:Server",
            "icon:a b",
            "emoji:",
            "emoji:abc",
            "emoji:🏠 x",
            "zz9",
        ] {
            assert!(country(Some(bad.into())).is_err(), "{bad}");
        }
    }
}
