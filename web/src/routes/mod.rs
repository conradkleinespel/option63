mod api;
mod assets;
mod home;
mod legal;
pub mod segments;

use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(home::index);
    cfg.service(legal::legal);
    cfg.service(api::parse_vcard);
    cfg.service(segments::it);
    cfg.service(segments::personal);
    cfg.service(segments::ai_integrator);
    cfg.service(
        actix_web::web::resource("/{tail:.*}").route(actix_web::web::to(assets::catch_all)),
    );
}
