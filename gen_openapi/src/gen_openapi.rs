use std::fs;

use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};

use kromer::routes::krist::TransactionsApiDoc;

fn main() {
    let doc = gen_my_openapi();
    let _ = fs::write("./target/open_api.json", doc);
}

fn gen_my_openapi() -> String {
    #[derive(Deserialize, ToSchema)]
    struct Person {
        _id: i64,
        _name: String,
    }

    #[derive(OpenApi)]
    #[openapi(components(schemas(Person), responses()))]
    struct ApiDoc;

    let _ = ApiDoc::openapi().to_pretty_json().unwrap();

    TransactionsApiDoc::openapi().to_pretty_json().unwrap()
}
