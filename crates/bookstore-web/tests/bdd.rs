use bookstore_app::CatalogService;
use bookstore_app::{InMemoryProfitReportRepository, ProfitReportRepository};
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
    gst_inclusive_cents: Option<i64>,
    gst_component_cents: Option<i64>,
    profit_tenant_id: Option<String>,
    reported_revenue_cents: Option<i64>,
    reported_cogs_cents: Option<i64>,
    reported_gross_profit_cents: Option<i64>,
    profit_repo: Option<InMemoryProfitReportRepository>,
}

impl ApiWorld {
    async fn ensure_server(&mut self) {
        if self.base_url.is_some() {
            if self.profit_repo.is_none() {
                self.profit_repo = Some(InMemoryProfitReportRepository::new());
            }
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
        self.profit_repo = Some(InMemoryProfitReportRepository::new());
    }

    fn ensure_profit_repo(&mut self) {
        if self.profit_repo.is_none() {
            self.profit_repo = Some(InMemoryProfitReportRepository::new());
        }
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

#[given(expr = "a gst-inclusive amount of {int} cents in AUD")]
fn given_gst_amount(world: &mut ApiWorld, cents: i64) {
    world.gst_inclusive_cents = Some(cents);
}

#[when("I calculate the GST component")]
fn calculate_gst_component(world: &mut ApiWorld) {
    let gst = bookstore_domain::Money::from_minor(
        "AUD",
        world.gst_inclusive_cents.expect("gst-inclusive cents should be set"),
    )
    .expect("valid money")
    .gst_component_cents();
    world.gst_component_cents = Some(gst);
}

#[then(expr = "the GST component is {int} cents")]
fn then_gst_component(world: &mut ApiWorld, cents: i64) {
    assert_eq!(world.gst_component_cents, Some(cents));
}

#[given(expr = "tenant {word} has a sold line with revenue {int} cents and cost {int} cents")]
async fn given_sold_line(
    world: &mut ApiWorld,
    tenant_id: String,
    revenue_cents: i64,
    cost_cents: i64,
) {
    world.ensure_profit_repo();
    let snapshot = bookstore_app::OrderLineCostSnapshot {
        tenant_id,
        revenue: bookstore_domain::Money::from_minor("AUD", revenue_cents).expect("valid money"),
        cost: bookstore_domain::Money::from_minor("AUD", cost_cents).expect("valid money"),
    };
    let report = world.profit_repo.as_ref().expect("profit repository should be available");
    report.record(snapshot).await.expect("record sold line");
}

#[when(expr = "I build a profit report for tenant {word}")]
async fn build_profit_report(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_profit_repo();
    world.profit_tenant_id = Some(tenant_id.clone());
    let repo = world.profit_repo.as_ref().expect("profit repository should be available");
    let report = repo.profit_for_tenant(&tenant_id).await.expect("profit report");
    world.reported_revenue_cents = Some(report.revenue.minor_units);
    world.reported_cogs_cents = Some(report.cogs.minor_units);
    world.reported_gross_profit_cents = Some(report.gross_profit.minor_units);
}

#[then(expr = "reported revenue is {int} cents")]
fn then_reported_revenue(world: &mut ApiWorld, cents: i64) {
    assert_eq!(world.reported_revenue_cents, Some(cents));
}

#[then(expr = "reported cogs is {int} cents")]
fn then_reported_cogs(world: &mut ApiWorld, cents: i64) {
    assert_eq!(world.reported_cogs_cents, Some(cents));
}

#[then(expr = "reported gross profit is {int} cents")]
fn then_reported_gross_profit(world: &mut ApiWorld, cents: i64) {
    assert_eq!(world.reported_gross_profit_cents, Some(cents));
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
