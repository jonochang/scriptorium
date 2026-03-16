pub fn admin_dashboard_script() -> &'static str {
    r#"<script type="module">import init from '/static/wasm/bookstore-cart-wasm.js'; init();</script>"#
}
