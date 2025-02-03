use actix_files::{Files, NamedFile};
use actix_web::{web, HttpRequest, HttpResponse};

async fn serve_index(req: HttpRequest) -> actix_web::Result<HttpResponse> {
    let mut static_path = std::env::current_dir().unwrap();
    static_path.push("./docs/dist/index.html");

    let file = NamedFile::open(&static_path)?;
    Ok(file.into_response(&req))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    let mut static_path = std::env::current_dir().unwrap();
    static_path.push("./docs/dist");

    cfg.service(
        Files::new("", &static_path)
            .index_file("index.html")
            .prefer_utf8(true),
    ).route("/docs/{tail:.*}", web::get().to(serve_index));
}
