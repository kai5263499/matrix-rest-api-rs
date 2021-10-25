use env_logger::Env;
use actix_web::{web, App, HttpRequest, HttpServer, Responder};

async fn greet(req: HttpRequest) -> impl Responder {
    let name = req.match_info().get("name").unwrap_or("World");
    format!("Hello {}!", &name)
}

#[actix_web::main()]
async fn main() -> std::io::Result<()> {
  let env = Env::default()
        .filter_or("LOG_LEVEL", "debug")
        .write_style_or("LOG_STYLE", "always");

    env_logger::init_from_env(env);

    HttpServer::new(move || {
        App::new()
            .route("/{name}", web::get().to(greet))
    })
    .bind(("127.0.0.1", 8081))?
    .run()
    .await?;
    
    Ok(())
}
