/*
 * Manticore Search Client
 *
 * Сlient for Manticore Search. 
 *
 * The version of the OpenAPI document: 3.3.1
 * Contact: info@manticoresearch.com
 * Generated by: https://openapi-generator.tech
 */

/// MatchPhraseFilter : Query match expression



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchPhraseFilter {
    #[serde(rename = "query_phrase")]
    pub query_phrase: String,
    #[serde(rename = "query_fields")]
    pub query_fields: String,
}

impl MatchPhraseFilter {
    /// Query match expression
    pub fn new(query_phrase: String, query_fields: String) -> MatchPhraseFilter {
        MatchPhraseFilter {
            query_phrase,
            query_fields,
        }
    }
}


