pub mod arkiv_client;
pub mod response;

pub fn remove_jens_suffix(value: String) -> String {
    if value.ends_with("_EJBOrgUnit") {
        value[..value.len() - "_EJBOrgUnit".len()].to_string()
    } else {
        value
    }
}

pub fn add_jens_suffix(value: String) -> String {
    if value.ends_with("_EJBOrgUnit") {
        value
    } else {
        format!("{value}_EJBOrgUnit")
    }
}
