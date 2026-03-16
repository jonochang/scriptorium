use std::path::PathBuf;
use std::time::Instant;

use axum::Router;
use bookstore_app::{
    AdminBootstrap, AdminProduct, AdminService, CatalogService, PosService, SalesEvent,
    StorefrontService,
};
use bookstore_domain::PaymentMethod;
use bookstore_web::{AppState, app};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::{Element, Page};
use futures_util::StreamExt;
use reqwest::Client;
use serial_test::serial;
use std::env;
use tokio::time::{Duration, sleep};

fn chrome_executable() -> PathBuf {
    if let Some(path) = env::var_os("CHROME_EXECUTABLE") {
        return PathBuf::from(path);
    }

    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
        "/snap/bin/chromium",
    ];

    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.exists())
        .or_else(|| which_in_path("google-chrome"))
        .or_else(|| which_in_path("chromium"))
        .or_else(|| which_in_path("chromium-browser"))
        .expect("set CHROME_EXECUTABLE or install a Chromium-compatible browser")
}

fn which_in_path(name: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    env::split_paths(&path).map(|dir| dir.join(name)).find(|candidate| candidate.exists())
}

async fn spawn_app() -> anyhow::Result<(String, AdminService)> {
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
    let router: Router = app(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    tokio::spawn(async move {
        axum::serve(listener, router).await.expect("run browser test server");
    });
    Ok((format!("http://{addr}"), admin))
}

async fn launch_browser() -> anyhow::Result<(Browser, Page)> {
    let config = BrowserConfig::builder()
        .chrome_executable(chrome_executable())
        .no_sandbox()
        .build()
        .map_err(anyhow::Error::msg)?;
    let (browser, mut handler) = Browser::launch(config).await?;
    tokio::spawn(async move { while handler.next().await.is_some() {} });
    let page = browser.new_page("about:blank").await?;
    Ok((browser, page))
}

async fn wait_for_element(page: &Page, selector: &str) -> anyhow::Result<Element> {
    let start = Instant::now();
    loop {
        match page.find_element(selector).await {
            Ok(el) => return Ok(el),
            Err(_) => {
                if start.elapsed() > Duration::from_secs(10) {
                    anyhow::bail!("timed out waiting for selector: {selector}");
                }
                sleep(Duration::from_millis(50)).await;
            }
        }
    }
}

async fn wait_for_script_truth(page: &Page, script: &str) -> anyhow::Result<()> {
    wait_for_script_truth_with_timeout(page, script, Duration::from_secs(10)).await
}

async fn wait_for_script_truth_with_timeout(
    page: &Page,
    script: &str,
    timeout: Duration,
) -> anyhow::Result<()> {
    let start = Instant::now();
    loop {
        match page.evaluate(script).await {
            Ok(result) => {
                if result.into_value::<bool>()? {
                    return Ok(());
                }
            }
            Err(_) => {
                if start.elapsed() > timeout {
                    anyhow::bail!("timed out waiting for browser condition");
                }
                sleep(Duration::from_millis(50)).await;
                continue;
            }
        }
        if start.elapsed() > timeout {
            anyhow::bail!("timed out waiting for browser condition");
        }
        sleep(Duration::from_millis(50)).await;
    }
}

async fn evaluate_string(page: &Page, script: &str) -> anyhow::Result<String> {
    Ok(page.evaluate(script).await?.into_value::<String>()?)
}

async fn set_input_value(page: &Page, selector: &str, value: &str) -> anyhow::Result<()> {
    let selector_json = serde_json::to_string(selector)?;
    let value_json = serde_json::to_string(value)?;
    let script = format!(
        r#"(function() {{
          const el = document.querySelector({selector_json});
          if (!el) return "missing";
          el.value = {value_json};
          el.dispatchEvent(new Event("input", {{ bubbles: true }}));
          el.dispatchEvent(new Event("change", {{ bubbles: true }}));
          return el.value;
        }})()"#
    );
    let result = page.evaluate(script).await?.into_value::<String>()?;
    if result == "missing" {
        anyhow::bail!("missing input element: {selector}");
    }
    Ok(())
}

