use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::Next;
use actix_web::{HttpResponse, http::header};

pub async fn apex_to_www(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, actix_web::Error> {
    let redirect_to = {
        let conn_info = req.connection_info();
        let host = conn_info.host();
        let hostname = host.split(':').next().unwrap_or(host);
        if hostname.eq_ignore_ascii_case("option63.eu") {
            let path_and_query = req.uri().path_and_query().map(|p| p.as_str()).unwrap_or("");
            Some(format!("https://www.option63.eu{path_and_query}"))
        } else {
            None
        }
    };

    match redirect_to {
        Some(location) => Ok(req
            .into_response(
                HttpResponse::MovedPermanently()
                    .append_header((header::LOCATION, location))
                    .finish(),
            )
            .map_into_right_body()),
        None => {
            let res = next.call(req).await?;
            Ok(res.map_into_left_body())
        }
    }
}
