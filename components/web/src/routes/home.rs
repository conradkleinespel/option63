use crate::routes::segments;
use actix_web::{HttpResponse, get};
use askama::Template;

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate {
    segments: Vec<segments::SegmentSummary>,
}

#[get("/")]
pub async fn index() -> HttpResponse {
    let html = HomeTemplate {
        segments: segments::summaries(),
    }
    .render()
    .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
