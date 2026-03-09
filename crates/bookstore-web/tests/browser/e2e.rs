use std::path::PathBuf;
use std::time::Instant;

use axum::Router;
use bookstore_app::{AdminProduct, AdminService, CatalogService, PosService, StorefrontService};
use bookstore_web::{AppState, app};
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::{Element, Page};
use futures_util::StreamExt;
use serial_test::serial;
use tokio::time::{Duration, sleep};

fn chrome_executable() -> PathBuf {
    std::env::var_os("CHROME_EXECUTABLE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"))
}

async fn spawn_app() -> anyhow::Result<(String, AdminService)> {
    let admin = AdminService::new();
    let state = AppState {
        catalog: CatalogService::with_seed(),
        pos: PosService::with_seed(),
        storefront: StorefrontService::new(),
        admin: admin.clone(),
        db_pool: None,
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
    tokio::spawn(async move {
        while handler.next().await.is_some() {}
    });
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
    let start = Instant::now();
    loop {
        if page.evaluate(script).await?.into_value::<bool>()? {
            return Ok(());
        }
        if start.elapsed() > Duration::from_secs(10) {
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

#[tokio::test]
#[serial]
async fn browser_catalog_add_updates_cart_badge() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/catalog")).await?;
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
        })
        .await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/admin")).await?;
    set_input_value(&page, "#admin-password", "admin123").await?;
    let login = wait_for_element(&page, "#admin-login").await?;
    login.click().await?;
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
async fn browser_pos_flow_reaches_completion_screen() -> anyhow::Result<()> {
    let (base, _admin) = spawn_app().await?;
    let (_browser, page) = launch_browser().await?;

    page.goto(format!("{base}/pos")).await?;
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
          const button=[...document.querySelectorAll('button')].find((el)=>el.textContent?.includes('Prayer Card'));
          if(button) button.click();
          return !!button;
        })()"#,
    )
    .await?;
    let checkout = wait_for_element(&page, ".pos-wrap > button.pos-button--lg").await?;
    checkout.click().await?;
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
