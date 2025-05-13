// mask.rs
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use std::sync::RwLock;

static SECRET_LITERALS: Lazy<RwLock<HashSet<String>>> = Lazy::new(|| RwLock::new(HashSet::new()));

static PII_REGEX: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // e‑mail
        Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Za-z]{2,}").unwrap(),
        // tel (simple JP / intl)
        Regex::new(r"\\+?\\d[\\d\\-]{8,}\\d").unwrap(),
    ]
});

pub fn register_secret(raw: &str) {
    let mut guard = SECRET_LITERALS.write().unwrap();
    guard.insert(raw.to_string());
}

pub fn mask_text<S: AsRef<str>>(input: S) -> String {
    let s = input.as_ref();

    {
        let guard = SECRET_LITERALS.read().unwrap();
        if guard.contains(s) {
            return "***MASK***".into();
        }
    }

    for re in PII_REGEX.iter() {
        if re.is_match(s) {
            return re.replace_all(s, "***MASK***").into_owned();
        }
    }

    s.to_string()
}
