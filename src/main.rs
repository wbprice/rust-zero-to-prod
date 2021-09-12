use zero2prod::run;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let server: tide::Server<()> = run().await;
    server.listen("localhost:8080").await?;
    Ok(())
}
