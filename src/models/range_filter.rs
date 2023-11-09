/*
 * Manticore Search Client
 *
 * Сlient for Manticore Search.
 *
 * The version of the OpenAPI document: 3.3.1
 * Contact: info@manticoresearch.com
 * Generated by: https://openapi-generator.tech
 */

/// RangeFilter : Range attribute filter

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RangeFilter {
    #[serde(rename = "field")]
    pub field: String,
    #[serde(
        rename = "lte",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub lte: Option<Option<f32>>,
    #[serde(
        rename = "gte",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub gte: Option<Option<f32>>,
    #[serde(
        rename = "lt",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub lt: Option<Option<f32>>,
    #[serde(
        rename = "gt",
        default,
        with = "::serde_with::rust::double_option",
        skip_serializing_if = "Option::is_none"
    )]
    pub gt: Option<Option<f32>>,
}

impl RangeFilter {
    /// Range attribute filter
    pub fn new(field: String) -> RangeFilter {
        RangeFilter {
            field,
            lte: None,
            gte: None,
            lt: None,
            gt: None,
        }
    }
}
