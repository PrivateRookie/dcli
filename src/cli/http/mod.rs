use warp::http::Response;
use warp::{filters::BoxedFilter, Filter, Reply};

use crate::output::QueryOutput;

fn assets_filter() -> BoxedFilter<(impl Reply,)> {
    warp::path("assets").and(warp::fs::dir("./assets")).boxed()
}

const CT_KEY: &str = "Content-Type";

pub async fn serve(port: u16, output: QueryOutput) {
    let json_resp = output.to_json().unwrap();
    let json_resp_clone = json_resp.clone();
    let csv_resp = output.to_csv().unwrap();
    let yaml_resp = output.to_yaml().unwrap();
    let index = warp::get().and(warp::path::end()).map(|| {
        Response::builder()
            .header(CT_KEY, "text/html")
            .body(include_str!("assets/index.html"))
    });
    let favicon = warp::get()
        .and(warp::path("favicon.ico"))
        .and(warp::path::end())
        .map(|| {
            Response::builder()
                .header(CT_KEY, "image/svg+xml")
                .body(include_str!("assets/favicon.svg"))
        });
    let loading_svg = warp::get()
        .and(warp::path("assets"))
        .and(warp::path("loader.svg"))
        .and(warp::path::end())
        .map(|| {
            Response::builder()
                .header(CT_KEY, "image/svg+xml")
                .body(include_str!("assets/loader.svg"))
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
        .or(loading_svg)
        .or(assets_filter())
        .or(data_api)
        .or(download_csv)
        .or(download_json)
        .or(download_yaml);
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}
