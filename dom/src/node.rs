#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element {
        namespace_uri: Option<String>,
        prefix: Option<String>,
        local_name: String,
        tag_name: String,
    },
}
