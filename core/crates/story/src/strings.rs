//! The string table (`data/strings.toml`): every proper noun the design borrows, once, so a commercial build renames
//! in one place. Captions and barks name the world through `{token}`s this fills.

use std::collections::BTreeMap;
use std::sync::OnceLock;

const STRINGS_TOML: &str = include_str!("../../../../data/strings.toml");

pub fn table() -> &'static BTreeMap<String, String> {
    static T: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    T.get_or_init(|| {
        let v: toml::Value = toml::from_str(STRINGS_TOML).expect("data/strings.toml");
        let mut m = BTreeMap::new();
        if let Some(t) = v.as_table() {
            for (_, sect) in t {
                if let Some(s) = sect.as_table() {
                    for (k, x) in s {
                        if let Some(x) = x.as_str() {
                            m.insert(k.clone(), x.to_string());
                        }
                    }
                }
            }
        }
        m
    })
}

/// Fill a line's tokens from the string table and `extra` ({name}, {teller}, ...); a line that begins with a token
/// begins with a capital.
pub fn fill(text: &str, extra: &[(&str, &str)]) -> String {
    let mut s = text.to_string();
    for (k, v) in extra {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    for (k, v) in table() {
        s = s.replace(&format!("{{{k}}}"), v);
    }
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => s,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn tokens_fill_from_the_table_and_the_caller() {
        assert_eq!(super::fill("{ferals}. Lost ones.", &[]), "Sewerats. Lost ones.");
        assert_eq!(super::fill("{name} is on the wall.", &[("name", "Kerr")]), "Kerr is on the wall.");
        assert!(super::fill("the {order} came", &[]).contains("Oathkeepers"));
    }
}
