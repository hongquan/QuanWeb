pub mod html;
pub mod jinja_extra;
pub mod markdown;
pub mod urls;

pub fn split_search_query(query: Option<&str>) -> Option<Vec<&str>> {
    let tokens: Option<Vec<&str>> =
        query.map(|s| s.split_whitespace().filter(|&s| !s.is_empty()).collect());
    tokens.filter(|v| !v.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_search_query_single_word() {
        let query = Some("Linux");
        let expected: Option<Vec<&str>> = Some(vec!["Linux"]);
        assert_eq!(split_search_query(query), expected);
    }

    #[test]
    fn test_split_search_query_multiple_words() {
        let query = Some("Linux kernel");
        let expected: Option<Vec<&str>> = Some(vec!["Linux", "kernel"]);
        assert_eq!(split_search_query(query), expected);
    }
}
