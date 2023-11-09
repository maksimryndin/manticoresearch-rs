#[macro_use]
extern crate serde_derive;

#[macro_use]
pub mod apis;
#[cfg(feature = "migrate")]
pub mod migrator;
pub mod models;

pub use apis::configuration::Configuration;
use models::MatchFilter;
use serde::Serializer;
use std::collections::HashMap;

impl serde::Serialize for MatchFilter {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        HashMap::from([(
            "match".to_string(),
            HashMap::from([(&self.query_fields, &self.query_string)]),
        )])
        .serialize(s)
    }
}
