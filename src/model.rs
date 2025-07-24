pub mod search;

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
pub struct AiSearchQuery {
    pub q: Option<String>,
    pub engine: Option<String>,
    pub language: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct GenerateQueryResult {
    pub url: String, // url to search service
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct UserQueryRequest {
    pub engine: String,
    pub prompt: String,
    pub language: Option<String>,
}
