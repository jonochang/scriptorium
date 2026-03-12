pub fn storefront_cart_script() -> &'static str {
    r#"<script>
const CART_KEY = "scriptorium-storefront-cart";
function readCart() {
  try { return JSON.parse(localStorage.getItem(CART_KEY) || "[]"); } catch { return []; }
}
function writeCart(cart) {
  localStorage.setItem(CART_KEY, JSON.stringify(cart));
}
function money(cents) {
  return `$${(Number(cents || 0) / 100).toFixed(2)}`;
}
function updateCartCount(cart) {
  const count = cart.reduce((sum, item) => sum + Number(item.quantity || 0), 0);
  const badge = document.getElementById("site-cart-count");
  if (badge) badge.textContent = String(count);
}
function readAddQuantity(button) {
  const targetId = button.dataset.addBookQuantityTarget;
  if (!targetId) return 1;
  const input = document.getElementById(targetId);
  const quantity = Math.max(1, Number(input?.value || 1));
  return Number.isFinite(quantity) ? quantity : 1;
}
function addToCartFromDataset(button) {
  const cart = readCart();
  const id = button.dataset.addBookId;
  const price = Number(button.dataset.addBookPriceCents || 0);
  const quantity = readAddQuantity(button);
  const existing = cart.find((item) => item.id === id);
  if (existing) {
    existing.quantity += quantity;
  } else {
    cart.push({
      id,
      title: button.dataset.addBookTitle,
      author: button.dataset.addBookAuthor,
      price_cents: price,
      quantity,
    });
  }
  writeCart(cart);
  const feedback = document.getElementById(button.dataset.feedbackTarget || "cart-feedback");
  if (feedback) {
    feedback.textContent = `Added ${quantity} to cart. Cart now has ${cart.reduce((sum, item) => sum + item.quantity, 0)} item(s).`;
    feedback.className = "notice-panel notice-panel--success";
  }
  renderCartPage();
}
function renderCartPage() {
  const cart = readCart();
  updateCartCount(cart);
  const cartItems = document.getElementById("cart-items");
  const cartSummary = document.getElementById("cart-summary");
  if (cartItems) {
    if (!cart.length) {
      cartItems.innerHTML = '<div class="empty-inline">Your cart is empty.</div>';
    } else {
      cartItems.innerHTML = cart.map((item) => `
        <div class="list-row">
          <div>
            <div class="list-title">${item.title}</div>
            <div class="list-meta">${item.author} · Qty ${item.quantity}</div>
          </div>
          <div class="button-row button-row--compact">
            <button class="ghost-link ghost-link--ink ghost-link--mini" type="button" data-cart-decrement="${item.id}">−</button>
            <button class="ghost-link ghost-link--ink ghost-link--mini" type="button" data-cart-increment="${item.id}">+</button>
            <button class="ghost-link ghost-link--ink ghost-link--mini" type="button" data-cart-remove="${item.id}">Remove</button>
            <strong>${money(item.price_cents * item.quantity)}</strong>
          </div>
        </div>
      `).join("");
    }
  }
  if (cartSummary) {
    const total = cart.reduce((sum, item) => sum + (item.price_cents * item.quantity), 0);
    cartSummary.textContent = `Cart total: ${money(total)}`;
  }
  syncRecommendations(cart);
  bindCartControls();
}
function syncRecommendations(cart) {
  const cartIds = new Set(cart.map((item) => item.id));
  const cartTitles = new Set(cart.map((item) => String(item.title || "").trim().toLowerCase()));
  const rows = Array.from(document.querySelectorAll("[data-recommendation-book-id]"));
  let visible = 0;
  rows.forEach((row) => {
    const recommendationTitle = String(row.dataset.recommendationTitle || "").trim().toLowerCase();
    const hidden =
      cartIds.has(row.dataset.recommendationBookId) ||
      (recommendationTitle && cartTitles.has(recommendationTitle));
    row.hidden = hidden;
    row.style.display = hidden ? "none" : "";
    if (!hidden) visible += 1;
  });
  const empty = document.getElementById("cart-recommendations-empty");
  if (empty) {
    empty.hidden = visible !== 0;
  }
}
function mutateCart(id, operation) {
  const cart = readCart().map((item) => ({ ...item }));
  const entry = cart.find((item) => item.id === id);
  if (!entry) return;
  if (operation === "increment") entry.quantity += 1;
  if (operation === "decrement") entry.quantity = Math.max(0, entry.quantity - 1);
  const nextCart = operation === "remove" ? cart.filter((item) => item.id !== id) : cart.filter((item) => item.quantity > 0);
  writeCart(nextCart);
  renderCartPage();
}
function bindCartControls() {
  document.querySelectorAll("[data-cart-increment]").forEach((button) => {
    button.onclick = () => mutateCart(button.dataset.cartIncrement, "increment");
  });
  document.querySelectorAll("[data-cart-decrement]").forEach((button) => {
    button.onclick = () => mutateCart(button.dataset.cartDecrement, "decrement");
  });
  document.querySelectorAll("[data-cart-remove]").forEach((button) => {
    button.onclick = () => mutateCart(button.dataset.cartRemove, "remove");
  });
}
document.querySelectorAll("[data-add-book-id]").forEach((button) => {
  button.addEventListener("click", () => addToCartFromDataset(button));
});
const clearCartButton = document.getElementById("clear-cart");
if (clearCartButton) {
  clearCartButton.addEventListener("click", () => {
    writeCart([]);
    renderCartPage();
  });
}
renderCartPage();
</script>"#
}

