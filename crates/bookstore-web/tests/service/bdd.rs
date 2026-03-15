use bookstore_app::{
    AdminBootstrap, AdminService, CatalogService, PosService, SalesEvent, StorefrontService,
};
use bookstore_app::{InMemoryProfitReportRepository, ProfitReportRepository};
use bookstore_domain::PaymentMethod;
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
    pos_session_token: Option<String>,
    storefront_session_id: Option<String>,
    intake_isbn: Option<String>,
    intake_title: Option<String>,
    intake_author: Option<String>,
    intake_on_hand: Option<i64>,
    admin_token: Option<String>,
    admin_service: Option<AdminService>,
    admin_order_id: Option<String>,
}

impl ApiWorld {
    async fn ensure_server(&mut self) {
        if self.base_url.is_some() {
            if self.profit_repo.is_none() {
                self.profit_repo = Some(InMemoryProfitReportRepository::new());
            }
            return;
        }

        let admin = AdminService::with_bootstrap(AdminBootstrap::local_defaults());
        let state = AppState {
            catalog: CatalogService::with_seed(),
            pos: PosService::with_seed(),
            storefront: StorefrontService::new(),
            admin: admin.clone(),
            db_pool: None,
            cover_storage: None,
            isbn_lookup: None,
        };

        let listener =
            tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind test listener");
        let addr = listener.local_addr().expect("resolve bound addr");
        tokio::spawn(async move {
            axum::serve(listener, app(state)).await.expect("run test server");
        });

        self.base_url = Some(format!("http://{addr}"));
        self.profit_repo = Some(InMemoryProfitReportRepository::new());
        self.admin_service = Some(admin);
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

#[when("I open the POS shell page")]
async fn open_pos_shell(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response =
        reqwest::get(format!("{base}/pos")).await.expect("pos shell request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when(expr = "I fetch static file {string}")]
async fn fetch_static_file(world: &mut ApiWorld, path: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response =
        reqwest::get(format!("{base}{path}")).await.expect("static file request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when("I open the storefront catalog page")]
async fn open_storefront_catalog(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response =
        reqwest::get(format!("{base}/catalog")).await.expect("catalog request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when("I open the admin intake page")]
async fn open_admin_intake(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response = reqwest::get(format!("{base}/admin/intake"))
        .await
        .expect("admin intake request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when("I open the admin dashboard page")]
async fn open_admin_dashboard(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response = reqwest::get(format!("{base}/admin"))
        .await
        .expect("admin dashboard request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when("I open the admin orders page")]
async fn open_admin_orders(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response = reqwest::get(format!("{base}/admin/orders"))
        .await
        .expect("admin orders request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when(expr = "I search the storefront catalog for {word}")]
async fn search_storefront_catalog(world: &mut ApiWorld, query: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response = reqwest::get(format!("{base}/catalog/search?q={query}"))
        .await
        .expect("catalog search request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when(expr = "I open the storefront catalog page filtered for {word}")]
async fn open_storefront_catalog_filtered(world: &mut ApiWorld, query: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response = reqwest::get(format!("{base}/catalog?q={query}"))
        .await
        .expect("catalog request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when("I open the storefront checkout page")]
async fn open_storefront_checkout(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response =
        reqwest::get(format!("{base}/checkout")).await.expect("checkout request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when(expr = "I open the storefront product page for {word}")]
async fn open_storefront_product_page(world: &mut ApiWorld, book_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response = reqwest::get(format!("{base}/catalog/items/{book_id}"))
        .await
        .expect("product detail request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[when("I open the storefront cart page")]
async fn open_storefront_cart(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response = reqwest::get(format!("{base}/cart")).await.expect("cart request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read body"));
}

#[given(expr = "I scan ISBN {word} for admin intake")]
fn admin_scan_isbn_for_intake(world: &mut ApiWorld, isbn: String) {
    world.intake_isbn = Some(isbn);
}

#[when("I lookup isbn metadata for intake")]
async fn admin_lookup_isbn_metadata(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let isbn = world.intake_isbn.clone().expect("isbn should be set");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/admin/products/isbn-lookup"))
        .json(&serde_json::json!({ "token": token, "isbn": isbn }))
        .send()
        .await
        .expect("isbn lookup request should succeed");
    world.status = Some(response.status());
    let body = response.text().await.expect("read response body");
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        world.intake_title =
            json.get("title").and_then(serde_json::Value::as_str).map(str::to_string);
        world.intake_author =
            json.get("author").and_then(serde_json::Value::as_str).map(str::to_string);
    }
    world.response_body = Some(body);
}

#[then(expr = "the intake metadata title is {string}")]
fn admin_intake_title(world: &mut ApiWorld, title: String) {
    assert_eq!(world.intake_title.as_deref(), Some(title.as_str()));
}

#[then(expr = "the intake metadata author is {string}")]
fn admin_intake_author(world: &mut ApiWorld, author: String) {
    assert_eq!(world.intake_author.as_deref(), Some(author.as_str()));
}

#[when(expr = "I record intake with cost {int} cents retail {int} cents and quantity {int}")]
async fn admin_record_intake(
    world: &mut ApiWorld,
    _cost_cents: i64,
    _retail_cents: i64,
    quantity: i64,
) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let isbn = world.intake_isbn.clone().expect("isbn should be set");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/admin/inventory/receive"))
        .json(&serde_json::json!({
            "token": token,
            "tenant_id": "church-a",
            "isbn": isbn,
            "quantity": quantity
        }))
        .send()
        .await
        .expect("inventory receive request should succeed");
    world.status = Some(response.status());
    let body = response.text().await.expect("read response body");
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        world.intake_on_hand = json.get("on_hand").and_then(serde_json::Value::as_i64);
    }
    world.response_body = Some(body);
}

#[then(expr = "the intake quantity on hand is {int}")]
fn admin_intake_on_hand(world: &mut ApiWorld, quantity: i64) {
    assert_eq!(world.intake_on_hand, Some(quantity));
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

#[then(expr = "the response does not contain {string}")]
fn response_does_not_contain(world: &mut ApiWorld, expected: String) {
    let body = world.response_body.as_ref().expect("body should exist");
    assert!(!body.contains(&expected), "response body unexpectedly included {expected}: {body}");
}

#[then(expr = "the json field {word} equals {int}")]
fn response_json_field_equals_int(world: &mut ApiWorld, field: String, expected: i64) {
    let body = world.response_body.as_ref().expect("body should exist");
    let json =
        serde_json::from_str::<serde_json::Value>(body).expect("response should be valid json");
    let actual = json.get(field.as_str()).and_then(serde_json::Value::as_i64);
    assert_eq!(actual, Some(expected));
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

#[when(expr = "I log into POS with shift pin {int}")]
async fn pos_login(world: &mut ApiWorld, pin: i64) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/login"))
        .json(&serde_json::json!({ "pin": pin.to_string() }))
        .send()
        .await
        .expect("pos login request should succeed");
    world.status = Some(response.status());
    let body = response.text().await.expect("read response body");
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        world.pos_session_token =
            json.get("session_token").and_then(serde_json::Value::as_str).map(str::to_string);
    }
    world.response_body = Some(body);
}

#[when(expr = "I scan ISBN {word}")]
async fn pos_scan(world: &mut ApiWorld, barcode: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.pos_session_token.clone().expect("pos session should be created");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/scan"))
        .json(&serde_json::json!({ "session_token": token, "barcode": barcode }))
        .send()
        .await
        .expect("pos scan request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I add quick item {word} with quantity {int}")]
async fn pos_quick_item(world: &mut ApiWorld, item_id: String, quantity: i64) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.pos_session_token.clone().expect("pos session should be created");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/cart/items"))
        .json(&serde_json::json!({
            "session_token": token,
            "item_id": item_id,
            "quantity": quantity
        }))
        .send()
        .await
        .expect("pos quick item request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I complete external card checkout with reference {word}")]
async fn pos_external_checkout(world: &mut ApiWorld, external_ref: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.pos_session_token.clone().expect("pos session should be created");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/payments/external-card"))
        .json(&serde_json::json!({ "session_token": token, "external_ref": external_ref }))
        .send()
        .await
        .expect("pos external payment request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I complete external card checkout with reference {word} and discount {int} cents")]
async fn pos_external_checkout_with_discount(
    world: &mut ApiWorld,
    external_ref: String,
    discount_cents: i64,
) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.pos_session_token.clone().expect("pos session should be created");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/payments/external-card"))
        .json(&serde_json::json!({
            "session_token": token,
            "external_ref": external_ref,
            "discount_cents": discount_cents
        }))
        .send()
        .await
        .expect("pos external payment request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I pay cash with tendered {int} cents and donate change")]
async fn pos_cash_checkout(world: &mut ApiWorld, tendered_cents: i64) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.pos_session_token.clone().expect("pos session should be created");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/payments/cash"))
        .json(&serde_json::json!({
            "session_token": token,
            "tendered_cents": tendered_cents,
            "donate_change": true
        }))
        .send()
        .await
        .expect("pos cash payment request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I attempt cash checkout with tendered {int} cents")]
async fn pos_cash_checkout_underpayment(world: &mut ApiWorld, tendered_cents: i64) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.pos_session_token.clone().expect("pos session should be created");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/payments/cash"))
        .json(&serde_json::json!({
            "session_token": token,
            "tendered_cents": tendered_cents,
            "donate_change": false
        }))
        .send()
        .await
        .expect("pos cash payment request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I attempt cash checkout on an empty cart with tendered {int} cents")]
async fn pos_cash_checkout_empty_cart(world: &mut ApiWorld, tendered_cents: i64) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.pos_session_token.clone().expect("pos session should be created");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/payments/cash"))
        .json(&serde_json::json!({
            "session_token": token,
            "tendered_cents": tendered_cents,
            "donate_change": false
        }))
        .send()
        .await
        .expect("pos cash payment request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I complete IOU checkout for {word} {word}")]
async fn pos_iou_checkout(world: &mut ApiWorld, first_name: String, last_name: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.pos_session_token.clone().expect("pos session should be created");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/payments/iou"))
        .json(&serde_json::json!({
            "session_token": token,
            "customer_name": format!("{first_name} {last_name}")
        }))
        .send()
        .await
        .expect("pos iou payment request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when("I attempt IOU checkout with blank customer name")]
async fn pos_iou_checkout_blank_name(world: &mut ApiWorld) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.pos_session_token.clone().expect("pos session should be created");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/payments/iou"))
        .json(&serde_json::json!({
            "session_token": token,
            "customer_name": ""
        }))
        .send()
        .await
        .expect("pos iou payment request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I scan ISBN {word} with a blank POS session token")]
async fn pos_scan_blank_session_token(world: &mut ApiWorld, barcode: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/pos/scan"))
        .json(&serde_json::json!({ "session_token": "", "barcode": barcode }))
        .send()
        .await
        .expect("pos scan request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(
    expr = "I create a storefront checkout session for item {word} quantity {int} and email {word}"
)]
async fn create_storefront_checkout_session(
    world: &mut ApiWorld,
    item_id: String,
    quantity: i64,
    email: String,
) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/storefront/checkout/session"))
        .json(&serde_json::json!({
            "email": email,
            "donation_cents": 0,
            "line_items": [
                {
                    "item_id": item_id,
                    "quantity": quantity
                }
            ]
        }))
        .send()
        .await
        .expect("storefront checkout session request should succeed");
    world.status = Some(response.status());
    let body = response.text().await.expect("read response body");
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        world.storefront_session_id =
            json.get("session_id").and_then(serde_json::Value::as_str).map(str::to_string);
    }
    world.response_body = Some(body);
}

#[when(expr = "I finalize payment webhook with reference {word} for created session")]
async fn finalize_payment_webhook(world: &mut ApiWorld, external_ref: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let session_id =
        world.storefront_session_id.clone().expect("storefront session id should be present");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/payments/webhook"))
        .json(&serde_json::json!({
            "external_ref": external_ref,
            "session_id": session_id
        }))
        .send()
        .await
        .expect("payment webhook request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I login as admin with username {word} and password {word}")]
async fn admin_login(world: &mut ApiWorld, username: String, password: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/admin/auth/login"))
        .json(&serde_json::json!({
            "username": username,
            "password": password
        }))
        .send()
        .await
        .expect("admin login request should succeed");
    world.status = Some(response.status());
    let body = response.text().await.expect("read response body");
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        world.admin_token =
            json.get("token").and_then(serde_json::Value::as_str).map(str::to_string);
    }
    world.response_body = Some(body);
}

#[when(expr = "I upsert admin product {word} for tenant {word}")]
async fn admin_upsert_product(world: &mut ApiWorld, product_id: String, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/admin/products"))
        .json(&serde_json::json!({
            "token": token,
            "tenant_id": tenant_id,
            "product_id": product_id,
            "title": "Celebration of Discipline",
            "isbn": "9780060652937",
            "category": "Spiritual Formation",
            "vendor": "Church Supplier",
            "cost_cents": 900,
            "retail_cents": 1699
        }))
        .send()
        .await
        .expect("admin product upsert request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I attempt cross-origin admin product upsert for tenant {word}")]
async fn admin_upsert_product_cross_origin(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/admin/products"))
        .header("origin", "https://evil.example")
        .json(&serde_json::json!({
            "token": token,
            "tenant_id": tenant_id,
            "product_id": "bk-evil",
            "title": "Bad Origin",
            "isbn": "9780060652937",
            "category": "Spiritual Formation",
            "vendor": "Church Supplier",
            "cost_cents": 900,
            "retail_cents": 1699
        }))
        .send()
        .await
        .expect("admin product upsert request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I list admin products for tenant {word}")]
async fn admin_list_products(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{base}/api/admin/products?tenant_id={tenant_id}"))
        .bearer_auth(token)
        .send()
        .await
        .expect("admin product list request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I list admin products for tenant {word} without auth")]
async fn admin_list_products_no_auth(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response = reqwest::get(format!("{base}/api/admin/products?tenant_id={tenant_id}"))
        .await
        .expect("admin product list request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I adjust admin inventory for tenant {word} isbn {word} by {int} for {word}")]
async fn admin_adjust_inventory(
    world: &mut ApiWorld,
    tenant_id: String,
    isbn: String,
    delta: i64,
    reason: String,
) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/admin/inventory/adjust"))
        .json(&serde_json::json!({
            "token": token,
            "tenant_id": tenant_id,
            "isbn": isbn,
            "delta": delta,
            "reason": reason
        }))
        .send()
        .await
        .expect("admin inventory adjust request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I receive admin inventory for tenant {word} isbn {word} quantity {int}")]
async fn admin_receive_inventory(
    world: &mut ApiWorld,
    tenant_id: String,
    isbn: String,
    quantity: i64,
) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/admin/inventory/receive"))
        .json(&serde_json::json!({
            "token": token,
            "tenant_id": tenant_id,
            "isbn": isbn,
            "quantity": quantity
        }))
        .send()
        .await
        .expect("admin inventory receive request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I fetch admin inventory journal for tenant {word}")]
async fn admin_fetch_inventory_journal(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{base}/api/admin/inventory/journal?tenant_id={tenant_id}"))
        .bearer_auth(token)
        .send()
        .await
        .expect("admin inventory journal request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I fetch admin categories for tenant {word}")]
async fn admin_fetch_categories(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{base}/api/admin/categories?tenant_id={tenant_id}"))
        .bearer_auth(token)
        .send()
        .await
        .expect("admin categories request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I fetch admin vendors for tenant {word}")]
async fn admin_fetch_vendors(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{base}/api/admin/vendors?tenant_id={tenant_id}"))
        .bearer_auth(token)
        .send()
        .await
        .expect("admin vendors request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I delete admin product {word} for tenant {word}")]
async fn admin_delete_product(world: &mut ApiWorld, product_id: String, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .delete(format!("{base}/api/admin/products/{product_id}?tenant_id={tenant_id}"))
        .bearer_auth(token)
        .send()
        .await
        .expect("admin product delete request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I fetch admin report summary for tenant {word}")]
async fn admin_fetch_report_summary(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{base}/api/admin/reports/summary?tenant_id={tenant_id}"))
        .bearer_auth(token)
        .send()
        .await
        .expect("admin report summary request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I fetch admin orders for tenant {word}")]
async fn admin_fetch_orders(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{base}/api/admin/orders?tenant_id={tenant_id}"))
        .bearer_auth(token)
        .send()
        .await
        .expect("admin orders request should succeed");
    world.status = Some(response.status());
    let body = response.text().await.expect("read response body");
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        world.admin_order_id = json
            .as_array()
            .and_then(|orders| {
                orders.iter().find(|order| {
                    order.get("status").and_then(serde_json::Value::as_str) == Some("UnpaidIou")
                })
            })
            .and_then(|order| order.get("order_id"))
            .and_then(serde_json::Value::as_str)
            .map(str::to_string);
    }
    world.response_body = Some(body);
}

#[when(expr = "I mark the first admin IOU order paid for tenant {word}")]
async fn admin_mark_first_iou_paid(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let order_id = world.admin_order_id.clone().expect("admin order id should be set");
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/api/admin/orders/{order_id}/mark-paid?tenant_id={tenant_id}"))
        .bearer_auth(token)
        .header("origin", base)
        .send()
        .await
        .expect("admin mark paid request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I fetch admin report summary for tenant {word} without auth")]
async fn admin_fetch_report_summary_no_auth(world: &mut ApiWorld, tenant_id: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response = reqwest::get(format!("{base}/api/admin/reports/summary?tenant_id={tenant_id}"))
        .await
        .expect("admin report summary request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(
    expr = "I record sales event for tenant {word} on {word} with payment {word} sales {int} donations {int} cogs {int}"
)]
async fn admin_record_sales_event(
    world: &mut ApiWorld,
    tenant_id: String,
    occurred_on: String,
    payment_method: String,
    sales_cents: i64,
    donations_cents: i64,
    cogs_cents: i64,
) {
    world.ensure_server().await;
    let admin = world.admin_service.as_ref().expect("admin service should exist");
    let payment = match payment_method.as_str() {
        "cash" => PaymentMethod::Cash,
        "external_card" => PaymentMethod::ExternalCard,
        "online_card" => PaymentMethod::OnlineCard,
        "iou" => PaymentMethod::Iou,
        "iou_settled" => PaymentMethod::IouSettled,
        other => panic!("unknown payment method in BDD step: {other}"),
    };
    let date = chrono::NaiveDate::parse_from_str(&occurred_on, "%Y-%m-%d")
        .expect("valid date in BDD step");
    admin
        .record_sales_event(SalesEvent {
            tenant_id,
            payment_method: payment,
            sales_cents,
            donations_cents,
            cogs_cents,
            occurred_at: date.and_hms_opt(0, 0, 0).unwrap(),
        })
        .await;
    world.status = Some(StatusCode::OK);
    world.response_body = Some("{\"status\":\"ok\"}".to_string());
}

#[when(expr = "I fetch admin report summary for tenant {word} from {word} to {word}")]
async fn admin_fetch_report_summary_range(
    world: &mut ApiWorld,
    tenant_id: String,
    from: String,
    to: String,
) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let token = world.admin_token.clone().expect("admin token should be set");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{base}/api/admin/reports/summary?tenant_id={tenant_id}&from={from}&to={to}"))
        .bearer_auth(token)
        .send()
        .await
        .expect("admin report summary range request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[when(expr = "I lookup i18n key {string} for locale {string}")]
async fn i18n_lookup(world: &mut ApiWorld, key: String, locale: String) {
    world.ensure_server().await;
    let base = world.base_url.as_ref().expect("base url must exist");
    let response = reqwest::get(format!("{base}/api/i18n?key={key}&locale={locale}"))
        .await
        .expect("i18n lookup request should succeed");
    world.status = Some(response.status());
    world.response_body = Some(response.text().await.expect("read response body"));
}

#[then("the response contains Celebration of Discipline")]
fn body_contains_seed(world: &mut ApiWorld) {
    assert_eq!(world.status, Some(StatusCode::OK));
    let body = world.response_body.as_ref().expect("body should exist");
    assert!(body.contains("Celebration of Discipline"));
}

#[tokio::test]
async fn bdd() {
    let writer = ApiWorld::cucumber().fail_on_skipped().run("tests/features/service").await;
    assert!(!writer.execution_has_failed(), "cucumber scenarios failed");
}
