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