pub fn storefront_checkout_script() -> &'static str {
    r#"<script>
const CART_KEY='scriptorium-storefront-cart';
function readCart(){try{return JSON.parse(localStorage.getItem(CART_KEY)||'[]');}catch{return [];}}
function writeCart(cart){localStorage.setItem(CART_KEY,JSON.stringify(cart));}
function money(cents){return `$${(Number(cents||0)/100).toFixed(2)}`;}
function updateCartCount(cart){const count=cart.reduce((sum,item)=>sum+Number(item.quantity||0),0);const badge=document.getElementById('site-cart-count');if(badge)badge.textContent=String(count);}
function cartSubtotal(cart){return cart.reduce((sum,item)=>sum+(Number(item.price_cents||0)*Number(item.quantity||0)),0);}
function currentDonation(){return Number(document.getElementById('checkout-donation-select')?.value||0);}
function selectedDelivery(){return document.querySelector('input[name="checkout-delivery"]:checked')?.value||'pickup';}
function shippingCents(subtotal){if(subtotal<=0)return 0;return selectedDelivery()==='shipping'?599:0;}
function taxCents(subtotal){return Math.round(subtotal*0.07);}
function updateDeliveryUI(){
  const isShipping=selectedDelivery()==='shipping';
  document.getElementById('checkout-address-section').hidden=!isShipping;
  const radios=document.querySelectorAll('input[name="checkout-delivery"]');
  radios.forEach(r=>{
    const label=r.closest('label');
    if(r.checked){label.style.border='2px solid #6B2737';label.style.background='rgba(107,39,55,0.06)';}
    else{label.style.border='1px solid #d5d0c8';label.style.background='';}
  });
  renderCheckout();
}
function renderCheckout(){
  const cart=readCart();updateCartCount(cart);
  const lines=document.getElementById('checkout-lines');
  const subtotal=cartSubtotal(cart);const shipping=shippingCents(subtotal);const tax=taxCents(subtotal);
  const donation=currentDonation();const total=subtotal+shipping+tax+donation;
  document.getElementById('checkout-subtotal').textContent=money(subtotal);
  document.getElementById('checkout-shipping').textContent=shipping?money(shipping):'Free';
  document.getElementById('checkout-tax').textContent=money(tax);
  document.getElementById('checkout-donation').textContent=money(donation);
  document.getElementById('checkout-total').textContent=money(total);
  const submitLabel=document.getElementById('checkout-submit-label');
  if(submitLabel)submitLabel.textContent=`Place Order — ${money(total)}`;
  if(!cart.length){lines.innerHTML='<div class="empty-inline">Your cart is empty.</div>';return total;}
  lines.innerHTML=cart.map(item=>`<div class="list-row list-row--soft"><div><div class="list-title">${item.title}</div><div class="list-meta">${item.author} · Qty ${item.quantity}</div></div><strong>${money(item.price_cents*item.quantity)}</strong></div>`).join('');
  return total;
}
async function createCheckoutSession(){
  const cart=readCart();const totalCents=renderCheckout();const donation=currentDonation();
  const email=document.getElementById('checkout-email').value.trim();
  const name=document.getElementById('checkout-name').value.trim();
  const delivery=selectedDelivery();
  const panel=document.getElementById('checkout-status');
  const lineItems=cart.filter(item=>Number(item.quantity||0)>0).map(item=>({item_id:item.id,quantity:Number(item.quantity||0)}));
  if(!totalCents||!lineItems.length){panel.textContent='Add at least one title before placing the order.';panel.className='notice-panel notice-panel--danger';return;}
  if(!name){panel.textContent='Please enter your name.';panel.className='notice-panel notice-panel--danger';return;}
  if(delivery==='shipping'){const addr=document.getElementById('checkout-address').value.trim();if(!addr){panel.textContent='Please enter a shipping address.';panel.className='notice-panel notice-panel--danger';return;}}
  panel.textContent='Placing your order...';panel.className='notice-panel';
  const res=await fetch('/api/storefront/checkout/session',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({email,customer_name:name,delivery_method:delivery,donation_cents:donation,line_items:lineItems})});
  const json=await res.json().catch(()=>({}));
  if(!res.ok){panel.textContent=json.message||json.error||'Order failed. Please try again.';panel.className='notice-panel notice-panel--danger';return;}
  writeCart([]);
  window.location.href='/orders?placed='+encodeURIComponent(json.order_id||json.session_id);
}
document.getElementById('create-checkout-session').addEventListener('click',createCheckoutSession);
document.getElementById('checkout-donation-select').addEventListener('change',renderCheckout);
document.querySelectorAll('input[name="checkout-delivery"]').forEach(r=>r.addEventListener('change',updateDeliveryUI));
updateDeliveryUI();
</script></body></html>"#
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
        "</style></head><body class=\"page-shell\">",
        site_nav,
        page_header,
        r#"<main class="page-stack page-stack--wide"><section class="checkout-layout"><article class="surface-card"><h2 class="section-title">Contact and delivery</h2><div class="fieldset-grid"><div><label class="field-label" for="checkout-name">Full name</label><input id="checkout-name" placeholder="Jane Parishioner" value="Jane Parishioner" /></div><div><label class="field-label" for="checkout-email">Receipt email</label><input id="checkout-email" placeholder="reader@example.com" value="jane@example.com" /></div></div><label class="field-label">Delivery method</label><div class="fieldset-grid" style="margin-bottom:12px"><label style="display:flex;align-items:center;gap:8px;padding:12px 14px;border:2px solid var(--wine,#6B2737);border-radius:8px;cursor:pointer;background:rgba(107,39,55,0.06)"><input type="radio" name="checkout-delivery" value="pickup" checked /> Pick up from Church <span style="margin-left:auto;font-weight:700;color:#2d6a2e">Free</span></label><label style="display:flex;align-items:center;gap:8px;padding:12px 14px;border:1px solid #d5d0c8;border-radius:8px;cursor:pointer"><input type="radio" name="checkout-delivery" value="shipping" /> Ship to my address <span style="margin-left:auto;font-weight:700">$5.99</span></label></div><div id="checkout-address-section" hidden><label class="field-label" for="checkout-address">Address</label><textarea id="checkout-address" placeholder="123 Parish Lane, Melbourne VIC"></textarea></div><label class="field-label" for="checkout-note">Order note</label><textarea id="checkout-note" placeholder="Optional note for parish pickup, gifting, or follow-up."></textarea><label class="field-label" for="checkout-donation-select">Optional parish support</label><select id="checkout-donation-select"><option value="0">No extra support</option><option value="200">Round up with $2.00</option><option value="500">Add $5.00 support</option><option value="1000">Add $10.00 support</option></select><div class="divider-title divider-title--spaced">Payment</div><div id="checkout-payment-placeholder" class="stripe-placeholder"><div class="stripe-placeholder__card"><span>4242 4242 4242 4242</span><strong>12 / 34</strong></div><p class="helper-copy helper-copy--flush">Card entry is handed off securely after you place the order.</p></div><button class="primary-button primary-button--block" type="button" id="create-checkout-session"><span id="checkout-submit-label">Place Order — $0.00</span></button><p class="helper-copy">We will confirm the session id, receipt email, and final total here before you move on.</p><div id="checkout-status" class="notice-panel" aria-live="polite">Ready to place your order.</div><div id="checkout-confirmation" class="surface-card" hidden><p class="divider-title">Order confirmation</p><h3 class="section-title">Session ready</h3><div class="stack-list stack-list--tight"><div class="list-row list-row--soft"><span>Session id</span><strong id="checkout-confirmation-session">-</strong></div><div class="list-row list-row--soft"><span>Receipt</span><strong id="checkout-confirmation-email">-</strong></div><div class="list-row list-row--soft"><span>Total handed off</span><strong id="checkout-confirmation-total">-</strong></div></div><div class="button-row"><a class="accent-button" href="/catalog">Keep shopping</a><a class="ghost-link ghost-link--ink" href="/cart">Review cart</a></div></div></article><article class="surface-card"><h2 class="section-title">Order summary</h2><div id="checkout-lines" class="stack-list"><div class="empty-inline">Your cart is empty.</div></div><div class="summary-row"><span>Subtotal</span><strong id="checkout-subtotal">$0.00</strong></div><div class="summary-row"><span>Shipping</span><strong id="checkout-shipping">$5.99</strong></div><div class="summary-row"><span>Tax</span><strong id="checkout-tax">$0.00</strong></div><div class="summary-row"><span>Parish support</span><strong id="checkout-donation">$0.00</strong></div><div class="summary-row summary-row--total"><span>Total</span><strong id="checkout-total">$0.00</strong></div></article></section></main>"#,
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
