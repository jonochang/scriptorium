pub fn storefront_cart_script() -> &'static str {
    r#"<script type="module">import init from '/static/wasm/bookstore-cart-wasm.js'; init();</script>"#
}

pub fn storefront_checkout_script() -> &'static str {
    r#"<script type="module">import init from '/static/wasm/bookstore-cart-wasm.js'; init();</script></body></html>"#
}

fn storefront_checkout_extra_styles() -> &'static str {
    r#"
      .checkout-shell {
        display: grid;
        gap: 22px;
      }
      .checkout-header {
        display: flex;
        align-items: flex-end;
        justify-content: space-between;
        gap: 18px;
        padding-bottom: 24px;
        margin-bottom: 4px;
        border-bottom: 1px solid var(--parchment-dark);
      }
      .checkout-header p { margin: 6px 0 0; }
      .checkout-steps {
        display: flex;
        align-items: center;
        gap: 12px;
        flex-wrap: wrap;
      }
      .checkout-step {
        display: flex;
        align-items: center;
        gap: 8px;
        color: var(--warm-gray);
      }
      .checkout-step__dot {
        width: 24px;
        height: 24px;
        border-radius: 12px;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        font-size: 13px;
        font-weight: 700;
        background: var(--parchment-dark);
        color: var(--warm-gray);
        border: none;
      }
      .checkout-step__label {
        font-size: 14px;
        font-weight: 400;
      }
      .checkout-step.is-active { color: var(--ink); }
      .checkout-step.is-active .checkout-step__label { font-weight: 600; }
      .checkout-step.is-active .checkout-step__dot {
        background: var(--wine);
        color: white;
      }
      .checkout-step.is-done .checkout-step__dot {
        background: var(--wine);
        color: white;
      }
      .checkout-step__rail {
        width: 40px;
        height: 1px;
        background: var(--filled-border);
      }
      .checkout-card,
      .checkout-summary-card {
        display: grid;
        gap: 16px;
      }
      .checkout-card {
        padding: 24px 28px;
        border-radius: 12px;
        border: 1px solid var(--parchment-dark);
        background: #ffffff;
      }
      .checkout-delivery-grid,
      .checkout-payment-grid {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(2, minmax(0, 1fr));
      }
      .checkout-choice {
        display: grid;
        gap: 6px;
        padding: 14px 16px;
        border-radius: 12px;
        border: 2px solid var(--parchment-dark);
        background: transparent;
        cursor: pointer;
        text-align: left;
        transition: border-color 120ms ease, background 120ms ease;
      }
      .checkout-choice strong {
        color: var(--ink);
      }
      .checkout-choice span {
        color: var(--warm-gray);
        font-size: 0.86rem;
      }
      .checkout-choice em {
        margin-left: auto;
        font-style: normal;
        font-weight: 800;
        color: var(--success);
      }
      .checkout-choice.is-selected {
        border-color: var(--wine);
        background: var(--accent-light);
      }
      .checkout-support-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        flex-wrap: wrap;
      }
      .checkout-support-actions {
        display: flex;
        gap: 8px;
        flex-wrap: wrap;
      }
      .checkout-support-actions button {
        padding: 6px 14px;
        border-radius: 20px;
        border: 1px solid var(--filled-border);
        background: transparent;
        color: var(--ink-light);
        font-size: 13px;
        font-weight: 400;
        cursor: pointer;
        font-family: "Source Sans 3", "Segoe UI", system-ui, sans-serif;
      }
      .checkout-support-actions button.is-selected {
        background: var(--ink);
        border-color: var(--ink);
        color: white;
        font-weight: 600;
      }
      .checkout-actions {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        flex-wrap: wrap;
      }
      .checkout-summary-line {
        display: flex;
        align-items: start;
        justify-content: space-between;
        gap: 12px;
        padding: 10px 0;
        border-top: 1px solid var(--parchment-dark);
      }
      .checkout-summary-line:first-child {
        border-top: 0;
        padding-top: 0;
      }
      .checkout-trust {
        padding: 16px 18px;
        border-radius: 12px;
        background: var(--filled);
        display: grid;
        gap: 10px;
        color: var(--ink-light);
        font-size: 0.86rem;
      }
      .checkout-trust div {
        display: flex;
        align-items: center;
        gap: 8px;
      }
      .checkout-summary-aside {
        position: sticky;
        top: 24px;
        display: grid;
        gap: 12px;
      }
      .checkout-field {
        width: 100%;
        padding: 10px 14px;
        border-radius: 8px;
        border: 1px solid var(--filled-border);
        background: var(--parchment);
        color: var(--ink);
        font: 400 14px/1.4 "Source Sans 3", "Segoe UI", system-ui, sans-serif;
      }
      .checkout-field:focus {
        outline: none;
        border-color: var(--wine);
        box-shadow: 0 0 0 3px rgba(107,28,42,0.08);
      }
      .checkout-field--mono {
        font-family: "JetBrains Mono", "SF Mono", monospace;
        letter-spacing: 0.08em;
      }
      .checkout-copy {
        color: var(--warm-gray);
        font-size: 0.9rem;
      }
      .checkout-help {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 10px 14px;
        border-radius: 10px;
        background: var(--filled);
        color: var(--ink-light);
        font-size: 0.84rem;
      }
      [hidden] { display: none !important; }
      @media (max-width: 960px) {
        .checkout-header,
        .checkout-actions {
          align-items: start;
        }
        .checkout-summary-aside {
          position: static;
        }
      }
      @media (max-width: 640px) {
        .checkout-delivery-grid,
        .checkout-payment-grid {
          grid-template-columns: 1fr;
        }
      }
    "#
}

