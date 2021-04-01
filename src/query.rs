use std::collections::{HashMap, HashSet};

use indexmap::IndexMap;
use openapiv3::{
    Contact, Info, IntegerType, MediaType, OpenAPI, Operation, Parameter, ParameterData,
    ParameterSchemaOrContent, PathItem, ReferenceOr, Response, Responses, Schema, SchemaData,
    SchemaKind, Server, Type,
};
use serde::{Deserialize, Serialize};
use sqlparser::{ast::Expr, dialect::MySqlDialect, parser::Parser};
use warp::path::FullPath;

use crate::{mysql::Session, output::QueryOutput};

#[derive(Debug, Serialize, Deserialize)]
pub struct Paging {
    limit: Option<usize>,
    offset: Option<usize>,
}

impl Paging {
    pub fn to_params() -> Vec<ReferenceOr<Parameter>> {
        let mut params = vec![];
        params.push(ReferenceOr::Item(Parameter::Query {
            parameter_data: ParameterData {
                name: "limit".to_string(),
                description: Some("max row limit, or page size".to_string()),
                required: false,
                deprecated: None,
                format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema {
                    schema_data: SchemaData::default(),
                    schema_kind: SchemaKind::Type(Type::Integer(IntegerType::default())),
                })),
                example: None,
                examples: Default::default(),
            },
            allow_reserved: false,
            style: Default::default(),
            allow_empty_value: None,
        }));
        params.push(ReferenceOr::Item(Parameter::Query {
            parameter_data: ParameterData {
                name: "offset".to_string(),
                description: Some("query offset".to_string()),
                required: false,
                deprecated: None,
                format: ParameterSchemaOrContent::Schema(ReferenceOr::Item(Schema {
                    schema_data: SchemaData::default(),
                    schema_kind: SchemaKind::Type(Type::Integer(IntegerType::default())),
                })),
                example: None,
                examples: Default::default(),
            },
            allow_reserved: false,
            style: Default::default(),
            allow_empty_value: None,
        }));
        params
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Query {
    pub profile: String,
    pub sql: String,
    pub url: String,
    pub description: Option<String>,
    pub paging: Option<bool>,
}

impl Query {
    pub fn open_api(&self) -> PathItem {
        let mut get = Operation::default();
        get.operation_id = Some(self.url.clone());
        get.summary = self.description.clone();
        let mut content = IndexMap::new();
        content.insert("application/json".to_string(), MediaType::default());
        if self.paging != Some(false) {
            get.parameters = Paging::to_params();
        }
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
            ..Default::default()
        }
    }

    pub fn with_paging(&self, paging: Option<Paging>) -> String {
        // TODO should use cache to parse only once?
        let dialect = MySqlDialect {};

        if self.paging == Some(false) {
            return self.sql.clone();
        }

        if let Some(paging) = paging {
            let ast = Parser::parse_sql(&dialect, &self.sql).unwrap();
            let query_ast = ast.first().unwrap();
            if let sqlparser::ast::Statement::Query(mut query) = query_ast.clone() {
                if query.limit.is_some() || query.offset.is_some() {
                    log::warn!("paging is disabled for `LIMIT` or `OFFSET` in original sql");
                    self.sql.clone()
                } else {
                    if let Some(limit) = paging.limit {
                        query.limit = Some(Expr::Value(sqlparser::ast::Value::Number(
                            limit.to_string(),
                        )));
                    }
                    if let Some(offset) = paging.offset {
                        query.offset = Some(sqlparser::ast::Offset {
                            value: Expr::Value(sqlparser::ast::Value::Number(offset.to_string())),
                            rows: sqlparser::ast::OffsetRows::None,
                        });
                    }
                    query.to_string()
                }
            } else {
                self.sql.clone()
            }
        } else {
            self.sql.clone()
        }
    }
}

pub enum QueryData {
    Paged {
        data: QueryOutput,
        count: usize,
        next: String,
    },
    NotPaged {
        data: QueryOutput,
    },
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
        paging: Option<Paging>,
        sessions: &HashMap<String, Session>,
    ) -> anyhow::Result<QueryOutput> {
        // remove prefix and '/' around it
        let to_match = full_path.as_str()[(self.prefix.len() + 2)..].trim_end_matches('/');
        let query = self.queries.iter().find(|q| q.url == to_match).unwrap();
        let sess = sessions.get(&query.profile).unwrap();
        Ok(sess.query(&query.with_paging(paging)).await?)
    }

    pub fn with_meta(&self) -> Self {
        let mut copied = self.clone();
        copied.queries.push(Query {
            profile: "None".to_string(),
            sql: "".to_string(),
            url: "_meta".to_string(),
            description: Some("Meta data api".to_string()),
            paging: None,
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
            url: format!("/{}", self.prefix),
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
