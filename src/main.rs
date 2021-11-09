#[async_std::main]
async fn main() -> Result<(), tide::Error> {
    zero2prod::run("localhost:8080").await?;
    Ok(())
}
