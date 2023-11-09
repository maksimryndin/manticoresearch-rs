/*
 * Manticore Search Client
 *
 * Сlient for Manticore Search. 
 *
 * The version of the OpenAPI document: 3.3.1
 * Contact: info@manticoresearch.com
 * Generated by: https://openapi-generator.tech
 */

/// MatchOp : Query match expression with logical operator



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MatchOp {
    #[serde(rename = "query_info")]
    pub query_info: serde_json::Value,
}

impl MatchOp {
    /// Query match expression with logical operator
    pub fn new(query_info: serde_json::Value) -> MatchOp {
        MatchOp {
            query_info,
        }
    }
}


