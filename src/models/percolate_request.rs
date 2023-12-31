/*
 * Manticore Search Client
 *
 * Сlient for Manticore Search.
 *
 * The version of the OpenAPI document: 3.3.1
 * Contact: info@manticoresearch.com
 * Generated by: https://openapi-generator.tech
 */

/// PercolateRequest : Object with documents to percolate

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PercolateRequest {
    #[serde(rename = "query")]
    pub query: crate::models::PercolateRequestQuery,
}

impl PercolateRequest {
    /// Object with documents to percolate
    pub fn new(query: crate::models::PercolateRequestQuery) -> PercolateRequest {
        PercolateRequest { query }
    }
}
