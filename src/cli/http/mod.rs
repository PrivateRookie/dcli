use std::{collections::HashMap, convert::Infallible};

use rust_embed::RustEmbed;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::Filter;
use warp::{http::Response, path::FullPath};

use crate::{
    mysql::Session,
    output::{QueryOutput, QueryOutputMapSer},
    query::QueryPlan,
};

#[derive(RustEmbed)]
#[folder = "assets"]
struct Asset;

const CT_KEY: &str = "Content-Type";

pub async fn serve(port: u16, output: QueryOutput) {
    let json_resp = output.to_json().unwrap();
    let json_resp_clone = json_resp.clone();
    let csv_resp = output.to_csv().unwrap();
    let yaml_resp = output.to_yaml().unwrap();

    let data_api = warp::get().and(warp::path("data")).map(move || {
        Response::builder()
            .header("Content-Type", "application/json")
            .body(json_resp.clone())
    });
    let download_csv = warp::get()
        .and(warp::path("download"))
        .and(warp::path("csv"))
        .map(move || {
            Response::builder()
                .header(CT_KEY, "text/csv")
                .body(csv_resp.clone())
        });
    let download_json = warp::get()
        .and(warp::path("download"))
        .and(warp::path("json"))
        .map(move || {
            Response::builder()
                .header(CT_KEY, "application/json")
                .body(json_resp_clone.clone())
        });
    let download_yaml = warp::get()
        .and(warp::path("download"))
        .and(warp::path("yaml"))
        .map(move || {
            Response::builder()
                .header(CT_KEY, "application/text")
                .body(yaml_resp.clone())
        });
    let routes = serve_static()
        .or(data_api)
        .or(download_csv)
        .or(download_json)
        .or(download_yaml);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

fn index() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get().and(warp::path::end()).map(|| {
        Response::builder()
            .header(CT_KEY, "text/html")
            .body(Asset::get("index.html").unwrap())
    })
}

fn favicon() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("favicon.ico"))
        .and(warp::path::end())
        .map(|| {
            Response::builder()
                .header(CT_KEY, "image/svg+xml")
                .body(Asset::get("favicon.svg").unwrap())
        })
}

fn assets() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("assets"))
        .and(warp::any())
        .and(warp::path::full())
        .map(move |path: FullPath| {
            let relative_path = &path.as_str()[8..];
            if let Some(content) = Asset::get(relative_path) {
                let ct = if relative_path.ends_with("js") {
                    "application/javascript; charset=utf-8"
                } else if relative_path.ends_with("css") {
                    "text/css"
                } else if relative_path.ends_with("svg") {
                    "image/svg+xml"
                } else {
                    "text/plain"
                };
                Response::builder()
                    .header(CT_KEY, ct)
                    .body(content.to_owned())
            } else {
                Response::builder()
                    .status(404)
                    .body(Asset::get("404").unwrap())
            }
        })
}

fn serve_static() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    index().or(favicon()).or(assets())
}

async fn run(
    full_path: FullPath,
    plan: QueryPlan,
    sessions: HashMap<String, Session>,
) -> Result<impl warp::Reply, Infallible> {
    let output = plan.query(full_path, &sessions).await.unwrap();
    Ok(warp::reply::json(&QueryOutputMapSer(&output)))
}

pub async fn serve_plan(plan: QueryPlan, sessions: HashMap<String, Session>) {
    let prefix = plan.prefix.clone();
    let plan_meta = plan.with_meta();
    let overview = warp::get().and(
        warp::path(prefix)
            .and(warp::path("_meta"))
            .map(move || warp::reply::json(&plan_meta)),
    );
    let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "tracing=info,warp=debug".to_owned());
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_span_events(FmtSpan::CLOSE)
        .init();
    let api = warp::get()
        .and(warp::path(plan.prefix.clone()))
        .and(warp::any())
        .and(warp::path::full())
        .and(warp::any().map(move || plan.clone()))
        .and(warp::any().map(move || sessions.clone()))
        .and_then(run);
    let routes = serve_static().or(overview).or(api);
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}
