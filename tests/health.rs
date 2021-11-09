use zero2prod;

#[async_std::test]
async fn health_check() {
    spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get("http://localhost:8080/health")
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() {
    let _ = async_std::task::spawn(zero2prod::run("localhost:8080"));
}