pub fn storefront_checkout_shell_html(
    google_fonts_link: &'static str,
    shared_styles: &'static str,
    site_nav: &str,
    page_header: &str,
    site_footer: &'static str,
) -> String {
    [
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Scriptorium Checkout</title>",
        google_fonts_link,
        "<style>",
        shared_styles,
        storefront_checkout_extra_styles(),
        "</style></head><body class=\"page-shell\">",
        site_nav,
        page_header,
        r#"<main class="page-stack page-stack--wide"><section class="checkout-layout"><article class="checkout-shell"><div class="checkout-header"><div class="checkout-steps" aria-label="Checkout progress"><div id="checkout-step-details" class="checkout-step is-active"><span class="checkout-step__dot">1</span><span class="checkout-step__label">Details</span></div><span class="checkout-step__rail"></span><div id="checkout-step-payment" class="checkout-step"><span class="checkout-step__dot">2</span><span class="checkout-step__label">Payment</span></div><span class="checkout-step__rail"></span><div id="checkout-step-confirmation" class="checkout-step"><span class="checkout-step__dot">3</span><span class="checkout-step__label">Confirmation</span></div></div></div><section id="checkout-panel-details" class="checkout-card"><h2 class="section-title">Contact &amp; delivery</h2><div class="fieldset-grid"><div><label class="field-label" for="checkout-name">Full name</label><input id="checkout-name" class="checkout-field" placeholder="Full name" /></div><div><label class="field-label" for="checkout-email">Receipt email</label><input id="checkout-email" class="checkout-field" placeholder="reader@example.com" /></div></div><div><label class="field-label">Delivery method</label><div class="checkout-delivery-grid"><button type="button" class="checkout-choice is-selected" data-delivery-option="pickup" aria-pressed="true"><strong>Pick up from church</strong><span>Free after liturgy</span><em>Free</em></button><button type="button" class="checkout-choice" data-delivery-option="shipping" aria-pressed="false"><strong>Ship to my address</strong><span>Tracked parcel delivery</span><em>$5.99</em></button></div></div><div id="checkout-address-section" hidden><label class="field-label" for="checkout-address">Address</label><textarea id="checkout-address" class="checkout-field" placeholder="123 Parish Lane, Melbourne VIC 3000"></textarea></div><div><label class="field-label" for="checkout-note">Order note <span style="font-weight:400;color:var(--warm-gray)">optional</span></label><textarea id="checkout-note" class="checkout-field" placeholder="Parish pickup note, gifting instructions, or follow-up..."></textarea></div><section class="checkout-card" style="padding:18px 20px;background:var(--filled)"><div class="checkout-support-row"><div><h3 class="section-title" style="font-size:1rem;margin-bottom:4px">Parish support</h3><p class="checkout-copy" style="margin:0">Add a voluntary contribution to help keep the bookstore running.</p></div><div class="checkout-support-actions"><button type="button" class="is-selected" data-support-amount="0" aria-pressed="true">None</button><button type="button" data-support-amount="200" aria-pressed="false">$2</button><button type="button" data-support-amount="500" aria-pressed="false">$5</button><button type="button" data-support-amount="1000" aria-pressed="false">$10</button></div></div></section><div class="checkout-actions"><a class="ghost-link ghost-link--ink" href="/cart">Back to cart</a><button class="primary-button" type="button" id="checkout-continue">Continue to payment</button></div></section><section id="checkout-panel-payment" class="checkout-card" hidden><h2 class="section-title">Card details</h2><div><label class="field-label" for="checkout-card-number">Card number</label><input id="checkout-card-number" class="checkout-field checkout-field--mono" placeholder="0000 0000 0000 0000" inputmode="numeric" /></div><div class="checkout-payment-grid"><div><label class="field-label" for="checkout-card-expiry">Expiry</label><input id="checkout-card-expiry" class="checkout-field checkout-field--mono" placeholder="MM / YY" inputmode="numeric" /></div><div><label class="field-label" for="checkout-card-cvc">CVC</label><input id="checkout-card-cvc" class="checkout-field checkout-field--mono" placeholder="123" inputmode="numeric" /></div></div><div class="checkout-help">Secure payment processing. Card entry is handed off safely after you place the order.</div><div class="checkout-actions"><button class="ghost-link ghost-link--ink" type="button" id="checkout-back-to-details">Back to details</button><button class="primary-button primary-button--block" type="button" id="create-checkout-session"><span id="checkout-submit-label">Place Order — $0.00</span></button></div><div id="checkout-status" class="notice-panel" aria-live="polite">Ready to place your order.</div></section></article><aside class="checkout-summary-aside"><article class="surface-card checkout-summary-card"><h2 class="section-title">Order summary</h2><div id="checkout-lines" class="stack-list"><div class="empty-inline">Your cart is empty.</div></div><div class="summary-row"><span>Subtotal</span><strong id="checkout-subtotal">$0.00</strong></div><div class="summary-row"><span>Shipping</span><strong id="checkout-shipping">Free</strong></div><div class="summary-row"><span>Tax</span><strong id="checkout-tax">$0.00</strong></div><div class="summary-row"><span>Parish support</span><strong id="checkout-donation">$0.00</strong></div><div class="summary-row summary-row--total"><span>Total</span><strong id="checkout-total">$0.00</strong></div></article><div class="checkout-trust"><div><span>Secure</span><strong>Secure payment processing</strong></div><div><span>Receipt</span><strong id="checkout-trust-receipt">Receipt sent to your email</strong></div><div><span>Delivery</span><strong id="checkout-trust-delivery">Pick up at church after liturgy</strong></div></div></aside></section></main>"#,
        site_footer,
        storefront_checkout_script(),
    ]
    .concat()
}

