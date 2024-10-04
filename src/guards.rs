use actix_web::guard::GuardContext;

pub const INTERNAL_KEY_HEADER: &str = "Kromer-Key";

pub fn internal_key_guard(ctx: &GuardContext) -> bool {
    let capy_password = std::env::var("INTERNAL_KEY").expect("No INTERNAL_KEY set in .env file");
    ctx.head()
        .headers()
        .get(INTERNAL_KEY_HEADER)
        .map_or(false, |it| it.as_bytes() == capy_password.as_bytes())
}
