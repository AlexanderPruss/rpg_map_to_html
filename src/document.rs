pub mod markdown_content;

pub mod html;

/// If the [string] contains the [pattern], replace it by [replace_by]. Otherwise returns 
/// the original string without allocating a new one.
fn replace_if_contains(string: String, pattern: &str, replace_by: &str) -> String{
    if !string.contains(pattern) {
        return string;
    }
    string.replace(pattern, replace_by)
}

#[cfg(test)]
pub mod test {

    pub(crate) mod fixtures;
}
