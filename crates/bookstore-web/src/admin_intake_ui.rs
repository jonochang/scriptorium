pub fn admin_intake_script() -> &'static str {
    r#"
  <script type="module">import init from '/static/wasm/bookstore-cart-wasm.js'; init();</script>
</body>
</html>"#
}
