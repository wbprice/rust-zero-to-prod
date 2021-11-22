use std::net::TcpListener;

#[async_std::main]
async fn main() -> Result<(), tide::Error> {
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Unable to bind port");
    zero2prod::run(listener).await?;
    Ok(())
}
