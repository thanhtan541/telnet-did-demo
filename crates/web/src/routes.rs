use actix_web::{get, HttpResponse};

#[get("/health_check")]
pub async fn health_check() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}

#[get("/")]
pub async fn index() -> Result<HttpResponse, actix_web::Error> {
    Ok(HttpResponse::Ok().finish())
}
