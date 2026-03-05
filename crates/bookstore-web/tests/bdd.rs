use bookstore_app::CatalogService;
use bookstore_domain::{Book, Inventory};
use bookstore_web::{AppState, app};
use cucumber::writer::Stats;
use cucumber::{World, given, then, when};
use reqwest::StatusCode;

#[derive(Default, World, Debug)]
struct ApiWorld {
    base_url: Option<String>,
    response_body: Option<String>,
    status: Option<StatusCode>,
    tenant_id: Option<String>,
    locale: Option<String>,
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

        let state = AppState { catalog: CatalogService::from_inventory(inventory), db_pool: None };

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
    world.tenant_id = None;
    world.locale = None;
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response =
        reqwest::get(format!("{base}/books")).await.expect("books request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when("I request the health endpoint")]
async fn request_health(world: &mut ApiWorld) {
    world.tenant_id = None;
    world.locale = None;
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response =
        reqwest::get(format!("{base}/health")).await.expect("health request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[given(expr = "I set tenant id to {word}")]
fn set_tenant(world: &mut ApiWorld, tenant_id: String) {
    world.tenant_id = Some(tenant_id);
}

#[given(expr = "I set locale to {word}")]
fn set_locale(world: &mut ApiWorld, locale: String) {
    world.locale = Some(locale);
}

#[when("I request the request context endpoint")]
async fn request_context(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let url = format!("{base}/context");
    let client = reqwest::Client::new();
    let mut request = client.get(url);

    if let Some(tenant_id) = &world.tenant_id {
        request = request.header("x-tenant-id", tenant_id);
    }
    if let Some(locale) = &world.locale {
        request = request.header("accept-language", locale);
    }

    let response = request.send().await.expect("context request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[then(expr = "the status code is {int}")]
fn status_code_is(world: &mut ApiWorld, status: u16) {
    assert_eq!(world.status, Some(StatusCode::from_u16(status).expect("valid status code")));
}

#[then(expr = "the response contains {string}")]
fn response_contains(world: &mut ApiWorld, expected: String) {
    let body = world.response_body.as_ref().expect("body should exist");
    assert!(body.contains(&expected), "response body did not include {expected}: {body}");
}

#[then("the response contains Celebration of Discipline")]
fn body_contains_seed(world: &mut ApiWorld) {
    assert_eq!(world.status, Some(StatusCode::OK));
    let body = world.response_body.as_ref().expect("body should exist");
    assert!(body.contains("Celebration of Discipline"));
}

#[tokio::test]
async fn bdd() {
    let writer = ApiWorld::cucumber().fail_on_skipped().run("tests/features").await;
    assert!(!writer.execution_has_failed(), "cucumber scenarios failed");
}
