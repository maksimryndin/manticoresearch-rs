/*
 * Manticore Search Client
 *
 * Сlient for Manticore Search.
 *
 * The version of the OpenAPI document: 3.3.1
 * Contact: info@manticoresearch.com
 * Generated by: https://openapi-generator.tech
 */

/// MatchOpFilter : Query match expression

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchOpFilter {
    #[serde(rename = "query_string")]
    pub query_string: String,
    #[serde(rename = "query_fields")]
    pub query_fields: String,
    #[serde(rename = "operator")]
    pub operator: Operator,
}

impl MatchOpFilter {
    /// Query match expression
    pub fn new(query_string: String, query_fields: String, operator: Operator) -> MatchOpFilter {
        MatchOpFilter {
            query_string,
            query_fields,
            operator,
        }
    }
}

///
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum Operator {
    #[serde(rename = "or")]
    Or,
    #[serde(rename = "and")]
    And,
}

impl Default for Operator {
    fn default() -> Operator {
        Self::Or
    }
}
