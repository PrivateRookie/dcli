use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use warp::path::FullPath;

use crate::{mysql::Session, output::QueryOutput};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Query {
    pub name: String,
    pub profile: String,
    pub sql: String,
    pub url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryPlan {
    pub prefix: String,
    pub queries: Vec<Query>,
}

impl QueryPlan {
    // check query required profile exists, sql is valid etc
    pub fn validate(self) -> bool {
        // 1. check profile exists
        // 2. check url not conflict
        // 3. check sql valid
        todo!()
    }

    // return all required profile of a plan
    pub fn profiles(&self) -> HashSet<String> {
        let mut profiles = HashSet::new();
        for query in self.queries.iter() {
            profiles.insert(query.profile.clone());
        }
        profiles
    }

    pub async fn query(
        &self,
        full_path: FullPath,
        sessions: &HashMap<String, Session>,
    ) -> anyhow::Result<QueryOutput> {
        // remove prefix and '/' around it
        let to_match = &full_path.as_str()[(self.prefix.len() + 2)..];
        let query = self.queries.iter().find(|q| q.url == to_match).unwrap();
        let sess = sessions.get(&query.profile).unwrap();
        Ok(sess.query(&query.sql).await?)
    }

    pub fn with_meta(&self) -> Self {
        let mut copied = self.clone();
        copied.queries.push(Query {
            name: "Api Meta".to_string(),
            profile: "None".to_string(),
            sql: "".to_string(),
            url: "_meta".to_string(),
        });
        copied
    }
}
