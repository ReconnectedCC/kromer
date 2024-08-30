fn main() -> Result<(), Box<dyn std::error::Error>> {
    kromer_economy_api::start()?;

    Ok(())
}
