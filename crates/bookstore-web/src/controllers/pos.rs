use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Html;
use bookstore_app::{PosPaymentOutcome, SalesEvent};
use bookstore_domain::{OrderChannel, OrderStatus, PaymentMethod};
use std::time::Instant;

use crate::AppState;
use crate::models::{
    ApiError, PosCashPaymentRequest, PosExternalCardRequest, PosIouRequest, PosLoginRequest,
    PosLoginResponse, PosQuickItemRequest, PosCartQuantityRequest, PosResponse, PosScanRequest,
};
use crate::web_support::{current_utc_datetime, log_checkout_event, pos_cart_response};

pub async fn pos_shell() -> Html<&'static str> {
    Html(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>Scriptorium POS</title>
  <link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;500;600;700&family=DM+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet">
  <style>
    :root {
      --wine: #6B2737;
      --wine-light: #8B3A4A;
      --wine-dark: #4A1A26;
      --gold: #B8903A;
      --gold-light: #CCAA5E;
      --gold-pale: #F5ECD7;
      --parchment: #FAF7F2;
      --parchment-dark: #EDE8E0;
      --ink: #2C1810;
      --ink-light: #5A4A3A;
      --warm-gray: #8A7A6A;
      --warm-gray-light: #B5A898;
      --success: #5A7D5E;
      --success-light: #EEF3EE;
      --warning: #A07040;
      --warning-light: #F5EDE3;
      --danger: #9B5A5A;
      --danger-light: #F5EDED;
      --blue: #5A7A9B;
      --blue-light: #ECF1F5;
      --radius: 12px;
      --radius-lg: 16px;
      --shadow: 0 4px 18px rgba(44,24,16,.10);
      --shadow-lg: 0 10px 32px rgba(44,24,16,.18);
    }
    * { box-sizing: border-box; }
    body {
      margin: 0;
      font-family: "DM Sans", sans-serif;
      background:
        radial-gradient(circle at top, rgba(204,170,94,.18), transparent 26%),
        linear-gradient(180deg, var(--wine-dark), var(--wine) 34%, #55202d 100%);
      color: #fff;
      min-height: 100vh;
    }
    .pos-shell {
      min-height: 100vh;
      padding: 18px 14px 28px;
      display: flex;
      justify-content: center;
    }
    .pos-wrap {
      width: 100%;
      max-width: 460px;
      display: grid;
      gap: 14px;
    }
    .card {
      background: var(--parchment);
      color: var(--ink);
      border-radius: var(--radius-lg);
      padding: 16px;
      box-shadow: var(--shadow);
    }
    .pos-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 14px;
      padding: 16px 18px;
      border-radius: var(--radius-lg);
      background: linear-gradient(135deg, rgba(107,39,55,.96), rgba(139,58,74,.9));
      box-shadow: var(--shadow-lg);
    }
    .pos-header__brand {
      display: grid;
      gap: 4px;
    }
    .pos-header__brand-mark {
      color: rgba(245,236,215,.42);
      font-size: 1.6rem;
      line-height: 1;
    }
    .pos-header__title {
      margin: 0;
      font-family: "Crimson Pro", serif;
      font-size: 1.7rem;
      color: var(--gold-light);
      letter-spacing: .05em;
      text-transform: uppercase;
    }
    .pos-header__subtitle {
      color: rgba(255,255,255,.68);
      font-size: .8rem;
      letter-spacing: .24em;
      text-transform: uppercase;
    }
    .pos-header__meta {
      display: flex;
      gap: 10px;
      align-items: center;
      flex-wrap: wrap;
      justify-content: end;
    }
    .pos-header__back {
      min-height: 38px;
      padding: 0 12px;
      border-radius: 999px;
      border: 1px solid rgba(255,255,255,.18);
      background: rgba(255,255,255,.08);
      color: white;
      font: 700 .86rem/1 "DM Sans", sans-serif;
    }
    .session-row {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
    }
    .session-pill {
      display: inline-flex;
      align-items: center;
      gap: 8px;
      padding: 8px 12px;
      border-radius: 999px;
      background: rgba(255,255,255,.12);
      font-size: .85rem;
      color: rgba(255,255,255,.9);
    }
    .section-title {
      margin: 0 0 12px;
      font-family: "Crimson Pro", serif;
      font-size: 1.45rem;
    }
    .subtle {
      margin: 6px 0 0;
      color: var(--warm-gray);
      font-size: .9rem;
      line-height: 1.5;
    }
    .center-shell {
      min-height: calc(100vh - 46px);
      display: flex;
      flex-direction: column;
      justify-content: center;
      gap: 18px;
    }
    .pin-head {
      text-align: center;
      padding: 10px 10px 0;
      position: relative;
    }
    .pin-cross {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      width: 40px;
      height: 40px;
      margin-bottom: 14px;
      color: rgba(255,255,255,.35);
      font-size: 2rem;
      line-height: 1;
    }
    .pin-head h1 {
      margin: 0;
      font-family: "Crimson Pro", serif;
      font-size: 2.2rem;
      color: var(--gold-light);
      letter-spacing: .05em;
    }
    .pin-head p {
      margin: 6px 0 0;
      color: rgba(255,255,255,.66);
    }
    .pin-links {
      display: flex;
      justify-content: center;
      gap: 14px;
      flex-wrap: wrap;
      margin-top: 14px;
    }
    .pin-links a {
      color: rgba(255,255,255,.78);
      text-decoration: none;
      font-size: .9rem;
    }
    .pin-card {
      background: rgba(250,247,242,.12);
      border: 1px solid rgba(255,255,255,.12);
      border-radius: 24px;
      backdrop-filter: blur(16px);
      box-shadow: var(--shadow-lg);
      padding: 18px;
    }
    .pin-dots {
      display: flex;
      justify-content: center;
      gap: 14px;
      margin: 4px 0 20px;
    }
    .pin-dot {
      width: 18px;
      height: 18px;
      border-radius: 50%;
      border: 2px solid var(--gold-light);
      background: transparent;
      transition: all .16s ease;
    }
    .pin-dot--filled { background: var(--gold-light); }
    .pin-grid {
      display: grid;
      grid-template-columns: repeat(3, 1fr);
      gap: 12px;
    }
    .pin-key {
      min-height: 72px;
      border-radius: 18px;
      border: 0;
      background: rgba(255,255,255,.1);
      color: white;
      font: 700 1.75rem/1 "DM Sans", sans-serif;
      box-shadow: inset 0 1px 0 rgba(255,255,255,.08);
    }
    .pin-key--ghost {
      font-size: 1rem;
      color: rgba(255,255,255,.74);
    }
    .status-panel {
      min-height: 86px;
      padding: 14px;
      border-radius: 14px;
      border: 1px solid var(--parchment-dark);
      background: #fff;
    }
    .status-panel h3 {
      margin: 0 0 6px;
      font-size: 1rem;
    }
    .status-panel p {
      margin: 0;
      color: var(--ink-light);
      line-height: 1.45;
    }
    .status-success { background: var(--success-light); border-color: rgba(90,125,94,.25); }
    .status-warning { background: var(--warning-light); border-color: rgba(160,112,64,.22); }
    .status-danger { background: var(--danger-light); border-color: rgba(155,90,90,.24); }
    .toolbar {
      display: flex;
      gap: 8px;
      background: white;
      padding: 6px;
      border-radius: 16px;
      border: 1px solid var(--parchment-dark);
    }
    .toolbar button {
      flex: 1;
      min-height: 42px;
      border-radius: 12px;
      border: 0;
      font: 700 .95rem/1 "DM Sans", sans-serif;
      color: var(--warm-gray);
      background: transparent;
    }
    .toolbar button.is-active {
      background: var(--gold-pale);
      color: var(--wine-dark);
      box-shadow: inset 0 -3px 0 var(--gold);
    }
    .field-label {
      display: block;
      margin: 0 0 8px;
      font-size: .9rem;
      font-weight: 600;
      color: var(--ink-light);
    }
    input {
      width: 100%;
      min-height: 46px;
      border-radius: 10px;
      border: 1px solid var(--parchment-dark);
      padding: 10px 12px;
      background: #fff;
      color: var(--ink);
      font: 500 16px/1.2 "DM Sans", sans-serif;
    }
    .pos-button--lg {
      width: 100%;
      min-height: 58px;
      border: 0;
      border-radius: var(--radius);
      font-size: 17px;
      font-weight: 700;
      background: var(--wine);
      color: #fff;
      margin: 0;
      box-shadow: 0 4px 12px rgba(107,39,55,.22);
    }
    .pos-button--gold {
      background: var(--gold);
      box-shadow: 0 4px 12px rgba(184,144,58,.22);
    }
    .pos-button--success {
      background: var(--success);
      box-shadow: 0 4px 12px rgba(90,125,94,.24);
    }
    .pos-button--ghost {
      background: white;
      color: var(--ink);
      border: 1px solid var(--parchment-dark);
      box-shadow: none;
    }
    .pos-button--light {
      background: white;
      color: var(--success);
      box-shadow: none;
    }
    .row {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 10px;
    }
    .scan-frame {
      min-height: 176px;
      border-radius: 18px;
      background: linear-gradient(180deg, rgba(44,24,16,.92), rgba(44,24,16,.78));
      position: relative;
      overflow: hidden;
      margin-bottom: 12px;
    }
    .scan-frame::before {
      content: "";
      position: absolute;
      inset: 24px;
      border-radius: 18px;
      border: 2px solid rgba(204,170,94,.45);
    }
    .scan-frame::after {
      content: "";
      position: absolute;
      left: 18%;
      right: 18%;
      top: 50%;
      height: 2px;
      background: var(--gold);
      box-shadow: 0 0 18px rgba(204,170,94,.48);
      animation: scanline 2.4s ease-in-out infinite;
    }
    @keyframes scanline {
      0%,100% { transform: translateY(-52px); }
      50% { transform: translateY(52px); }
    }
    .scan-caption {
      position: absolute;
      left: 0;
      right: 0;
      bottom: 16px;
      text-align: center;
      color: rgba(255,255,255,.6);
      font-size: .86rem;
    }
    .quick-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 10px;
    }
    .quick-tile {
      border: 1px solid var(--parchment-dark);
      border-radius: 16px;
      background: linear-gradient(180deg, #fff, var(--gold-pale));
      color: var(--ink);
      min-height: 112px;
      padding: 14px;
      text-align: left;
      font: 700 1rem/1.2 "DM Sans", sans-serif;
      position: relative;
    }
    .quick-emoji {
      display: block;
      font-size: 1.8rem;
      margin-bottom: 10px;
    }
    .quick-price {
      display: inline-flex;
      margin-top: 8px;
      padding: 4px 10px;
      border-radius: 999px;
      background: rgba(255,255,255,.7);
      color: var(--wine);
      font-size: .9rem;
    }
    .basket-card {
      position: sticky;
      bottom: 12px;
    }
    .cart-list {
      display: grid;
      gap: 10px;
    }
    .cart-row {
      display: grid;
      gap: 8px;
      grid-template-columns: 1fr auto;
      padding: 12px;
      border-radius: 12px;
      background: #fff;
      border: 1px solid var(--parchment-dark);
    }
    .cart-title { font-weight: 700; }
    .cart-meta { color: var(--warm-gray); font-size: .9rem; margin-top: 4px; }
    .cart-price { font-weight: 800; color: var(--wine); }
    .cart-controls {
      display: flex;
      align-items: center;
      justify-content: end;
      gap: 8px;
      margin-top: 8px;
    }
    .qty-pill {
      display: inline-flex;
      align-items: center;
      justify-content: center;
      min-width: 32px;
      min-height: 32px;
      padding: 0 10px;
      border-radius: 999px;
      background: var(--filled);
      color: var(--ink);
      font-weight: 700;
    }
    .cart-tag {
      display: inline-flex;
      align-items: center;
      min-height: 26px;
      margin-top: 8px;
      padding: 0 10px;
      border-radius: 999px;
      font-size: .76rem;
      font-weight: 700;
    }
    .cart-tag--quick { color: var(--warning); background: var(--warning-light); }
    .cart-tag--scan { color: var(--blue); background: var(--blue-light); }
    .empty-state {
      padding: 16px;
      border-radius: 12px;
      background: linear-gradient(180deg, #fff, #f7f3ec);
      border: 1px dashed var(--parchment-dark);
      color: var(--ink-light);
      text-align: center;
    }
    .totals {
      display: grid;
      gap: 10px;
      padding: 14px;
      border-radius: 14px;
      background: linear-gradient(180deg, rgba(107,39,55,.06), rgba(184,144,58,.12));
    }
    .totals-row {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 12px;
    }
    .totals-row strong {
      font-size: 1.2rem;
      color: var(--wine);
    }
    .actions { display: grid; gap: 10px; }
    .hint { margin: 0; color: var(--warm-gray); font-size: .86rem; }
    .payment-option {
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 14px;
      padding: 16px;
      border-radius: 16px;
      border: 1px solid var(--parchment-dark);
      background: white;
    }
    .payment-option__main {
      display: flex;
      align-items: center;
      gap: 14px;
      min-width: 0;
      flex: 1;
    }
    .payment-icon {
      width: 50px;
      height: 50px;
      border-radius: 50%;
      display: grid;
      place-items: center;
      font-size: 1.35rem;
    }
    .payment-title {
      display: block;
      font-weight: 800;
      color: var(--ink);
    }
    .payment-copy-stack {
      display: grid;
      gap: 4px;
      text-align: left;
    }
    .payment-copy {
      display: block;
      color: var(--warm-gray);
      font-size: .9rem;
    }
    .payment-chevron {
      color: var(--warm-gray);
      font-size: 1.4rem;
      line-height: 1;
    }
    .payment-total-card {
      padding: 18px;
      border-radius: 20px;
      background: rgba(255,255,255,.12);
      border: 1px solid rgba(255,255,255,.14);
      text-align: center;
      box-shadow: var(--shadow-lg);
    }
    .payment-total-card__label {
      color: rgba(255,255,255,.72);
      font-size: .78rem;
      font-weight: 700;
      letter-spacing: .18em;
      text-transform: uppercase;
    }
    .payment-total-card__value {
      margin-top: 10px;
      font-size: 3.2rem;
      font-weight: 800;
      line-height: 1;
      color: white;
    }
    .cash-grid {
      display: grid;
      grid-template-columns: repeat(2, minmax(0, 1fr));
      gap: 10px;
      margin-top: 14px;
    }
    .cash-grid button {
      min-height: 76px;
      border-radius: 14px;
      border: 1px solid var(--parchment-dark);
      background: white;
      color: var(--ink);
      font: 800 1.2rem/1 "DM Sans", sans-serif;
    }
    .cash-grid span {
      display: block;
      margin-top: 6px;
      color: var(--warm-gray);
      font-size: .78rem;
      font-weight: 600;
    }
    .round-up-button {
      width: 100%;
      min-height: 56px;
      border-radius: 16px;
      border: 1px dashed rgba(160,112,64,.42);
      background: var(--gold-pale);
      color: var(--warning);
      font: 800 .96rem/1 "DM Sans", sans-serif;
    }
    .round-up-button--active {
      background: rgba(184,144,58,.18);
      border-style: solid;
      color: var(--wine);
    }
    .complete-screen {
      min-height: calc(100vh - 46px);
      display: flex;
      flex-direction: column;
      justify-content: center;
      gap: 18px;
      text-align: center;
      padding: 24px;
      border-radius: 28px;
      background: linear-gradient(180deg, #5A7D5E 0%, #6f9a74 100%);
      color: white;
    }
    .complete-mark {
      width: 88px;
      height: 88px;
      margin: 0 auto;
      border-radius: 50%;
      display: grid;
      place-items: center;
      background: rgba(255,255,255,.14);
      box-shadow: var(--shadow-lg);
    }
    .complete-mark span {
      width: 60px;
      height: 60px;
      border-radius: 50%;
      display: grid;
      place-items: center;
      background: white;
      color: var(--success);
      font-size: 2rem;
      font-weight: 800;
    }
    .complete-title {
      margin: 0;
      font: 800 2rem/1 "DM Sans", sans-serif;
      letter-spacing: .08em;
    }
    .receipt-card {
      background: rgba(255,255,255,.12);
      border: 1px solid rgba(255,255,255,.14);
      border-radius: 18px;
      padding: 18px;
    }
    .receipt-row {
      display: flex;
      justify-content: space-between;
      gap: 12px;
      padding: 8px 0;
      color: rgba(255,255,255,.78);
    }
    .receipt-row strong { color: white; }
    .receipt-row--big strong {
      font-size: 2.4rem;
      line-height: 1;
    }
    .discount-grid {
      display: grid;
      grid-template-columns: repeat(3, minmax(0, 1fr));
      gap: 10px;
      margin-top: 14px;
    }
    .discount-chip {
      min-height: 56px;
      border-radius: 14px;
      border: 1px dashed rgba(184,144,58,.5);
      background: white;
      color: var(--wine);
      font: 700 .92rem/1.1 "DM Sans", sans-serif;
    }
    .discount-chip--active {
      background: var(--gold-pale);
      border-style: solid;
    }
    .top-actions {
      display: flex;
      gap: 10px;
      flex-wrap: wrap;
    }
    .top-actions button {
      flex: 1;
    }
  </style>
</head>
<body>
  <div id="app"></div>
  <script type="module">
    import { h, render } from "https://esm.sh/preact@10.25.4";
    import htm from "https://esm.sh/htm@3.1.1";
    import { useState } from "https://esm.sh/preact@10.25.4/hooks";

    const html = htm.bind(h);

    const QUICK_ITEMS = [
      { itemId: "prayer-card-50c", label: "Prayer Card", emoji: "🙏", priceLabel: "$0.50", note: "Pocket devotion" },
      { itemId: "votive-candle", label: "Votive Candle", emoji: "🕯️", priceLabel: "$1.00", note: "Shrine shelf" },
      { itemId: "charcoal-pack", label: "Charcoal", emoji: "🔥", priceLabel: "$2.50", note: "Thurible refill" },
      { itemId: "incense-sachet", label: "Incense", emoji: "🌿", priceLabel: "$4.50", note: "Home blessing" },
      { itemId: "small-icon", label: "Small Icon", emoji: "🖼️", priceLabel: "$12.00", note: "Gift table" },
      { itemId: "holy-water-bottle", label: "Holy Water", emoji: "💧", priceLabel: "$3.00", note: "Travel bottle" },
      { itemId: "bookmark", label: "Bookmark", emoji: "📑", priceLabel: "$1.50", note: "Reader keepsake" },
      { itemId: "greeting-card", label: "Greeting Card", emoji: "✉️", priceLabel: "$3.50", note: "Feast day note" },
    ];

    function App() {
      const [screen, setScreen] = useState("login");
      const [mode, setMode] = useState("scan");
      const [pin, setPin] = useState("");
      const [token, setToken] = useState("");
      const [barcode, setBarcode] = useState("9780060652937");
      const [cart, setCart] = useState([]);
      const [total, setTotal] = useState(0);
      const [status, setStatus] = useState({
        tone: "warning",
        title: "Shift not started",
        detail: "Enter the four-digit PIN to open the parish till.",
      });
      const [paymentMethod, setPaymentMethod] = useState("");
      const [customTendered, setCustomTendered] = useState("20.00");
      const [donateChange, setDonateChange] = useState(true);
      const [iouName, setIouName] = useState("John Doe");
      const [receiptEmail, setReceiptEmail] = useState("jane@example.com");
      const [discountCode, setDiscountCode] = useState("");
      const [lastSale, setLastSale] = useState(null);

      const money = (cents) => `$${(Number(cents || 0) / 100).toFixed(2)}`;
      const discountRate = discountCode === "clergy" ? 0.10 : discountCode === "volunteer" ? 0.15 : discountCode === "bulk" ? 0.20 : 0;
      const discountValue = Math.round(total * discountRate);
      const amountDue = Math.max(total - discountValue, 0);

      const applyCart = (payload) => {
        setCart(Array.isArray(payload.items) ? payload.items : []);
        setTotal(Number.isFinite(payload.total_cents) ? payload.total_cents : 0);
      };

      const setUiStatus = (tone, title, detail) => {
        setStatus({ tone, title, detail });
      };

      const request = async (url, payload) => {
        const res = await fetch(url, {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(payload),
        });
        const json = await res.json().catch(() => ({}));
        if (!res.ok) {
          setUiStatus("danger", json.error || "Request failed", json.message || "The POS endpoint returned an error.");
          return { ok: false, json };
        }
        return { ok: true, json };
      };

      const resetSale = () => {
        setCart([]);
        setTotal(0);
        setPaymentMethod("");
        setDonateChange(true);
        setIouName("John Doe");
        setReceiptEmail("jane@example.com");
        setDiscountCode("");
        setLastSale(null);
        setUiStatus("warning", "Ready for next customer", "Scan a title or tap a quick item to build the next basket.");
        setScreen("main");
      };

      const startShift = async (pinValue) => {
        const result = await request("/api/pos/login", { pin: pinValue });
        if (!result.ok) {
          setPin("");
          return;
        }
        const nextToken = result.json.session_token || "";
        setToken(nextToken);
        setPin("");
        resetSale();
        setUiStatus("success", "Shift started", nextToken ? `Session ${nextToken} is ready for scanning, baskets, and payment.` : "POS session opened.");
      };

      const pushDigit = (digit) => {
        if (pin.length >= 4) return;
        const next = `${pin}${digit}`;
        setPin(next);
        if (next.length === 4) {
          window.setTimeout(() => startShift(next), 220);
        }
      };

      const removeDigit = () => {
        setPin((current) => current.slice(0, -1));
      };

      const openPinHelp = () => {
        setUiStatus(
          "warning",
          "PIN help",
          "For local testing, use 1234. In parish use, ask an admin to reset the volunteer PIN before opening the till."
        );
        setScreen("help");
      };

      const scanItem = async () => {
        if (!token) {
          setUiStatus("danger", "Shift missing", "Start a shift before scanning items.");
          setScreen("login");
          return;
        }
        const result = await request("/api/pos/scan", { session_token: token, isbn: barcode });
        if (!result.ok) return;
        applyCart(result.json);
        setLastSale(null);
        setUiStatus("success", "Scanned to cart", result.json.message || "The item was added to the current sale.");
      };

      const addQuickItem = async (item) => {
        const result = await request("/api/pos/cart/items", { session_token: token, item_id: item.itemId, quantity: 1 });
        if (!result.ok) return;
        applyCart(result.json);
        setLastSale(null);
        setUiStatus("success", "Quick item added", result.json.message || `${item.label} was added to the basket.`);
      };

      const changeCartQuantity = async (item, delta) => {
        const nextQuantity = Math.max(0, Number(item.quantity || 0) + delta);
        const result = await request("/api/pos/cart/quantity", {
          session_token: token,
          item_id: item.item_id,
          quantity: nextQuantity,
        });
        if (!result.ok) return;
        applyCart(result.json);
        setUiStatus("success", "Basket updated", nextQuantity ? `${item.title} quantity is now ${nextQuantity}.` : `${item.title} was removed from the basket.`);
      };

      const beginCheckout = () => {
        if (!total) {
          setUiStatus("warning", "Basket empty", "Scan an item or tap a quick tile before opening payment options.");
          return;
        }
        setPaymentMethod("");
        setScreen("payment");
      };

      const completeCard = async () => {
        const result = await request("/api/pos/payments/external-card", {
          session_token: token,
          external_ref: "square-ui-posh",
          discount_cents: discountValue,
        });
        if (!result.ok) return;
        finalizeSale(result.json, "Card sale complete");
      };

      const completeCash = async (tenderedCents) => {
        const result = await request("/api/pos/payments/cash", {
          session_token: token,
          tendered_cents: tenderedCents,
          donate_change: donateChange,
          discount_cents: discountValue,
        });
        if (!result.ok) return;
        finalizeSale(result.json, "Cash sale complete");
      };

      const completeIou = async () => {
        const result = await request("/api/pos/payments/iou", {
          session_token: token,
          customer_name: iouName,
          discount_cents: discountValue,
        });
        if (!result.ok) return;
        finalizeSale(result.json, "Sale moved to IOU");
      };

      const finalizeSale = (payload, fallbackTitle) => {
        setLastSale(payload);
        setCart([]);
        setTotal(0);
        const tone = payload.status === "iou" ? "warning" : "success";
        const detailParts = [
          `Total ${money(payload.total_cents || 0)}`,
          payload.discount_cents ? `Discount ${money(payload.discount_cents)}` : "",
          payload.change_due_cents ? `Change ${money(payload.change_due_cents)}` : "",
          payload.donation_cents ? `Donation ${money(payload.donation_cents)}` : "",
        ].filter(Boolean);
        setUiStatus(tone, payload.message || fallbackTitle, detailParts.join(" · ") || "Payment completed.");
        setScreen("complete");
      };

      const cashPresets = [
        { label: money(amountDue), cents: amountDue, note: "Exact" },
        { label: "$20.00", cents: 2000, note: "Quick cash" },
        { label: "$50.00", cents: 5000, note: "Notes" },
        { label: "$100.00", cents: 10000, note: "Large note" },
      ].filter((option) => option.cents >= amountDue && amountDue > 0);

      const statusClass = `status-panel ${status.tone === "success" ? "status-success" : status.tone === "danger" ? "status-danger" : "status-warning"}`;

      if (screen === "login") {
        return html`
          <main class="pos-shell">
            <div class="pos-wrap center-shell">
              <section class="pin-head">
                <div class="pin-cross">✠</div>
                <h1>SCRIPTORIUM</h1>
                <p>Point of Sale</p>
              </section>
              <section class="pin-card">
                <div class="pin-dots" aria-label="Enter PIN">
                  ${[0, 1, 2, 3].map((index) => html`<span class=${`pin-dot ${index < pin.length ? "pin-dot--filled" : ""}`}></span>`)}
                </div>
                <div class="pin-grid">
                  ${["1", "2", "3", "4", "5", "6", "7", "8", "9", "", "0", "⌫"].map((key) => {
                    if (!key) return html`<div></div>`;
                    return html`<button class=${`pin-key ${key === "⌫" ? "pin-key--ghost" : ""}`} onClick=${() => key === "⌫" ? removeDigit() : pushDigit(key)}>${key}</button>`;
                  })}
                </div>
              </section>
              <section class=${statusClass}>
                <h3>${status.title}</h3>
                <p>${status.detail}</p>
              </section>
              <div class="pin-links">
                <button type="button" class="ghost-link" onClick=${openPinHelp}>Forgot PIN?</button>
                <a href="/admin">Admin login</a>
              </div>
            </div>
          </main>
        `;
      }

      if (screen === "help") {
        return html`
          <main class="pos-shell">
            <div class="pos-wrap center-shell">
              <section class="pin-head">
                <div class="pin-cross">✠</div>
                <h1>SCRIPTORIUM</h1>
                <p>PIN recovery</p>
              </section>
              <section class="pin-card">
                <div class="pilgrim-panel">
                  <h3>Forgot the shift PIN?</h3>
                  <p>For local testing, the demo PIN is <strong>1234</strong>.</p>
                  <p>For live parish use, open the admin area to rotate volunteer access before the next shift begins.</p>
                </div>
                <div class="button-row">
                  <button class="primary-button" type="button" onClick=${() => setScreen("login")}>Back to keypad</button>
                  <a class="ghost-link ghost-link--ink" href="/admin">Open admin sign-in</a>
                </div>
              </section>
              <section class=${statusClass}>
                <h3>${status.title}</h3>
                <p>${status.detail}</p>
              </section>
            </div>
          </main>
        `;
      }

      if (screen === "payment") {
        return html`
          <main class="pos-shell">
            <div class="pos-wrap">
              <section class="pos-header">
                <div class="pos-header__brand">
                  <span class="pos-header__brand-mark">☦</span>
                  <h1 class="pos-header__title">Scriptorium</h1>
                  <span class="pos-header__subtitle">Payment</span>
                </div>
                <div class="pos-header__meta">
                  <button class="pos-header__back" onClick=${() => setScreen("main")}>← Basket</button>
                  <span class="session-pill">${cart.length} line item(s)</span>
                </div>
              </section>
              <section class="payment-total-card">
                <div class="payment-total-card__label">Total Due</div>
                <div class="payment-total-card__value">${money(amountDue)}</div>
                ${discountCode && html`<div class="session-row" style=${{ justifyContent: "center", marginTop: "14px" }}><span class="session-pill">Discount selected ${money(discountValue)} (${discountCode})</span></div>`}
              </section>
              ${!paymentMethod && html`
                <section class="card actions">
                  <button class="payment-option" onClick=${() => setPaymentMethod("card")}>
                    <span class="payment-option__main">
                      <span class="payment-icon" style=${{ background: "var(--blue-light)" }}>💳</span>
                      <span class="payment-copy-stack">
                        <span class="payment-title">Credit / Debit Card</span>
                        <span class="payment-copy">Use the external terminal, then confirm the sale back at the counter.</span>
                      </span>
                    </span>
                    <span class="payment-chevron">›</span>
                  </button>
                  <button class="payment-option" onClick=${() => setPaymentMethod("cash")}>
                    <span class="payment-option__main">
                      <span class="payment-icon" style=${{ background: "var(--success-light)" }}>💵</span>
                      <span class="payment-copy-stack">
                        <span class="payment-title">Cash</span>
                        <span class="payment-copy">Use quick tender buttons, calculate change, and invite a round-up gift.</span>
                      </span>
                    </span>
                    <span class="payment-chevron">›</span>
                  </button>
                  <button class="payment-option" onClick=${() => setPaymentMethod("iou")}>
                    <span class="payment-option__main">
                      <span class="payment-icon" style=${{ background: "var(--warning-light)" }}>🧾</span>
                      <span class="payment-copy-stack">
                        <span class="payment-title">Put on Tab / IOU</span>
                        <span class="payment-copy">Record the customer name and follow up later from the admin queue.</span>
                      </span>
                    </span>
                    <span class="payment-chevron">›</span>
                  </button>
                </section>
              `}
              ${paymentMethod === "card" && html`
                <section class="card">
                  <h2 class="section-title">Card handoff</h2>
                  <p class="subtle">Open the terminal, take the card, then return here to confirm the payment.</p>
                  <div class="totals" style=${{ marginTop: "14px" }}>
                    <div class="totals-row"><span>Cart subtotal</span><strong>${money(total)}</strong></div>
                    ${discountCode && html`<div class="totals-row"><span>Discount selected</span><span>${money(discountValue)} (${discountCode})</span></div>`}
                    <div class="totals-row"><span>Amount due</span><strong>${money(amountDue)}</strong></div>
                    <div class="totals-row"><span>Provider</span><span>Square handoff</span></div>
                  </div>
                  <div class="actions" style=${{ marginTop: "14px" }}>
                    <button class="pos-button--lg" onClick=${completeCard}>Payment Received</button>
                    <button class="pos-button--lg pos-button--ghost" onClick=${() => setPaymentMethod("")}>Back to methods</button>
                  </div>
                </section>
              `}
              ${paymentMethod === "cash" && html`
                <section class="card">
                  <h2 class="section-title">Cash tendered</h2>
                  <p class="subtle">Choose a quick amount or type the amount tendered at the counter.</p>
                  <div class="cash-grid">
                    ${cashPresets.map((option) => html`
                      <button onClick=${() => completeCash(option.cents)}>
                        ${option.label}
                        <span>${option.note}</span>
                      </button>
                    `)}
                  </div>
                  <div style=${{ marginTop: "14px" }}>
                    <label class="field-label" for="custom-tendered">Custom cash amount</label>
                    <input id="custom-tendered" value=${customTendered} onInput=${(event) => setCustomTendered(event.target.value)} />
                  </div>
                  <button class=${`round-up-button ${donateChange ? "round-up-button--active" : ""}`} style=${{ marginTop: "14px" }} onClick=${() => setDonateChange((current) => !current)}>
                    ${donateChange ? "Round Up / Donate change is on" : "Round Up / Donate"}
                  </button>
                  <div class="actions" style=${{ marginTop: "14px" }}>
                    <button class="pos-button--lg" onClick=${() => completeCash(Math.round(Number(customTendered || 0) * 100))}>Use custom amount</button>
                    <button class="pos-button--lg pos-button--ghost" onClick=${() => setPaymentMethod("")}>Back to methods</button>
                  </div>
                </section>
              `}
              ${paymentMethod === "iou" && html`
                <section class="card">
                  <h2 class="section-title">Record IOU</h2>
                  <p class="subtle">This order will appear in the admin queue until the customer pays.</p>
                  <label class="field-label" for="iou-name">Customer name</label>
                  <input id="iou-name" value=${iouName} onInput=${(event) => setIouName(event.target.value)} />
                  <div class="actions" style=${{ marginTop: "14px" }}>
                    <button class="pos-button--lg pos-button--gold" onClick=${completeIou}>Record IOU</button>
                    <button class="pos-button--lg pos-button--ghost" onClick=${() => setPaymentMethod("")}>Back to methods</button>
                  </div>
                </section>
              `}
              <section class=${statusClass}>
                <h3>${status.title}</h3>
                <p>${status.detail}</p>
              </section>
              <button class="pos-button--lg pos-button--ghost" onClick=${() => setScreen("main")}>Back to basket</button>
            </div>
          </main>
        `;
      }

      if (screen === "complete") {
        const sale = lastSale || {};
        return html`
          <main class="pos-shell">
            <div class="pos-wrap complete-screen">
              <div class="complete-mark"><span>✓</span></div>
              <h1 class="complete-title">SALE COMPLETE</h1>
              <section class="receipt-card">
                <div class="receipt-row"><span>Payment outcome</span><strong>${sale.status === "iou" ? "IOU recorded" : "Paid"}</strong></div>
                <div class="receipt-row"><span>Order total</span><strong>${money(sale.total_cents || 0)}</strong></div>
                <div class="receipt-row"><span>Discount</span><strong>${money(sale.discount_cents || 0)}</strong></div>
                <div class=${`receipt-row ${sale.change_due_cents ? "receipt-row--big" : ""}`}><span>Change due</span><strong>${money(sale.change_due_cents || 0)}</strong></div>
                <div class="receipt-row"><span>Donation</span><strong>${money(sale.donation_cents || 0)}</strong></div>
              </section>
              <section class="receipt-card">
                <label class="field-label" for="receipt-email" style=${{ color: "white", textAlign: "left" }}>Email receipt</label>
                <div class="row">
                  <input id="receipt-email" value=${receiptEmail} onInput=${(event) => setReceiptEmail(event.target.value)} />
                  <button class="pos-button--lg" onClick=${() => setUiStatus("success", "Receipt queued", receiptEmail ? `Receipt will be sent to ${receiptEmail}.` : "Add an email to send a receipt.")}>Send receipt</button>
                </div>
              </section>
              <section class=${statusClass}>
                <h3>${status.title}</h3>
                <p>${status.detail}</p>
              </section>
              <button class="pos-button--lg pos-button--light" onClick=${resetSale}>Start next sale →</button>
            </div>
          </main>
        `;
      }

      return html`
        <main class="pos-shell">
          <div class="pos-wrap">
            <section class="pos-header">
              <div class="pos-header__brand">
                <span class="pos-header__brand-mark">☦</span>
                <h1 class="pos-header__title">Scriptorium</h1>
                <span class="pos-header__subtitle">Point of Sale</span>
              </div>
              <div class="pos-header__meta">
                <span class="session-pill">${token ? `Shift ${token}` : "Shift offline"}</span>
                <span class="session-pill">${cart.length ? `${cart.length} item(s)` : "Awaiting first item"}</span>
              </div>
            </section>
            <section class="card">
              <div class="toolbar">
                <button class=${mode === "scan" ? "is-active" : ""} onClick=${() => setMode("scan")}>Scan Item</button>
                <button class=${mode === "quick" ? "is-active" : ""} onClick=${() => setMode("quick")}>Quick Items</button>
              </div>
              ${mode === "scan" ? html`
                <div style=${{ marginTop: "14px" }}>
                  <div class="scan-frame"><div class="scan-caption">Point camera at ISBN, EAN-13, or typed barcode</div></div>
                  <label class="field-label" for="barcode">ISBN / barcode</label>
                  <input id="barcode" value=${barcode} onInput=${(event) => setBarcode(event.target.value)} />
                  <div class="actions" style=${{ marginTop: "10px" }}>
                    <button class="pos-button--lg" onClick=${scanItem}>Scan to cart</button>
                    <p class="hint">Use the camera lane or type the barcode when labels are faint.</p>
                  </div>
                </div>
              ` : html`
                <div class="quick-grid" style=${{ marginTop: "14px" }}>
                  ${QUICK_ITEMS.map((item) => html`
                    <button class="quick-tile" onClick=${() => addQuickItem(item)}>
                      <span class="quick-emoji">${item.emoji}</span>
                      ${item.label}
                      <span class="quick-price">${item.priceLabel}</span>
                    </button>
                  `)}
                </div>
              `}
            </section>
            <section class="card basket-card">
              <h2 class="section-title">Basket</h2>
              ${cart.length ? html`
                <div class="cart-list">
                  ${cart.map((item) => html`
                    <div class="cart-row" key=${item.item_id}>
                      <div>
                        <div class="cart-title">${item.title}</div>
                        <div class="cart-meta">${item.is_quick_item ? "Quick item" : "Scanned title"}</div>
                        <div class="cart-controls">
                          <button class="ghost-link ghost-link--ink ghost-link--mini" onClick=${() => changeCartQuantity(item, -1)}>−</button>
                          <span class="qty-pill">Qty ${item.quantity}</span>
                          <button class="ghost-link ghost-link--ink ghost-link--mini" onClick=${() => changeCartQuantity(item, 1)}>+</button>
                        </div>
                        <span class=${`cart-tag ${item.is_quick_item ? "cart-tag--quick" : "cart-tag--scan"}`}>${item.is_quick_item ? "Quick item" : "Scanned item"}</span>
                      </div>
                      <div class="cart-price">${money(item.unit_price_cents * item.quantity)}</div>
                    </div>
                  `)}
                </div>
              ` : html`<div class="empty-state">Cart empty. Scan an item or use a quick tile to start the sale.</div>`}
              <div class="totals" style=${{ marginTop: "12px" }}>
                <div class="totals-row"><span>Current total</span><strong>${money(total)}</strong></div>
                ${discountCode && html`<div class="totals-row"><span>Discount selected</span><span>${money(discountValue)} (${discountCode})</span></div>`}
                <div class="totals-row"><span>Amount due</span><strong>${money(amountDue)}</strong></div>
                <div class="totals-row"><span>Checkout path</span><span>Card, cash, or IOU</span></div>
              </div>
              <div class="discount-grid">
                ${[
                  ["", "No discount"],
                  ["clergy", "10% Clergy"],
                  ["volunteer", "15% Volunteer"],
                  ["bulk", "20% Bulk"],
                ].map(([code, label]) => html`<button class=${`discount-chip ${discountCode===code?"discount-chip--active":""}`} onClick=${() => setDiscountCode(code)}>${label}</button>`)}
              </div>
            </section>
            <section class=${statusClass}>
              <h3>${status.title}</h3>
              <p>${status.detail}</p>
            </section>
            <button class="pos-button--lg" onClick=${beginCheckout}>Checkout · ${money(total)}</button>
          </div>
        </main>
      `;
    }

    render(html`<${App} />`, document.getElementById("app"));
  </script>
</body>
</html>"#,
    )
}


pub async fn pos_login(
    State(state): State<AppState>,
    Json(request): Json<PosLoginRequest>,
) -> Result<Json<PosLoginResponse>, ApiError> {
    let session_token = state
        .pos
        .login_with_pin(&request.pin)
        .await
        .map_err(|err| ApiError::new(StatusCode::UNAUTHORIZED, err.to_string()))?;
    Ok(Json(PosLoginResponse { session_token }))
}

pub async fn pos_scan(
    State(state): State<AppState>,
    Json(request): Json<PosScanRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let snapshot = state
        .pos
        .scan_item(&request.session_token, &request.barcode)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    Ok(Json(pos_cart_response(snapshot, "Item added to cart")))
}

pub async fn pos_quick_item(
    State(state): State<AppState>,
    Json(request): Json<PosQuickItemRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let snapshot = state
        .pos
        .add_quick_item(&request.session_token, &request.item_id, request.quantity)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    Ok(Json(pos_cart_response(snapshot, "Quick item added to cart")))
}

pub async fn pos_set_cart_quantity(
    State(state): State<AppState>,
    Json(request): Json<PosCartQuantityRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let snapshot = state
        .pos
        .set_cart_quantity(&request.session_token, &request.item_id, request.quantity)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    Ok(Json(pos_cart_response(snapshot, "Basket updated")))
}

pub async fn pos_pay_cash(
    State(state): State<AppState>,
    Json(request): Json<PosCashPaymentRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let started_at = Instant::now();
    let receipt = state
        .pos
        .checkout_cash(
            &request.session_token,
            request.tendered_cents,
            request.donate_change,
            request.discount_cents,
        )
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    let now = current_utc_datetime();
    state
        .admin
        .record_sales_event(SalesEvent {
            tenant_id: "church-a".to_string(),
            payment_method: PaymentMethod::Cash,
            sales_cents: receipt.total_cents,
            donations_cents: receipt.donation_cents,
            cogs_cents: receipt.total_cents / 2,
            occurred_at: now,
        })
        .await;
    state
        .admin
        .create_order(
            "church-a",
            "Walk In",
            OrderChannel::Pos,
            OrderStatus::Paid,
            PaymentMethod::Cash,
            receipt.total_cents,
            now,
        )
        .await;
    log_checkout_event("pos_checkout", "sale_complete", "cash", receipt.total_cents, started_at);
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::Paid { "sale_complete" } else { "iou" },
        message: if receipt.donation_cents > 0 {
            "Cash sale complete with donated change".to_string()
        } else {
            "Cash sale complete".to_string()
        },
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
        discount_cents: receipt.discount_cents,
        items: Vec::new(),
    }))
}