async fn login_as_admin(page: &Page, base: &str, next: &str) -> anyhow::Result<()> {
    page.goto(format!("{base}/admin?next={next}")).await?;
    wait_for_element(page, "#admin-login-form").await?;
    set_input_value(page, "#admin-username", "admin").await?;
    set_input_value(page, "#admin-password", "admin123").await?;
    let login = wait_for_element(page, "#admin-login").await?;
    login.click().await?;
    wait_for_script_truth(
        page,
        &format!(
            r#"(function(){{
              return window.location.pathname === {path:?};
            }})()"#,
            path = next
        ),
    )
    .await?;
    Ok(())
}

async fn create_paid_pos_order(base: &str) -> anyhow::Result<()> {
    let client = Client::new();
    let login: serde_json::Value = client
        .post(format!("{base}/api/pos/login"))
        .json(&serde_json::json!({ "pin": "1234" }))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let token = login["session_token"].as_str().unwrap_or_default();
    anyhow::ensure!(!token.is_empty(), "missing POS session token");

    client
        .post(format!("{base}/api/pos/scan"))
        .json(&serde_json::json!({ "session_token": token, "barcode": "9780060652937" }))
        .send()
        .await?
        .error_for_status()?;

    client
        .post(format!("{base}/api/pos/payments/external-card"))
        .json(&serde_json::json!({
            "session_token": token,
            "external_ref": "browser-admin-orders"
        }))
        .send()
        .await?
        .error_for_status()?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_cart_wasm_module_loads() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/catalog")).await?;
    wait_for_script_truth(
        &page,
        r#"(function(){return window.__SCRIPTORIUM_CART_READY === true;})()"#,
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_catalog_add_updates_cart_badge() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/catalog")).await?;
    wait_for_script_truth(
        &page,
        r#"(function(){return window.__SCRIPTORIUM_CART_READY === true;})()"#,
    )
    .await?;
    let add_button = wait_for_element(&page, "[data-add-book-id='bk-100']").await?;
    add_button.click().await?;
    wait_for_script_truth(
        &page,
        r#"(function(){return document.getElementById('site-cart-count')?.textContent === '1';})()"#,
    )
    .await?;

    let feedback = evaluate_string(
        &page,
        r#"(function(){return document.getElementById('catalog-feedback')?.textContent || "";})()"#,
    )
    .await?;
    assert!(feedback.contains("Added 1 to cart"));
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_catalog_card_link_opens_product_detail() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/catalog")).await?;
    page.evaluate(
        r#"(function(){
          const link = document.querySelector('.catalog-card__link');
          if (!link) return false;
          link.click();
          return true;
        })()"#,
    )
    .await?;
    sleep(Duration::from_millis(250)).await;
    wait_for_element(&page, ".product-summary").await?;
    let title = evaluate_string(
        &page,
        r#"(function(){return document.querySelector('.product-summary h1, .display-title, .section-title')?.textContent || "";})()"#,
    )
    .await?;
    assert!(!title.trim().is_empty());
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_cart_hides_titles_already_in_basket() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/catalog")).await?;
    page.evaluate(
        r#"(function(){
          localStorage.setItem("scriptorium-storefront-cart", JSON.stringify([
            {id:"bk-100", title:"The Purpose Driven Life", author:"Rick Warren", price_cents:1899, quantity:1}
          ]));
          return true;
        })()"#,
    )
    .await?;
    page.goto(format!("{base}/cart")).await?;
    wait_for_script_truth(
        &page,
        r#"(function(){
          const row = document.querySelector('[data-recommendation-book-id="bk-100"]');
          return !!row && row.hidden === true;
        })()"#,
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_checkout_updates_summary_and_advances_to_payment() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/catalog")).await?;
    page.evaluate(
        r#"(function(){
          localStorage.setItem("scriptorium-storefront-cart", JSON.stringify([
            {id:"bk-100", title:"The Purpose Driven Life", author:"Rick Warren", price_cents:1899, quantity:1},
            {id:"bk-101", title:"Knowing God", author:"J.I. Packer", price_cents:2099, quantity:1},
            {id:"bk-102", title:"Celebration of Discipline", author:"Richard Foster", price_cents:1699, quantity:1}
          ]));
          return true;
        })()"#,
    )
    .await?;

    page.goto(format!("{base}/checkout")).await?;
    wait_for_script_truth(
        &page,
        "window.__SCRIPTORIUM_CHECKOUT_READY === true",
    )
    .await?;
    wait_for_element(&page, "#checkout-step-details.is-active").await?;

    let initial_total = evaluate_string(
        &page,
        r#"(function(){return document.getElementById('checkout-total')?.textContent || "";})()"#,
    )
    .await?;
    assert_eq!(initial_total.trim(), "$60.96");

    page.evaluate(
        r#"(function(){
          document.querySelector('[data-delivery-option="shipping"]')?.click();
          document.querySelector('[data-support-amount="500"]')?.click();
          return true;
        })()"#,
    )
    .await?;

    wait_for_script_truth(
        &page,
        r#"(function(){
          const shipping = document.getElementById('checkout-shipping')?.textContent || "";
          const support = document.getElementById('checkout-donation')?.textContent || "";
          const total = document.getElementById('checkout-total')?.textContent || "";
          const trust = document.getElementById('checkout-trust-delivery')?.textContent || "";
          return shipping.includes('$5.99') &&
            support.includes('$5.00') &&
            total.includes('$71.95') &&
            trust.includes('Shipped to your address');
        })()"#,
    )
    .await?;

    let continue_button = wait_for_element(&page, "#checkout-continue").await?;
    continue_button.click().await?;

    wait_for_script_truth(
        &page,
        r#"(function(){
          const payment = document.getElementById('checkout-step-payment');
          const card = document.getElementById('checkout-card-number');
          return payment?.classList.contains('is-active') && !!card && !card.disabled;
        })()"#,
    )
    .await?;

    set_input_value(&page, "#checkout-card-number", "4242424242424242").await?;
    set_input_value(&page, "#checkout-card-expiry", "1234").await?;
    set_input_value(&page, "#checkout-card-cvc", "987").await?;

    wait_for_script_truth(
        &page,
        r#"(function(){
          const card = document.getElementById('checkout-card-number')?.value || "";
          const expiry = document.getElementById('checkout-card-expiry')?.value || "";
          const cvc = document.getElementById('checkout-card-cvc')?.value || "";
          const submit = document.getElementById('checkout-submit-label')?.textContent || "";
          return card === '4242 4242 4242 4242' &&
            expiry === '12 / 34' &&
            cvc === '987' &&
            submit.includes('$71.95');
        })()"#,
    )
    .await?;

    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_admin_login_loads_dashboard_data() -> anyhow::Result<()> {
    let (base, admin) = spawn_app().await?;
    admin
        .upsert_product(AdminProduct {
            tenant_id: "church-a".to_string(),
            product_id: "bk-100".to_string(),
            title: "The Purpose Driven Life".to_string(),
            isbn: "9780310337508".to_string(),
            category: "Discipleship".to_string(),
            vendor: "Church Supplier".to_string(),
            cost_cents: 900,
            retail_cents: 1899,
            cover_image_key: None,
        })
        .await?;
    let (_browser, page) = launch_browser().await?;

    login_as_admin(&page, &base, "/admin").await?;
    wait_for_script_truth(
        &page,
        r#"(function(){
          const products = document.getElementById('admin-products')?.textContent || "";
          const status = document.getElementById('admin-status')?.textContent || "";
          return products.includes('The Purpose Driven Life') &&
            !status.includes('Login failed') &&
            !status.includes('Sign in first');
        })()"#,
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_admin_orders_page_shows_row_actions() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    create_paid_pos_order(&base).await?;
    let (_browser, page) = launch_browser().await?;

    login_as_admin(&page, &base, "/admin/orders").await?;
    wait_for_script_truth_with_timeout(
        &page,
        r#"(function(){
          const view = document.querySelector('#admin-orders button[onclick*="viewOrder"]');
          const resend = document.querySelector('#admin-orders button[onclick*="resendReceipt"]');
          const status = document.getElementById('admin-status')?.textContent || '';
          return !!view && !!resend && status.includes('Dashboard refreshed');
        })()"#,
        Duration::from_secs(20),
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_admin_dashboard_renders_payment_breakdown_and_low_stock() -> anyhow::Result<()> {
    let (base, admin) = spawn_app().await?;
    admin
        .upsert_product(AdminProduct {
            tenant_id: "church-a".to_string(),
            product_id: "bk-low".to_string(),
            title: "Low Stock Title".to_string(),
            isbn: "9780310337508".to_string(),
            category: "Books".to_string(),
            vendor: "Church Supplier".to_string(),
            cost_cents: 900,
            retail_cents: 1899,
            cover_image_key: None,
        })
        .await?;
    admin.receive_inventory("church-a", "9780310337508", 2).await?;
    admin
        .record_sales_event(SalesEvent {
            tenant_id: "church-a".to_string(),
            payment_method: PaymentMethod::OnlineCard,
            sales_cents: 2417,
            donations_cents: 0,
            cogs_cents: 0,
            occurred_at: chrono::NaiveDate::from_ymd_opt(2026, 3, 9)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        })
        .await;
    let (_browser, page) = launch_browser().await?;

    login_as_admin(&page, &base, "/admin").await?;
    wait_for_script_truth(
        &page,
        r#"(function(){
          const payments = document.getElementById('admin-payment-breakdown')?.textContent || '';
          const lowStock = document.getElementById('admin-low-stock')?.textContent || '';
          return payments.toLowerCase().includes('online card') &&
            lowStock.includes('Low Stock Title') &&
            lowStock.includes('2 left');
        })()"#,
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_admin_intake_save_receives_initial_stock() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    login_as_admin(&page, &base, "/admin/intake").await?;
    wait_for_script_truth(
        &page,
        r#"(function(){
          const status = document.getElementById('intake-auth-status')?.textContent || '';
          return status.includes('Signed in');
        })()"#,
    )
    .await?;
    set_input_value(&page, "#isbn", "9780060652937").await?;
    let lookup = wait_for_element(&page, "#lookup").await?;
    lookup.click().await?;
    wait_for_script_truth(
        &page,
        r#"(function(){
          const title = document.getElementById('title')?.value || '';
          return title.includes('Celebration of Discipline');
        })()"#,
    )
    .await?;
    let save = wait_for_element(&page, "#save-product").await?;
    save.click().await?;
    wait_for_script_truth(
        &page,
        r#"(function(){
          const status = document.getElementById('intake-lookup-status')?.textContent || '';
          return status.includes('on hand 5');
        })()"#,
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_pos_flow_reaches_completion_screen() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/pos")).await?;
    wait_for_element(&page, ".pin-dots").await?;
    for key in ["1", "2", "3", "4"] {
        page.evaluate(format!(
            r#"(function(){{
              const button=[...document.querySelectorAll('button')].find((el)=>el.textContent?.trim()==={key:?});
              if(button) button.click();
              return !!button;
            }})()"#
        ))
        .await?;
    }
    wait_for_element(&page, ".basket-card").await?;
    page.evaluate(
        r#"(function(){
          const button=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('Scan to cart'));
          if(button) button.click();
          return !!button;
        })()"#,
    )
    .await?;
    wait_for_element(&page, ".cart-row").await?;
    let checkout = wait_for_element(&page, ".pos-wrap > button.pos-button--lg").await?;
    checkout.click().await?;
    let card = wait_for_element(&page, ".payment-option").await?;
    card.click().await?;
    page.evaluate(
        r#"(function(){
          const button=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('Payment Received'));
          if(button) button.click();
          return !!button;
        })()"#,
    )
    .await?;
    wait_for_element(&page, ".complete-screen").await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_pos_payment_screen_shows_total_and_round_up_action() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/pos")).await?;
    wait_for_element(&page, ".pin-dots").await?;
    for key in ["1", "2", "3", "4"] {
        page.evaluate(format!(
            r#"(function(){{
              const button=[...document.querySelectorAll('button')].find((el)=>el.textContent?.trim()==={key:?});
              if(button) button.click();
              return !!button;
            }})()"#
        ))
        .await?;
    }
    wait_for_element(&page, ".basket-card").await?;
    page.evaluate(
        r#"(function(){
          const quick=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('Quick Items'));
          if(quick) quick.click();
          return !!quick;
        })()"#,
    )
    .await?;
    wait_for_element(&page, ".quick-grid").await?;
    page.evaluate(
        r#"(function(){
          const button=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('Prayer Card'));
          if(button) button.click();
          return !!button;
        })()"#,
    )
    .await?;
    wait_for_element(&page, ".cart-row").await?;
    page.evaluate(
        r#"(function(){
          const checkout=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('Checkout'));
          if(checkout) checkout.click();
          return !!checkout;
        })()"#,
    )
    .await?;
    wait_for_script_truth(
        &page,
        r#"(function(){
          return document.body.textContent.includes('Total Due');
        })()"#,
    )
    .await?;
    page.evaluate(
        r#"(function(){
          const button=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('Cash'));
          if(button) button.click();
          return !!button;
        })()"#,
    )
    .await?;
    wait_for_script_truth(
        &page,
        r#"(function(){ return document.body.textContent.includes('Round Up / Donate'); })()"#,
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_pos_forgot_pin_opens_help_state() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/pos")).await?;
    wait_for_element(&page, ".pin-dots").await?;
    page.evaluate(
        r#"(function(){
          const button=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('Forgot PIN?'));
          if(button) button.click();
          return !!button;
        })()"#,
    )
    .await?;
    wait_for_script_truth(
        &page,
        r#"(function(){
          const text = document.body.textContent || '';
          return text.includes('PIN recovery') && text.includes('1234') && text.includes('Back to keypad');
        })()"#,
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[serial]
async fn browser_pos_discount_changes_amount_due() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/pos")).await?;
    wait_for_element(&page, ".pin-dots").await?;
    for key in ["1", "2", "3", "4"] {
        page.evaluate(format!(
            r#"(function(){{
              const button=[...document.querySelectorAll('button')].find((el)=>el.textContent?.trim()==={key:?});
              if(button) button.click();
              return !!button;
            }})()"#
        ))
        .await?;
    }
    wait_for_element(&page, ".basket-card").await?;
    page.evaluate(
        r#"(function(){
          const scan=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('Scan to cart'));
          if (scan) scan.click();
          return !!scan;
        })()"#,
    )
    .await?;
    wait_for_element(&page, ".cart-row").await?;
    page.evaluate(
        r#"(function(){
          const discount=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('10% Clergy'));
          if (discount) discount.click();
          return !!discount;
        })()"#,
    )
    .await?;
    wait_for_script_truth(
        &page,
        r#"(function(){ return document.body.textContent.includes('clergy'); })()"#,
    )
    .await?;
    page.evaluate(
        r#"(function(){
          const checkout=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('Checkout'));
          if(checkout) checkout.click();
          return !!checkout;
        })()"#,
    )
    .await?;
    wait_for_script_truth(
        &page,
        r#"(function(){
          const text = document.body.textContent || '';
          return text.includes('$15.29') && text.includes('$1.70');
        })()"#,
    )
    .await?;
    Ok(())
}
