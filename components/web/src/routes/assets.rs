use actix_files::NamedFile;
use actix_web::web::Path;
use actix_web::{Error, HttpResponse, Responder};
use askama::Template;
use std::path::Path as StdPath;

#[derive(Template)]
#[template(path = "404.html")]
struct NotFoundTemplate {}

fn sanitize_path(path: &str) -> Option<String> {
    let cleaned: Vec<&str> = path
        .split('/')
        .filter(|segment| *segment != "." && *segment != ".." && !segment.is_empty())
        .collect();

    if cleaned.is_empty() {
        None
    } else {
        Some(cleaned.join("/"))
    }
}

pub enum PageResponse {
    File(NamedFile),
    NotFound,
}

impl Responder for PageResponse {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> actix_web::HttpResponse<Self::Body> {
        match self {
            PageResponse::File(file) => file.respond_to(_req),
            PageResponse::NotFound => {
                let html = NotFoundTemplate {}
                    .render()
                    .unwrap_or_else(|e| format!("Template error: {e}"));
                HttpResponse::NotFound()
                    .content_type("text/html; charset=utf-8")
                    .body(html)
            }
        }
    }
}

pub async fn catch_all(tail: Path<String>) -> Result<PageResponse, Error> {
    let raw_path = tail.into_inner();

    let safe_path = sanitize_path(&raw_path)
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Invalid path"))?;

    let file_path = format!("./components/web/src/assets/{}", safe_path);

    if !StdPath::new(&file_path).exists() {
        return Ok(PageResponse::NotFound);
    }

    let file = NamedFile::open(&file_path).map_err(actix_web::error::ErrorInternalServerError)?;
    Ok(PageResponse::File(file))
}