pub async fn pos_pay_external_card(
    State(state): State<AppState>,
    Json(request): Json<PosExternalCardRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let started_at = Instant::now();
    let receipt = state
        .pos
        .checkout_external_card(
            &request.session_token,
            &request.external_ref,
            request.discount_cents,
        )
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    let now = current_utc_datetime();
    state
        .admin
        .record_sales_event(SalesEvent {
            tenant_id: "church-a".to_string(),
            payment_method: PaymentMethod::ExternalCard,
            sales_cents: receipt.total_cents,
            donations_cents: receipt.donation_cents,
            cogs_cents: receipt.total_cents / 2,
            occurred_at: now,
        })
        .await;
    state
        .admin
        .create_order(
            "church-a",
            "Walk In",
            OrderChannel::Pos,
            OrderStatus::Paid,
            PaymentMethod::ExternalCard,
            receipt.total_cents,
            now,
        )
        .await;
    log_checkout_event(
        "pos_checkout",
        "sale_complete",
        "external_card",
        receipt.total_cents,
        started_at,
    );
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::Paid { "sale_complete" } else { "iou" },
        message: "Card sale complete".to_string(),
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
        discount_cents: receipt.discount_cents,
        items: Vec::new(),
    }))
}

pub async fn pos_pay_iou(
    State(state): State<AppState>,
    Json(request): Json<PosIouRequest>,
) -> Result<Json<PosResponse>, ApiError> {
    let started_at = Instant::now();
    let receipt = state
        .pos
        .checkout_iou(&request.session_token, &request.customer_name, request.discount_cents)
        .await
        .map_err(|err| ApiError::new(StatusCode::BAD_REQUEST, err.to_string()))?;
    state
        .admin
        .create_order(
            "church-a",
            &request.customer_name,
            OrderChannel::Pos,
            OrderStatus::UnpaidIou,
            PaymentMethod::Iou,
            receipt.total_cents,
            current_utc_datetime(),
        )
        .await;
    log_checkout_event("pos_checkout", "iou", "iou", receipt.total_cents, started_at);
    Ok(Json(PosResponse {
        status: if receipt.outcome == PosPaymentOutcome::UnpaidIou {
            "iou"
        } else {
            "sale_complete"
        },
        message: "Sale moved to IOU".to_string(),
        total_cents: receipt.total_cents,
        change_due_cents: receipt.change_due_cents,
        donation_cents: receipt.donation_cents,
        discount_cents: receipt.discount_cents,
        items: Vec::new(),
    }))
}
