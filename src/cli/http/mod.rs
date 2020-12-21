use rust_embed::RustEmbed;
use warp::Filter;
use warp::{http::Response, path::FullPath};

use crate::output::QueryOutput;

#[derive(RustEmbed)]
#[folder = "assets"]
struct Asset;

const CT_KEY: &str = "Content-Type";

pub async fn serve(port: u16, output: QueryOutput) {
    let json_resp = output.to_json().unwrap();
    let json_resp_clone = json_resp.clone();
    let csv_resp = output.to_csv().unwrap();
    let yaml_resp = output.to_yaml().unwrap();
    let index = warp::get().and(warp::path::end()).map(|| {
        Response::builder()
            .header(CT_KEY, "text/html")
            .body(Asset::get("index.html").unwrap())
    });

    let assets = warp::get()
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
        });
    let favicon = warp::get()
        .and(warp::path("favicon.ico"))
        .and(warp::path::end())
        .map(|| {
            Response::builder()
                .header(CT_KEY, "image/svg+xml")
                .body(Asset::get("favicon.svg").unwrap())
        });
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
    let routes = index
        .or(favicon)
        .or(assets)
        .or(data_api)
        .or(download_csv)
        .or(download_json)
        .or(download_yaml);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
