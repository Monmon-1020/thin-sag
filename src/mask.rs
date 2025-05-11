use dashmap::DashSet;
use once_cell::sync::Lazy;
use regex::Regex;
use sha2::{Digest, Sha256};

static MASK_CACHE: Lazy<DashSet<[u8; 32]>> = Lazy::new(DashSet::new);

static MAIL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap());
static PHONE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\d{2,4}[- ]?\d{2,4}[- ]?\d{3,4}\b").unwrap());
static CC_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b").unwrap());

pub fn cache_secret(s: &str) {
    let hash = Sha256::digest(s.as_bytes());
    MASK_CACHE.insert(hash.into());
}

pub fn mask_text(input: &str) -> String {
    let hash = Sha256::digest(input.as_bytes());
    if MASK_CACHE.contains(&(Into::<[u8; 32]>::into(hash))) {
        return "***MASK***".into();
    }
    let mut out = input.to_owned();
    for re in [&*MAIL_RE, &*PHONE_RE, &*CC_RE] {
        out = re.replace_all(&out, "***MASK***").into_owned();
    }
    out
}
