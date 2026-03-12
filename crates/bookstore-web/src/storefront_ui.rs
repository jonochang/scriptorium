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
function selectedSupport(){return Number(document.querySelector('[data-support-amount].is-selected')?.dataset.supportAmount||0);}
function selectedDelivery(){return document.querySelector('[data-delivery-option].is-selected')?.dataset.deliveryOption||'pickup';}
function shippingCents(subtotal){if(subtotal<=0)return 0;return selectedDelivery()==='shipping'?599:0;}
function taxCents(subtotal){return Math.round(subtotal*0.07);}
function checkoutState(){
  const cart=readCart();
  const subtotal=cartSubtotal(cart);
  const shipping=shippingCents(subtotal);
  const tax=taxCents(subtotal);
  const support=selectedSupport();
  return {cart,subtotal,shipping,tax,support,total:subtotal+shipping+tax+support};
}
function cardDigits(){return String(document.getElementById('checkout-card-number')?.value||'').replace(/\D/g,'');}
function canContinue(){
  return readCart().length>0&&document.getElementById('checkout-name')?.value.trim()&&document.getElementById('checkout-email')?.value.trim();
}
function canSubmit(){
  return canContinue()&&cardDigits().length>=15;
}
function setStatus(message,variant){
  const panel=document.getElementById('checkout-status');
  if(!panel)return;
  panel.textContent=message;
  panel.className=`notice-panel${variant?` notice-panel--${variant}`:''}`;
}
function setDelivery(delivery){
  document.querySelectorAll('[data-delivery-option]').forEach((option)=>{
    const active=option.dataset.deliveryOption===delivery;
    option.classList.toggle('is-selected',active);
    option.setAttribute('aria-pressed',active?'true':'false');
  });
  const addressSection=document.getElementById('checkout-address-section');
  if(addressSection)addressSection.hidden=delivery!=='shipping';
}
function setSupport(amount){
  document.querySelectorAll('[data-support-amount]').forEach((button)=>{
    const active=Number(button.dataset.supportAmount||0)===amount;
    button.classList.toggle('is-selected',active);
    button.setAttribute('aria-pressed',active?'true':'false');
  });
}
let checkoutStep=0;
let placingOrder=false;
function syncStepUI(){
  const detailsActive=checkoutStep===0;
  const paymentActive=checkoutStep===1;
  document.getElementById('checkout-step-details')?.classList.toggle('is-active',detailsActive);
  document.getElementById('checkout-step-details')?.classList.toggle('is-done',checkoutStep>0);
  document.getElementById('checkout-step-payment')?.classList.toggle('is-active',paymentActive);
  document.getElementById('checkout-step-confirmation')?.classList.toggle('is-active',false);
  const detailsPanel=document.getElementById('checkout-panel-details');
  const paymentPanel=document.getElementById('checkout-panel-payment');
  if(detailsPanel)detailsPanel.hidden=!detailsActive;
  if(paymentPanel)paymentPanel.hidden=!paymentActive;
  const title=document.getElementById('checkout-lead');
  if(title)title.textContent=detailsActive?'Confirm your details and delivery preference.':'Enter payment to complete your order.';
}
function renderTrustSignals(email,delivery){
  const receipt=document.getElementById('checkout-trust-receipt');
  const deliveryNode=document.getElementById('checkout-trust-delivery');
  if(receipt)receipt.textContent=`Receipt sent to ${email||'your email'}`;
  if(deliveryNode)deliveryNode.textContent=delivery==='shipping'?'Shipped to your address':'Pick up at church after liturgy';
}
function renderCheckout(){
  const {cart,subtotal,shipping,tax,support,total}=checkoutState();
  updateCartCount(cart);
  const lines=document.getElementById('checkout-lines');
  document.getElementById('checkout-subtotal').textContent=money(subtotal);
  document.getElementById('checkout-shipping').textContent=shipping?money(shipping):'Free';
  document.getElementById('checkout-tax').textContent=money(tax);
  document.getElementById('checkout-donation').textContent=money(support);
  document.getElementById('checkout-total').textContent=money(total);
  const continueButton=document.getElementById('checkout-continue');
  if(continueButton)continueButton.disabled=!canContinue();
  const submitButton=document.getElementById('create-checkout-session');
  if(submitButton)submitButton.disabled=!canSubmit()||placingOrder;
  const submitLabel=document.getElementById('checkout-submit-label');
  if(submitLabel)submitLabel.textContent=placingOrder?'Placing order...':`Place Order — ${money(total)}`;
  renderTrustSignals(document.getElementById('checkout-email')?.value.trim(),selectedDelivery());
  if(!cart.length){lines.innerHTML='<div class="empty-inline">Your cart is empty.</div>';return total;}
  lines.innerHTML=cart.map(item=>`<div class="checkout-summary-line"><div><div class="list-title">${item.title}</div><div class="list-meta">${item.author} · Qty ${item.quantity}</div></div><strong>${money(Number(item.price_cents||0)*Number(item.quantity||0))}</strong></div>`).join('');
  return total;
}
function goToStep(step){
  if(step===1&&!canContinue()){renderCheckout();return;}
  checkoutStep=step;
  syncStepUI();
  renderCheckout();
}
function formatCard(value){
  const digits=String(value||'').replace(/\D/g,'').slice(0,16);
  return digits.replace(/(.{4})/g,'$1 ').trim();
}
function formatExpiry(value){
  const digits=String(value||'').replace(/\D/g,'').slice(0,4);
  if(digits.length>2)return `${digits.slice(0,2)} / ${digits.slice(2)}`;
  return digits;
}
async function createCheckoutSession(){
  const state=checkoutState();
  const email=document.getElementById('checkout-email').value.trim();
  const name=document.getElementById('checkout-name').value.trim();
  const delivery=selectedDelivery();
  const lineItems=state.cart.filter(item=>Number(item.quantity||0)>0).map(item=>({item_id:item.id,quantity:Number(item.quantity||0)}));
  if(!state.total||!lineItems.length){setStatus('Add at least one title before placing the order.','danger');return;}
  if(!name){setStatus('Please enter your name.','danger');return;}
  if(!email){setStatus('Please enter a receipt email.','danger');return;}
  if(delivery==='shipping'){const addr=document.getElementById('checkout-address').value.trim();if(!addr){setStatus('Please enter a shipping address.','danger');return;}}
  if(cardDigits().length<15){setStatus('Enter a valid card number to continue.','danger');return;}
  placingOrder=true;
  renderCheckout();
  setStatus('Placing your order...');
  const res=await fetch('/api/storefront/checkout/session',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({email,customer_name:name,delivery_method:delivery,donation_cents:state.support,line_items:lineItems})});
  const json=await res.json().catch(()=>({}));
  placingOrder=false;
  renderCheckout();
  if(!res.ok){setStatus(json.message||json.error||'Order failed. Please try again.','danger');return;}
  writeCart([]);
  window.location.href='/orders?placed='+encodeURIComponent(json.order_id||json.session_id);
}
document.getElementById('checkout-continue')?.addEventListener('click',()=>goToStep(1));
document.getElementById('checkout-back-to-details')?.addEventListener('click',()=>goToStep(0));
document.querySelectorAll('[data-delivery-option]').forEach((option)=>{
  option.addEventListener('click',()=>{setDelivery(option.dataset.deliveryOption||'pickup');renderCheckout();});
});
document.querySelectorAll('[data-support-amount]').forEach((button)=>{
  button.addEventListener('click',()=>{setSupport(Number(button.dataset.supportAmount||0));renderCheckout();});
});
document.getElementById('checkout-name')?.addEventListener('input',renderCheckout);
document.getElementById('checkout-email')?.addEventListener('input',renderCheckout);
document.getElementById('checkout-address')?.addEventListener('input',renderCheckout);
document.getElementById('checkout-card-number')?.addEventListener('input',(event)=>{event.target.value=formatCard(event.target.value);renderCheckout();});
document.getElementById('checkout-card-expiry')?.addEventListener('input',(event)=>{event.target.value=formatExpiry(event.target.value);renderCheckout();});
document.getElementById('checkout-card-cvc')?.addEventListener('input',(event)=>{event.target.value=String(event.target.value||'').replace(/\D/g,'').slice(0,4);renderCheckout();});
document.getElementById('create-checkout-session')?.addEventListener('click',createCheckoutSession);
setDelivery('pickup');
setSupport(0);
syncStepUI();
setStatus('Ready to place your order.');
renderCheckout();
</script></body></html>"#
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
        padding-bottom: 20px;
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
        width: 28px;
        height: 28px;
        border-radius: 999px;
        border: 2px solid var(--parchment-dark);
        display: inline-flex;
        align-items: center;
        justify-content: center;
        font-size: 0.78rem;
        font-weight: 800;
        background: transparent;
      }
      .checkout-step__label {
        font-size: 0.84rem;
        font-weight: 700;
      }
      .checkout-step.is-active { color: var(--ink); }
      .checkout-step.is-active .checkout-step__dot {
        background: var(--wine);
        border-color: var(--wine);
        color: white;
      }
      .checkout-step.is-done .checkout-step__dot {
        background: var(--success);
        border-color: var(--success);
        color: white;
      }
      .checkout-step__rail {
        width: 32px;
        height: 2px;
        border-radius: 999px;
        background: var(--parchment-dark);
      }
      .checkout-card,
      .checkout-summary-card {
        display: grid;
        gap: 16px;
      }
      .checkout-card {
        padding: 22px 24px;
        border-radius: var(--radius);
        border: 1px solid var(--parchment-dark);
        background: white;
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
        background: rgba(107,39,55,0.06);
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
        min-width: 62px;
        padding: 8px 14px;
        border-radius: 10px;
        border: 1px solid var(--parchment-dark);
        background: var(--filled);
        color: var(--ink-light);
        font-weight: 700;
        cursor: pointer;
      }
      .checkout-support-actions button.is-selected {
        background: var(--success);
        border-color: var(--success);
        color: white;
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
        padding: 14px 16px;
        border-radius: 12px;
        background: var(--filled);
        display: grid;
        gap: 8px;
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
        padding: 11px 14px;
        border-radius: 10px;
        border: 2px solid var(--parchment-dark);
        background: white;
        color: var(--ink);
        font: inherit;
      }
      .checkout-field:focus {
        outline: none;
        border-color: var(--wine);
      }
      .checkout-field--mono {
        font-family: "IBM Plex Mono", "SFMono-Regular", Consolas, monospace;
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
        r#"<main class="page-stack page-stack--wide"><section class="checkout-layout"><article class="checkout-shell"><div class="checkout-header"><div><h1 class="section-title" style="margin-bottom:4px">Checkout</h1><p id="checkout-lead" class="checkout-copy">Confirm your details and delivery preference.</p></div><div class="checkout-steps" aria-label="Checkout progress"><div id="checkout-step-details" class="checkout-step is-active"><span class="checkout-step__dot">1</span><span class="checkout-step__label">Details</span></div><span class="checkout-step__rail"></span><div id="checkout-step-payment" class="checkout-step"><span class="checkout-step__dot">2</span><span class="checkout-step__label">Payment</span></div><span class="checkout-step__rail"></span><div id="checkout-step-confirmation" class="checkout-step"><span class="checkout-step__dot">3</span><span class="checkout-step__label">Confirmation</span></div></div></div><section id="checkout-panel-details" class="checkout-card"><h2 class="section-title">Contact &amp; delivery</h2><div class="fieldset-grid"><div><label class="field-label" for="checkout-name">Full name</label><input id="checkout-name" class="checkout-field" placeholder="Jane Parishioner" value="Jane Parishioner" /></div><div><label class="field-label" for="checkout-email">Receipt email</label><input id="checkout-email" class="checkout-field" placeholder="reader@example.com" value="jane@example.com" /></div></div><div><label class="field-label">Delivery method</label><div class="checkout-delivery-grid"><button type="button" class="checkout-choice is-selected" data-delivery-option="pickup" aria-pressed="true"><strong>Pick up from church</strong><span>Free after liturgy</span><em>Free</em></button><button type="button" class="checkout-choice" data-delivery-option="shipping" aria-pressed="false"><strong>Ship to my address</strong><span>Tracked parcel delivery</span><em>$5.99</em></button></div></div><div id="checkout-address-section" hidden><label class="field-label" for="checkout-address">Address</label><textarea id="checkout-address" class="checkout-field" placeholder="123 Parish Lane, Melbourne VIC 3000"></textarea></div><div><label class="field-label" for="checkout-note">Order note <span style="font-weight:400;color:var(--warm-gray)">optional</span></label><textarea id="checkout-note" class="checkout-field" placeholder="Parish pickup note, gifting instructions, or follow-up..."></textarea></div><section class="checkout-card" style="padding:18px 20px;background:var(--filled)"><div class="checkout-support-row"><div><h3 class="section-title" style="font-size:1rem;margin-bottom:4px">Parish support</h3><p class="checkout-copy" style="margin:0">Add a voluntary contribution to help keep the bookstore running.</p></div><div class="checkout-support-actions"><button type="button" class="is-selected" data-support-amount="0" aria-pressed="true">None</button><button type="button" data-support-amount="200" aria-pressed="false">$2</button><button type="button" data-support-amount="500" aria-pressed="false">$5</button><button type="button" data-support-amount="1000" aria-pressed="false">$10</button></div></div></section><div class="checkout-actions"><a class="ghost-link ghost-link--ink" href="/cart">Back to cart</a><button class="primary-button" type="button" id="checkout-continue">Continue to payment</button></div></section><section id="checkout-panel-payment" class="checkout-card" hidden><h2 class="section-title">Card details</h2><div><label class="field-label" for="checkout-card-number">Card number</label><input id="checkout-card-number" class="checkout-field checkout-field--mono" placeholder="0000 0000 0000 0000" inputmode="numeric" /></div><div class="checkout-payment-grid"><div><label class="field-label" for="checkout-card-expiry">Expiry</label><input id="checkout-card-expiry" class="checkout-field checkout-field--mono" placeholder="MM / YY" inputmode="numeric" /></div><div><label class="field-label" for="checkout-card-cvc">CVC</label><input id="checkout-card-cvc" class="checkout-field checkout-field--mono" placeholder="123" inputmode="numeric" /></div></div><div class="checkout-help">Secure payment processing. Card entry is handed off safely after you place the order.</div><div class="checkout-actions"><button class="ghost-link ghost-link--ink" type="button" id="checkout-back-to-details">Back to details</button><button class="primary-button primary-button--block" type="button" id="create-checkout-session"><span id="checkout-submit-label">Place Order — $0.00</span></button></div><div id="checkout-status" class="notice-panel" aria-live="polite">Ready to place your order.</div></section></article><aside class="checkout-summary-aside"><article class="surface-card checkout-summary-card"><h2 class="section-title">Order summary</h2><div id="checkout-lines" class="stack-list"><div class="empty-inline">Your cart is empty.</div></div><div class="summary-row"><span>Subtotal</span><strong id="checkout-subtotal">$0.00</strong></div><div class="summary-row"><span>Shipping</span><strong id="checkout-shipping">Free</strong></div><div class="summary-row"><span>Tax</span><strong id="checkout-tax">$0.00</strong></div><div class="summary-row"><span>Parish support</span><strong id="checkout-donation">$0.00</strong></div><div class="summary-row summary-row--total"><span>Total</span><strong id="checkout-total">$0.00</strong></div></article><div class="checkout-trust"><div><span>Secure</span><strong>Secure payment processing</strong></div><div><span>Receipt</span><strong id="checkout-trust-receipt">Receipt sent to your email</strong></div><div><span>Delivery</span><strong id="checkout-trust-delivery">Pick up at church after liturgy</strong></div></div></aside></section></main>"#,
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