pub fn storefront_orders_shell_html(
    google_fonts_link: &'static str,
    shared_styles: &'static str,
    site_nav: &str,
    page_header: &str,
    site_footer: &'static str,
    placed_id: &str,
    orders: &[bookstore_app::AdminOrder],
) -> String {
    let success_banner = if !placed_id.is_empty() {
        format!(
            r#"<div class="notice-panel notice-panel--success" style="margin-bottom:16px">Order <strong>{}</strong> placed successfully! Thank you for your purchase.</div>"#,
            placed_id
        )
    } else {
        String::new()
    };

    let order_rows = if orders.is_empty() {
        r#"<div class="empty-inline">No orders yet. <a href="/catalog">Browse the catalog</a> to get started.</div>"#.to_string()
    } else {
        orders.iter().map(|o| {
            let total = format!("${:.2}", o.total_cents as f64 / 100.0);
            let status_class = if o.status == bookstore_domain::OrderStatus::Paid { "badge badge--success" } else { "badge badge--warning" };
            format!(
                r#"<div class="list-row list-row--soft"><div><div class="list-title">{order_id}</div><div class="list-meta">{customer} · {channel} · {date}</div></div><div style="display:flex;align-items:center;gap:12px"><span class="{status_class}">{status}</span><strong>{total}</strong></div></div>"#,
                order_id = o.order_id,
                customer = o.customer_name,
                channel = o.channel,
                date = o.created_at.format("%Y-%m-%d %H:%M"),
                status_class = status_class,
                status = o.status,
                total = total,
            )
        }).collect::<Vec<_>>().join("")
    };

    [
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\" /><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" /><title>Scriptorium Orders</title>",
        google_fonts_link,
        "<style>",
        shared_styles,
        "</style></head><body class=\"page-shell\">",
        site_nav,
        page_header,
        "<main class=\"page-stack page-stack--wide\"><article class=\"surface-card\">",
        &success_banner,
        "<h2 class=\"section-title\">Recent Orders</h2><div class=\"stack-list\">",
        &order_rows,
        "</div></article></main>",
        site_footer,
        "</body></html>",
    ]
    .concat()
}

#[cfg(test)]
mod tests {
    use super::storefront_checkout_shell_html;

    #[test]
    fn checkout_shell_includes_checkout_specific_style_selectors() {
        let html = storefront_checkout_shell_html("", "", "", "", "");

        assert!(html.contains(".checkout-choice.is-selected"));
        assert!(html.contains(".checkout-support-actions button.is-selected"));
        assert!(html.contains(".checkout-summary-aside"));
        assert!(html.contains("checkout-step__dot"));
    }
}
