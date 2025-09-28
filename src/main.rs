mod handler;

use actix_web::{App, HttpServer, web};
use handler::index;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    HttpServer::new(move || App::new().route("/", web::get().to(index)))
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
