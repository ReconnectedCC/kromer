use once_cell::sync::Lazy;
use regex::Regex;

static ADDRESS_RE_V2: Lazy<Regex> = Lazy::new(|| Regex::new(r"^k[a-z0-9]{9}$").unwrap());
static ADDRESS_LIST_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:k[a-z0-9]{9}|[a-f0-9]{10})(?:,(?:k[a-z0-9]{9}|[a-f0-9]{10}))*$").unwrap()
});
static NAME_FETCH_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:xn--)?[a-z0-9-_]{1,64}$").unwrap());
static NAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[a-z0-9_-]{1,64}$").unwrap());
static NAME_A_RECORD_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^\s.?#].[^\s]*$").unwrap());
static _NAME_META_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(?:([a-z0-9-_]{1,32})@)?([a-z0-9]{1,64})\.kst$").unwrap());

#[inline(always)]
pub fn is_valid_name(name: &str, fetching: bool) -> bool {
    let name = name.to_lowercase();

    // Bit silly to do it like this but yolo lol
    match fetching {
        true => NAME_FETCH_RE.is_match(&name) && !name.is_empty() && name.len() <= 64,
        false => NAME_RE.is_match(&name) && !name.is_empty() && name.len() <= 64,
    }
}

#[inline(always)]
pub fn is_valid_kromer_address(address: &str) -> bool {
    ADDRESS_RE_V2.is_match(address)
}

#[inline(always)]
pub fn is_valid_kromer_address_list(address_list: &str) -> bool {
    ADDRESS_LIST_RE.is_match(address_list)
}

#[inline(always)]
pub fn is_valid_a_record(a: &str) -> bool {
    !a.is_empty() && a.len() <= 255 && NAME_A_RECORD_RE.is_match(a)
}

#[inline(always)]
pub fn strip_name_suffix(name: &str) -> String {
    name.replace(r"\.kst$", "")
}
