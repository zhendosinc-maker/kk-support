use hbb_common::config::Config;

/// Organization defaults for the branded attended-support client.
///
/// Applied exactly once per machine (guarded by a marker option):
/// - Only fills options that are still EMPTY, so user-entered values are
///   never overwritten.
/// - If an ID server is already configured from any other source, nothing is
///   applied at all (no mixing of someone else's config with our key).
/// - A user can later change or CLEAR any value; the marker prevents the
///   defaults from ever being re-applied on subsequent launches.

const ORG_DEFAULTS_MARKER: &str = "org-defaults-applied";

#[cfg(not(standalone_test))]
pub const ORG_ID_SERVER: &str = "rustdesk.zhendosinc.ru";
#[cfg(not(standalone_test))]
pub const ORG_RELAY_SERVER: &str = "rustdesk.zhendosinc.ru:21117";
#[cfg(not(standalone_test))]
pub const ORG_KEY: &str = "sb+BGxi80QpiMzSRuAAUzp1ulW0LJjLdu+6FWja4dB8=";

#[cfg(standalone_test)]
pub const ORG_ID_SERVER: &str = "rustdesk.zhendosinc.ru";
#[cfg(standalone_test)]
pub const ORG_RELAY_SERVER: &str = "rustdesk.zhendosinc.ru:21117";
#[cfg(standalone_test)]
pub const ORG_KEY: &str = "sb+BGxi80QpiMzSRuAAUzp1ulW0LJjLdu+6FWja4dB8=";

/// Pure decision logic: returns the option values to write, or an empty Vec
/// if the defaults must not be applied (already applied, or an ID server is
/// already configured from another source). `options` maps name -> value.
pub fn decide_defaults(
    options: &std::collections::HashMap<String, String>,
) -> Vec<(&'static str, String)> {
    // Marker present (any value) means: never apply again.
    if options.contains_key(ORG_DEFAULTS_MARKER) {
        return vec![];
    }
    // Respect any pre-existing custom server configuration.
    if !options
        .get("custom-rendezvous-server")
        .unwrap_or(&String::new())
        .trim()
        .is_empty()
    {
        return vec![];
    }
    let mut out = Vec::new();
    let candidates: [(&'static str, &str); 3] = [
        ("custom-rendezvous-server", ORG_ID_SERVER),
        ("relay-server", ORG_RELAY_SERVER),
        ("key", ORG_KEY),
    ];
    for (name, default) in candidates {
        let cur = options.get(name).unwrap_or(&String::new());
        if cur.trim().is_empty() {
            out.push((name, default.to_string()));
        }
    }
    out
}

#[cfg(not(standalone_test))]
/// Applies the org defaults on first run. Safe to call on every startup.
pub fn apply_once() {
    let options = Config::get_options();
    let updates = decide_defaults(&options);
    if updates.is_empty() {
        return;
    }
    for (name, value) in updates {
        Config::set_option(name.to_string(), value);
    }
    Config::set_option(ORG_DEFAULTS_MARKER.to_string(), "1".to_string());
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
        let mut o = opts(&[("custom-rendezvous-server", ORG_ID_SERVER)]);
        o.insert(ORG_DEFAULTS_MARKER.into(), String::new());
        assert!(decide_defaults(&o).is_empty());
    }

    #[test]
    fn existing_custom_server_blocks_defaults() {
        let o = opts(&[("custom-rendezvous-server", "other.example.com"), ("key", "userkey")]);
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
    fn api_never_touched() {
        let out = decide_defaults(&opts(&[]));
        assert!(out.iter().all(|(k, _)| *k != "api-server"));
    }
}
