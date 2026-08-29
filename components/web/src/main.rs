mod redirect;
mod routes;

use actix_files::Files;
use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer};

const PORT: u16 = 8080;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("listening on http://0.0.0.0:{PORT}");

    HttpServer::new(move || {
        App::new()
            .wrap(from_fn(redirect::apex_to_www))
            .service(Files::new("/static", "./components/web/static").prefer_utf8(true))
            .configure(routes::configure)
    })
    .bind(("0.0.0.0", PORT))?
    .run()
    .await
}
