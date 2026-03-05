use std::sync::Arc;

use bookstore_core::{Book, Inventory};
use bookstore_web::{AppState, app};
use cucumber::{World, given, then, when};
use reqwest::StatusCode;
use tokio::sync::RwLock;

#[derive(Default, World, Debug)]
struct ApiWorld {
    base_url: Option<String>,
    response_body: Option<String>,
    status: Option<StatusCode>,
}

impl ApiWorld {
    async fn ensure_server(&mut self) {
        if self.base_url.is_some() {
            return;
        }

        let mut inventory = Inventory::new();
        inventory
            .add_book(Book {
                id: "bk-900".to_string(),
                title: "Celebration of Discipline".to_string(),
                author: "Richard Foster".to_string(),
                category: "Spiritual Formation".to_string(),
                price_cents: 1699,
            })
            .expect("seed should be valid");

        let state = AppState { inventory: Arc::new(RwLock::new(inventory)) };

        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind test listener");
        let addr = listener.local_addr().expect("resolve bound addr");
        tokio::spawn(async move {
            axum::serve(listener, app(state)).await.expect("run test server");
        });

        self.base_url = Some(format!("http://{addr}"));
    }
}

#[given("the bookstore api is running")]
async fn api_running(world: &mut ApiWorld) {
    world.ensure_server().await;
}

#[when("I request the books catalog")]
async fn request_books(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response =
        reqwest::get(format!("{base}/books")).await.expect("books request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[then("the response contains Celebration of Discipline")]
fn body_contains_seed(world: &mut ApiWorld) {
    assert_eq!(world.status, Some(StatusCode::OK));
    let body = world.response_body.as_ref().expect("body should exist");
    assert!(body.contains("Celebration of Discipline"));
}

#[tokio::test]
async fn bdd() {
    ApiWorld::cucumber().fail_on_skipped().run("tests/features").await;
}
