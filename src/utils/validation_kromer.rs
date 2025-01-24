use regex::Regex;

const ADDRESS_RE_V2: &str = r"^k[a-z0-9]{9}$";
const ADDRESS_LIST_RE: &str = r"^(?:k[a-z0-9]{9}|[a-f0-9]{10})(?:,(?:k[a-z0-9]{9}|[a-f0-9]{10}))*$";
const NAME_FETCH_RE: &str = r"^(?:xn--)?[a-z0-9-_]{1,64}$";
const NAME_RE: &str = r"^[a-z0-9_-]{1,64}$";
const NAME_A_RECORD_RE: &str = r"^[^\s.?#].[^\s]*$";
const _NAME_META_RE: &str = r"^(?:([a-z0-9-_]{1,32})@)?([a-z0-9]{1,64})\.kst$";

pub fn is_valid_name(name: String, fetching: bool) -> bool {
    let re = if fetching { NAME_FETCH_RE } else { NAME_RE };

    // Gonna just unwrap and assume it's okay, just don't break Regex strings pls
    let re = Regex::new(re).unwrap();
    let name = name.to_lowercase();

    re.is_match(&name) && !name.is_empty() && name.len() <= 64
}

pub fn is_valid_kromer_address(address: String) -> bool {
    let re = Regex::new(ADDRESS_RE_V2).unwrap();

    re.is_match(&address)
}

pub fn is_valid_kromer_address_list(address_list: String) -> bool {
    let re = Regex::new(ADDRESS_LIST_RE).unwrap();

    re.is_match(&address_list)
}

pub fn is_valid_a_record(a: String) -> bool {
    let re = Regex::new(NAME_A_RECORD_RE).unwrap();
    !a.is_empty() && a.len() <= 255 && re.is_match(&a)
}

pub fn strip_name_suffix(name: String) -> String {
    name.replace(r"\.kst$", "")
}
