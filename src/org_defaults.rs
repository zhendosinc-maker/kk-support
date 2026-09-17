//! Organization server defaults for the branded attended-support client.
//!
//! Pure decision logic, no external dependencies: this file must compile
//! standalone (`rustc --test src/org_defaults.rs`) so it can be unit-tested
//! in CI without the hbb_common crate.
//!
//! Behavior:
//! - Applied exactly once per machine (guarded by a marker option).
//! - Fills only EMPTY options; user-entered values are never overwritten.
//! - If an ID server is already configured from any other source, nothing is
//!   applied at all (no mixing of someone else's config with our key).
//! - A user can later change or CLEAR any value; the marker prevents the
//!   defaults from ever being re-applied on subsequent launches.
//!
//! The caller (core_main) persists the returned pairs plus the marker via
//! Config::set_option.

pub const ORG_DEFAULTS_MARKER: &str = "org-defaults-applied";
pub const ORG_ID_SERVER: &str = "rustdesk.zhendosinc.ru";
pub const ORG_RELAY_SERVER: &str = "rustdesk.zhendosinc.ru:21117";
pub const ORG_KEY: &str = "sb+BGxi80QpiMzSRuAAUzp1ulW0LJjLdu+6FWja4dB8=";

/// Returns the option values to write, or an empty Vec if the defaults must
/// not be applied (already applied once, or an ID server is already
/// configured from another source). `options` maps option name -> value.
pub fn decide_defaults(
    options: &std::collections::HashMap<String, String>,
) -> Vec<(&'static str, String)> {
    // Marker present (any value) means: never apply again.
    if options.contains_key(ORG_DEFAULTS_MARKER) {
        return vec![];
    }
    // Respect any pre-existing custom server configuration: do not mix it
    // with our key.
    let absent = String::new();
    if !options
        .get("custom-rendezvous-server")
        .unwrap_or(&absent)
        .trim()
        .is_empty()
    {
        return vec![];
    }
    let candidates: [(&'static str, &str); 3] = [
        ("custom-rendezvous-server", ORG_ID_SERVER),
        ("relay-server", ORG_RELAY_SERVER),
        ("key", ORG_KEY),
    ];
    let mut out = Vec::new();
    for (name, default) in candidates {
        let empty = String::new();
        let current = options.get(name).unwrap_or(&empty);
        if current.trim().is_empty() {
            out.push((name, default.to_string()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn opts(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn clean_profile_gets_all_defaults() {
        let out = decide_defaults(&opts(&[]));
        assert_eq!(
            out,
            vec![
                ("custom-rendezvous-server", ORG_ID_SERVER.to_string()),
                ("relay-server", ORG_RELAY_SERVER.to_string()),
                ("key", ORG_KEY.to_string()),
            ]
        );
    }

    #[test]
    fn second_run_never_reapplies() {
        let o = opts(&[
            ("custom-rendezvous-server", ORG_ID_SERVER),
            ("relay-server", ORG_RELAY_SERVER),
            ("key", ORG_KEY),
            (ORG_DEFAULTS_MARKER, "1"),
        ]);
        assert!(decide_defaults(&o).is_empty());
    }

    #[test]
    fn marker_even_empty_blocks() {
        let o = opts(&[(ORG_DEFAULTS_MARKER, "")]);
        assert!(decide_defaults(&o).is_empty());
    }

    #[test]
    fn existing_custom_server_blocks_defaults() {
        let o = opts(&[
            ("custom-rendezvous-server", "other.example.com"),
            ("key", "userkey"),
        ]);
        assert!(decide_defaults(&o).is_empty());
    }

    #[test]
    fn user_cleared_values_not_refilled() {
        let o = opts(&[
            ("custom-rendezvous-server", ORG_ID_SERVER),
            (ORG_DEFAULTS_MARKER, "1"),
            ("key", ""),
        ]);
        assert!(decide_defaults(&o).is_empty());
    }

    #[test]
    fn nonempty_field_not_overwritten() {
        let o = opts(&[("relay-server", "myrelay:1")]);
        let out = decide_defaults(&o);
        assert!(!out.iter().any(|(k, _)| *k == "relay-server"));
        assert!(out.contains(&("custom-rendezvous-server", ORG_ID_SERVER.to_string())));
        assert!(out.contains(&("key", ORG_KEY.to_string())));
    }

    #[test]
    fn whitespace_only_counts_as_empty() {
        let o = opts(&[("key", "   ")]);
        let out = decide_defaults(&o);
        assert!(out.contains(&("key", ORG_KEY.to_string())));
    }

    #[test]
    fn api_never_touched() {
        let out = decide_defaults(&opts(&[]));
        assert!(out.iter().all(|(k, _)| *k != "api-server"));
    }
}
