use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use indexmap::IndexMap;
use openapiv3::{
    Contact, Info, MediaType, OpenAPI, Operation, PathItem, Paths, ReferenceOr, RequestBody,
    Response, Responses, Server,
};
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

impl Query {
    pub fn open_api(&self) -> PathItem {
        let mut get = Operation::default();
        get.operation_id = Some(self.name.clone());
        let mut content = IndexMap::new();
        content.insert("application/json".to_string(), MediaType::default());
        get.request_body = Some(ReferenceOr::Item(RequestBody {
            description: Some("".to_string()),
            content: content.clone(),
            required: true,
        }));
        get.responses = Responses {
            default: Some(ReferenceOr::Item(Response {
                description: "OK".to_string(),
                headers: IndexMap::new(),
                content: content,
                links: IndexMap::new(),
            })),
            responses: IndexMap::new(),
        };
        PathItem {
            get: Some(get),
            put: None,
            post: None,
            delete: None,
            options: None,
            head: None,
            patch: None,
            trace: None,
            servers: vec![],
            parameters: vec![],
        }
    }
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

    pub fn open_api(&self) -> OpenAPI {
        let openapi = "3.0.3".to_string();
        let info = Info {
            title: "DCli-HTTP".to_string(),
            description: Some("Serve by DCli".to_string()),
            terms_of_service: None,
            contact: Some(Contact {
                name: Some("PrivateRookie".to_string()),
                url: Some("https://github.com/PrivateRookie".to_string()),
                email: Some("xdsailfish@gmail.com".to_string()),
            }),
            license: None,
            version: "3.0.0".to_string(),
        };
        let server = Server {
            url: "http://localhost:3030/api".to_string(),
            description: None,
            variables: None,
        };
        let servers = vec![server];
        let mut paths = IndexMap::new();
        for query in self.queries.iter() {
            paths.insert(
                format!("/{}", query.url),
                ReferenceOr::Item(query.open_api()),
            );
        }

        let components = None;
        let security = vec![];
        let tags = vec![];
        let external_docs = None;
        OpenAPI {
            openapi,
            info,
            servers,
            paths,
            components,
            security,
            tags,
            external_docs,
        }
    }
}
