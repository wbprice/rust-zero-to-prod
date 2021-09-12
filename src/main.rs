use zero2prod::run;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let server = run();
    server.listen("localhost:8080").await?;
    Ok(())
}
