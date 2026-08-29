use actix_web::{HttpResponse, get};
use askama::Template;

#[derive(Template)]
#[template(path = "legal.html")]
struct LegalTemplate {}

#[get("/legal/")]
pub async fn legal() -> HttpResponse {
    let html = LegalTemplate {}
        .render()
        .unwrap_or_else(|e| format!("Template error: {e}"));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
