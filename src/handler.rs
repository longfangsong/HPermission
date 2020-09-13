use actix_web::{HttpRequest, Responder, HttpResponse};
use crate::token::get_username;
use crate::CONFIG;
use actix_http::http::Method;
use actix_web::http::Uri;

pub async fn ping(_: HttpRequest) -> impl Responder {
    "pong"
}

pub async fn check(request: HttpRequest) -> impl Responder {
    println!("{:?}", request.method());
    let forwarded_uri = request.headers().get("x-forwarded-uri")
        .and_then(|it| it.to_str().ok())
        .and_then(|it| it.parse::<Uri>().ok());
    let (path, query) = match &forwarded_uri {
        Some(uri) => (uri.path(), uri.query().unwrap_or("")),
        None => (request.path(), request.query_string())
    };
    let username = request.headers().get("Authorization")
        .and_then(|it| it.to_str().ok())
        .map(|it| &it["Bearer ".len()..])
        .and_then(get_username);
    let write = match request.method() {
        &Method::POST | &Method::PUT | &Method::DELETE => true,
        &Method::GET => false,
        _ => return HttpResponse::UnprocessableEntity()
    };
    let passed = CONFIG.check(username.as_ref(), path, query, write);
    if passed {
        info!("Allow {} to visit {}", username.unwrap_or("anonymous user".to_string()), request.path());
        HttpResponse::Ok()
    } else {
        info!("Disallow {} to visit {}", username.unwrap_or("anonymous user".to_string()), request.path());
        HttpResponse::Unauthorized()
    }
}