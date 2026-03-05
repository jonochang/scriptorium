import { useState, useEffect, useRef, useCallback } from "react";

// ─── Design Tokens (softened, harmonious palette) ───
const T = {
  wine: "#6B2737",
  wineLight: "#8B3A4A",
  wineDark: "#4A1A26",
  wineMuted: "#8B6B74",
  gold: "#B8903A",
  goldLight: "#CCAA5E",
  goldPale: "#F5ECD7",
  parchment: "#FAF7F2",
  parchmentDark: "#EDE8E0",
  ink: "#2C1810",
  inkLight: "#5A4A3A",
  warmGray: "#8A7A6A",
  warmGrayLight: "#B5A898",
  white: "#FFFFFF",
  // Muted status colors — earth-toned, not saturated
  success: "#5A7D5E",
  successLight: "#EEF3EE",
  successAccent: "#4A6B4E",
  danger: "#9B5A5A",
  dangerLight: "#F5EDED",
  warning: "#A07040",
  warningLight: "#F5EDE3",
  blue: "#5A7A9B",
  blueLight: "#ECF1F5",
  // Autofill highlight — very subtle warm tint
  filled: "#F7F3EC",
  filledBorder: "#E0D8CC",
  shadow: "0 2px 12px rgba(44,24,16,0.06)",
  shadowLg: "0 8px 32px rgba(44,24,16,0.10)",
  radius: "12px",
  radiusSm: "8px",
  radiusLg: "16px",
  font: "'DM Sans', sans-serif",
  fontDisplay: "'Crimson Pro', serif",
};

// ─── Icon Components ───
const Icon = ({ name, size = 20, color = "currentColor" }) => {
  const icons = {
    scan: <path d="M3 7V5a2 2 0 0 1 2-2h2M17 3h2a2 2 0 0 1 2 2v2M21 17v2a2 2 0 0 1-2 2h-2M7 21H5a2 2 0 0 1-2-2v-2M7 12h10M12 7v10" />,
    grid: <><rect x="3" y="3" width="7" height="7" rx="1" /><rect x="14" y="3" width="7" height="7" rx="1" /><rect x="3" y="14" width="7" height="7" rx="1" /><rect x="14" y="14" width="7" height="7" rx="1" /></>,
    cart: <><circle cx="9" cy="21" r="1" /><circle cx="20" cy="21" r="1" /><path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6" /></>,
    card: <><rect x="1" y="4" width="22" height="16" rx="2" ry="2" /><line x1="1" y1="10" x2="23" y2="10" /></>,
    cash: <><rect x="2" y="6" width="20" height="12" rx="2" /><circle cx="12" cy="12" r="3" /><path d="M6 12h.01M18 12h.01" /></>,
    tab: <><path d="M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2" /><rect x="8" y="2" width="8" height="4" rx="1" ry="1" /></>,
    check: <polyline points="20 6 9 17 4 12" />,
    plus: <><line x1="12" y1="5" x2="12" y2="19" /><line x1="5" y1="12" x2="19" y2="12" /></>,
    minus: <line x1="5" y1="12" x2="19" y2="12" />,
    search: <><circle cx="11" cy="11" r="8" /><line x1="21" y1="21" x2="16.65" y2="16.65" /></>,
    orders: <><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" /><polyline points="14 2 14 8 20 8" /><line x1="16" y1="13" x2="8" y2="13" /><line x1="16" y1="17" x2="8" y2="17" /></>,
    alert: <><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" /><line x1="12" y1="9" x2="12" y2="13" /><line x1="12" y1="17" x2="12.01" y2="17" /></>,
    heart: <path d="M20.84 4.61a5.5 5.5 0 0 0-7.78 0L12 5.67l-1.06-1.06a5.5 5.5 0 0 0-7.78 7.78l1.06 1.06L12 21.23l7.78-7.78 1.06-1.06a5.5 5.5 0 0 0 0-7.78z" />,
    back: <><polyline points="15 18 9 12 15 6" /></>,
    mail: <><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z" /><polyline points="22,6 12,13 2,6" /></>,
    download: <><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" /></>,
    tag: <><path d="M20.59 13.41l-7.17 7.17a2 2 0 0 1-2.83 0L2 12V2h10l8.59 8.59a2 2 0 0 1 0 2.82z" /><line x1="7" y1="7" x2="7.01" y2="7" /></>,
    trend: <><polyline points="23 6 13.5 15.5 8.5 10.5 1 18" /><polyline points="17 6 23 6 23 12" /></>,
    clock: <><circle cx="12" cy="12" r="10" /><polyline points="12 6 12 12 16 14" /></>,
    book: <><path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20" /><path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z" /></>,
    package: <><path d="M12.89 1.45l8 4A2 2 0 0 1 22 7.24v9.53a2 2 0 0 1-1.11 1.79l-8 4a2 2 0 0 1-1.79 0l-8-4a2 2 0 0 1-1.1-1.8V7.24a2 2 0 0 1 1.11-1.79l8-4a2 2 0 0 1 1.78 0z" /><polyline points="2.32 6.16 12 11 21.68 6.16" /><line x1="12" y1="22.76" x2="12" y2="11" /></>,
    chevronRight: <polyline points="9 18 15 12 9 6" />,
    chevronLeft: <polyline points="15 18 9 12 15 6" />,
    eye: <><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" /><circle cx="12" cy="12" r="3" /></>,
  };
  return (
    <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke={color} strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
      {icons[name]}
    </svg>
  );
};

// ─── Shared Components ───
const Badge = ({ children, color = T.wineMuted, bg }) => (
  <span style={{
    display: "inline-flex", alignItems: "center", padding: "2px 10px",
    borderRadius: 20, fontSize: 12, fontWeight: 600, color,
    background: bg || `${color}14`,
  }}>{children}</span>
);

const Btn = ({ children, variant = "primary", size = "md", style, disabled, ...props }) => {
  const base = {
    display: "inline-flex", alignItems: "center", justifyContent: "center", gap: 8,
    border: "none", cursor: disabled ? "default" : "pointer", fontFamily: T.font, fontWeight: 600,
    borderRadius: T.radiusSm, transition: "all 0.15s ease", opacity: disabled ? 0.5 : 1,
  };
  const sizes = { sm: { padding: "6px 14px", fontSize: 13 }, md: { padding: "10px 20px", fontSize: 14 }, lg: { padding: "14px 28px", fontSize: 16 } };
  const variants = {
    primary: { background: T.wine, color: T.white },
    gold: { background: T.gold, color: T.white },
    success: { background: T.success, color: T.white },
    ghost: { background: "transparent", color: T.ink, border: `1.5px solid ${T.parchmentDark}` },
  };
  return <button style={{ ...base, ...sizes[size], ...variants[variant], ...style }} disabled={disabled} {...props}>{children}</button>;
};

const Card = ({ children, style, ...props }) => (
  <div style={{ background: T.white, borderRadius: T.radius, boxShadow: T.shadow, border: `1px solid ${T.parchmentDark}`, ...style }} {...props}>{children}</div>
);

// ─── Pagination ───
const Pagination = ({ page, totalPages, onPageChange }) => (
  <div style={{ display: "flex", alignItems: "center", justifyContent: "center", gap: 4, padding: "16px 0" }}>
    <button onClick={() => onPageChange(Math.max(1, page - 1))} disabled={page === 1}
      style={{ width: 36, height: 36, borderRadius: T.radiusSm, border: `1px solid ${T.parchmentDark}`, background: T.white, cursor: page === 1 ? "default" : "pointer", opacity: page === 1 ? 0.4 : 1, display: "flex", alignItems: "center", justifyContent: "center", fontFamily: T.font }}>
      <Icon name="chevronLeft" size={16} color={T.inkLight} />
    </button>
    {Array.from({ length: totalPages }, (_, i) => i + 1).map(p => (
      <button key={p} onClick={() => onPageChange(p)} style={{
        width: 36, height: 36, borderRadius: T.radiusSm,
        border: p === page ? `1.5px solid ${T.wine}` : `1px solid ${T.parchmentDark}`,
        background: p === page ? T.wine : T.white, color: p === page ? T.white : T.inkLight,
        cursor: "pointer", fontFamily: T.font, fontSize: 14, fontWeight: p === page ? 700 : 500,
        display: "flex", alignItems: "center", justifyContent: "center",
      }}>{p}</button>
    ))}
    <button onClick={() => onPageChange(Math.min(totalPages, page + 1))} disabled={page === totalPages}
      style={{ width: 36, height: 36, borderRadius: T.radiusSm, border: `1px solid ${T.parchmentDark}`, background: T.white, cursor: page === totalPages ? "default" : "pointer", opacity: page === totalPages ? 0.4 : 1, display: "flex", alignItems: "center", justifyContent: "center", fontFamily: T.font }}>
      <Icon name="chevronRight" size={16} color={T.inkLight} />
    </button>
    <span style={{ marginLeft: 12, fontSize: 13, color: T.warmGray }}>Page {page} of {totalPages}</span>
  </div>
);

// ─── SCREEN DATA ───
const quickItems = [
  { name: "Prayer Card", price: 0.50, emoji: "🙏", color: "#EDE5D8" },
  { name: "Votive Candle", price: 1.00, emoji: "🕯️", color: "#F0E8DA" },
  { name: "Charcoal (roll)", price: 3.00, emoji: "⚫", color: "#E5E2DD" },
  { name: "Incense (box)", price: 5.00, emoji: "💨", color: "#E4E2E8" },
  { name: "Small Icon", price: 8.00, emoji: "🖼️", color: "#EDE0DD" },
  { name: "Holy Water Bottle", price: 2.00, emoji: "💧", color: "#DDEAE8" },
  { name: "Bookmark", price: 1.50, emoji: "📑", color: "#E8E0EC" },
  { name: "Greeting Card", price: 3.50, emoji: "✉️", color: "#E2EAE2" },
];

const sampleCart = [
  { id: 1, name: "The Orthodox Study Bible", price: 34.95, qty: 1 },
  { id: 2, name: "Prayer Card", price: 0.50, qty: 2 },
];

const sampleProducts = [
  { id: 1, title: "The Orthodox Study Bible", author: "Thomas Nelson", price: 34.95, stock: 12, cat: "Books", img: "📖" },
  { id: 2, title: "The Way of a Pilgrim", author: "R. M. French", price: 12.99, stock: 5, cat: "Books", img: "📗" },
  { id: 3, title: "Beginning to Pray", author: "Anthony Bloom", price: 14.95, stock: 0, cat: "Books", img: "📕" },
  { id: 4, title: "Icon of Christ Pantocrator", author: "Hand-painted", price: 45.00, stock: 3, cat: "Icons", img: "🖼️" },
  { id: 5, title: "Beeswax Candles (12 pk)", author: "Monastery Supply", price: 8.99, stock: 24, cat: "Liturgical", img: "🕯️" },
  { id: 6, title: "Silver Cross Pendant", author: "St. Herman Press", price: 22.00, stock: 7, cat: "Gifts", img: "✝️" },
  { id: 7, title: "Theotokos Icon (small)", author: "Hand-painted", price: 35.00, stock: 2, cat: "Icons", img: "🖼️" },
  { id: 8, title: "Incense Starter Kit", author: "Athos Imports", price: 15.00, stock: 18, cat: "Liturgical", img: "💨" },
  { id: 9, title: "The Philokalia (Vol. 1)", author: "Various", price: 19.99, stock: 8, cat: "Books", img: "📖" },
  { id: 10, title: "Prayer Rope (100-knot)", author: "Athos Monastery", price: 12.00, stock: 15, cat: "Gifts", img: "📿" },
  { id: 11, title: "Liturgical Calendar 2026", author: "OCA Publishing", price: 9.50, stock: 6, cat: "Liturgical", img: "📅" },
  { id: 12, title: "The Arena", author: "St. Ignatius Brianchaninov", price: 16.95, stock: 4, cat: "Books", img: "📕" },
];

const sampleOrders = [
  { id: "ORD-1042", date: "Mar 2, 2026", channel: "POS", status: "Paid", total: 47.95, method: "Card", customer: "Walk-in" },
  { id: "ORD-1041", date: "Mar 2, 2026", channel: "POS", status: "IOU", total: 34.95, method: "Tab", customer: "John Doe" },
  { id: "ORD-1040", date: "Mar 2, 2026", channel: "Online", status: "Paid", total: 67.94, method: "Stripe", customer: "Maria S." },
  { id: "ORD-1039", date: "Mar 1, 2026", channel: "POS", status: "Paid", total: 12.50, method: "Cash", customer: "Walk-in" },
  { id: "ORD-1038", date: "Mar 1, 2026", channel: "Online", status: "Refunded", total: 14.95, method: "Stripe", customer: "Peter K." },
  { id: "ORD-1037", date: "Feb 28, 2026", channel: "POS", status: "Paid", total: 89.90, method: "Card", customer: "Walk-in" },
  { id: "ORD-1036", date: "Feb 28, 2026", channel: "POS", status: "IOU", total: 22.00, method: "Tab", customer: "Anna T." },
  { id: "ORD-1035", date: "Feb 27, 2026", channel: "Online", status: "Paid", total: 35.00, method: "Stripe", customer: "Elena P." },
  { id: "ORD-1034", date: "Feb 27, 2026", channel: "POS", status: "Paid", total: 15.50, method: "Cash", customer: "Walk-in" },
  { id: "ORD-1033", date: "Feb 26, 2026", channel: "POS", status: "Paid", total: 44.95, method: "Card", customer: "Walk-in" },
  { id: "ORD-1032", date: "Feb 25, 2026", channel: "Online", status: "Paid", total: 28.99, method: "Stripe", customer: "Dimitri L." },
  { id: "ORD-1031", date: "Feb 24, 2026", channel: "POS", status: "Paid", total: 8.50, method: "Cash", customer: "Walk-in" },
];

// ═══════════════════════════════════════════
// SCREEN 1: PIN LOGIN
// ═══════════════════════════════════════════
const PINLogin = ({ onLogin }) => {
  const [pin, setPin] = useState("");
  const handleDigit = (d) => {
    if (pin.length < 4) {
      const next = pin + d;
      setPin(next);
      if (next.length === 4) setTimeout(() => onLogin(), 400);
    }
  };
  return (
    <div style={{ minHeight: "100vh", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", background: `linear-gradient(180deg, ${T.wineDark} 0%, ${T.wine} 50%, ${T.wineLight} 100%)`, fontFamily: T.font, padding: 24 }}>
      <div style={{ fontSize: 40, marginBottom: 8, opacity: 0.35 }}>☦</div>
      <h1 style={{ fontFamily: T.fontDisplay, fontSize: 32, color: T.goldLight, fontWeight: 600, letterSpacing: 2, marginBottom: 4 }}>SCRIPTORIUM</h1>
      <p style={{ color: "rgba(255,255,255,0.45)", fontSize: 13, letterSpacing: 4, marginBottom: 40, textTransform: "uppercase" }}>Point of Sale</p>
      <div style={{ display: "flex", gap: 16, marginBottom: 36 }}>
        {[0,1,2,3].map(i => (
          <div key={i} style={{ width: 18, height: 18, borderRadius: "50%", border: `2px solid ${T.goldLight}`, background: i < pin.length ? T.goldLight : "transparent", transition: "all 0.2s ease" }} />
        ))}
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 12, maxWidth: 280, width: "100%" }}>
        {[1,2,3,4,5,6,7,8,9,"",0,"⌫"].map((d, i) => (
          d === "" ? <div key={i} /> :
          <button key={i} onClick={() => d === "⌫" ? setPin(pin.slice(0,-1)) : handleDigit(String(d))} style={{ width: "100%", aspectRatio: "1.3", borderRadius: T.radius, border: "none", fontSize: d === "⌫" ? 22 : 28, fontFamily: T.font, fontWeight: 500, background: "rgba(255,255,255,0.08)", color: T.white, cursor: "pointer", display: "flex", alignItems: "center", justifyContent: "center" }}>{d}</button>
        ))}
      </div>
      <p style={{ color: "rgba(255,255,255,0.3)", fontSize: 13, marginTop: 40, cursor: "pointer" }}>Forgot PIN? · Admin Login</p>
    </div>
  );
};

// ═══════════════════════════════════════════
// SCREEN 2: MAIN POS INTERFACE
// ═══════════════════════════════════════════
const POSMain = ({ onCheckout }) => {
  const [cart, setCart] = useState([...sampleCart]);
  const [mode, setMode] = useState("scan");
  const [discount, setDiscount] = useState(null);

  const subtotal = cart.reduce((s, i) => s + i.price * i.qty, 0);
  const discountAmt = discount ? subtotal * discount.pct : 0;
  const total = subtotal - discountAmt;

  const addQuickItem = (item) => {
    setCart(prev => {
      const existing = prev.find(c => c.name === item.name);
      if (existing) return prev.map(c => c.name === item.name ? { ...c, qty: c.qty + 1 } : c);
      return [...prev, { id: Date.now(), name: item.name, price: item.price, qty: 1 }];
    });
  };
  const updateQty = (id, delta) => setCart(prev => prev.map(c => c.id !== id ? c : { ...c, qty: Math.max(0, c.qty + delta) }).filter(c => c.qty > 0));

  return (
    <div style={{ minHeight: "100vh", background: T.parchment, fontFamily: T.font, display: "flex", flexDirection: "column" }}>
      <div style={{ background: T.wine, color: T.white, padding: "12px 16px", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <span style={{ fontFamily: T.fontDisplay, fontSize: 18, fontWeight: 600, color: T.goldLight }}>☦ Scriptorium</span>
        <div style={{ display: "flex", alignItems: "center", gap: 12, fontSize: 13 }}>
          <Badge color={T.goldLight} bg="rgba(184,144,58,0.2)">Shift: AM</Badge>
          <span style={{ opacity: 0.7 }}>Mary V.</span>
        </div>
      </div>

      <div style={{ display: "flex", background: T.white, borderBottom: `1px solid ${T.parchmentDark}` }}>
        {[{ key: "scan", label: "Scan Item", icon: "scan" }, { key: "grid", label: "Quick Items", icon: "grid" }].map(m => (
          <button key={m.key} onClick={() => setMode(m.key)} style={{ flex: 1, padding: "14px 0", border: "none", fontFamily: T.font, fontSize: 15, fontWeight: 600, cursor: "pointer", display: "flex", alignItems: "center", justifyContent: "center", gap: 8, background: mode === m.key ? T.goldPale : "transparent", color: mode === m.key ? T.wine : T.warmGray, borderBottom: mode === m.key ? `3px solid ${T.gold}` : "3px solid transparent" }}>
            <Icon name={m.icon} size={18} color={mode === m.key ? T.wine : T.warmGray} />{m.label}
          </button>
        ))}
      </div>

      <div style={{ flex: 1, overflow: "auto" }}>
        {mode === "scan" ? (
          <div style={{ padding: 16 }}>
            <div style={{ background: "#1a1a1a", borderRadius: T.radiusLg, height: 200, display: "flex", alignItems: "center", justifyContent: "center", position: "relative", overflow: "hidden" }}>
              <div style={{ position: "absolute", inset: 40, border: `2px solid rgba(184,144,58,0.5)`, borderRadius: 12 }} />
              <div style={{ position: "absolute", left: 40, right: 40, top: "50%", height: 2, background: T.gold, opacity: 0.6, animation: "scanline 2s ease-in-out infinite" }} />
              <style>{`@keyframes scanline { 0%,100% { transform: translateY(-60px); } 50% { transform: translateY(60px); } }`}</style>
              <div style={{ position: "absolute", bottom: 16, left: 0, right: 0, textAlign: "center", color: "rgba(255,255,255,0.5)", fontSize: 13, fontWeight: 500 }}>Point camera at barcode</div>
            </div>
            <p style={{ textAlign: "center", color: T.warmGray, fontSize: 13, marginTop: 12 }}>Scan any ISBN, EAN-13, or custom barcode</p>
          </div>
        ) : (
          <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: 10, padding: 16 }}>
            {quickItems.map((item, i) => (
              <button key={i} onClick={() => addQuickItem(item)} style={{ background: item.color, border: "none", borderRadius: T.radius, padding: "16px 12px", display: "flex", flexDirection: "column", alignItems: "center", gap: 6, cursor: "pointer", fontFamily: T.font }}>
                <span style={{ fontSize: 32 }}>{item.emoji}</span>
                <span style={{ fontSize: 14, fontWeight: 600, color: T.ink }}>{item.name}</span>
                <span style={{ fontSize: 16, fontWeight: 700, color: T.wine, background: "rgba(255,255,255,0.7)", padding: "2px 12px", borderRadius: 20 }}>${item.price.toFixed(2)}</span>
              </button>
            ))}
          </div>
        )}
      </div>

      <div style={{ background: T.white, borderTop: `1px solid ${T.parchmentDark}`, boxShadow: "0 -4px 16px rgba(0,0,0,0.05)" }}>
        <div style={{ maxHeight: 180, overflow: "auto", padding: "8px 16px" }}>
          {cart.length === 0 ? (
            <p style={{ textAlign: "center", color: T.warmGray, padding: 16, fontSize: 14 }}>Cart empty — scan an item or use Quick Items</p>
          ) : cart.map(item => (
            <div key={item.id} style={{ display: "flex", alignItems: "center", padding: "8px 0", borderBottom: `1px solid ${T.parchmentDark}`, gap: 8 }}>
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ fontSize: 14, fontWeight: 600, color: T.ink, whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>{item.name}</div>
                <div style={{ fontSize: 12, color: T.warmGray }}>${item.price.toFixed(2)} ea.</div>
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
                <button onClick={() => updateQty(item.id, -1)} style={{ width: 32, height: 32, borderRadius: 8, border: `1px solid ${T.parchmentDark}`, background: T.parchment, cursor: "pointer", display: "flex", alignItems: "center", justifyContent: "center" }}><Icon name="minus" size={14} color={T.inkLight} /></button>
                <span style={{ minWidth: 28, textAlign: "center", fontWeight: 700, fontSize: 16, color: T.ink }}>{item.qty}</span>
                <button onClick={() => updateQty(item.id, 1)} style={{ width: 32, height: 32, borderRadius: 8, border: `1px solid ${T.parchmentDark}`, background: T.parchment, cursor: "pointer", display: "flex", alignItems: "center", justifyContent: "center" }}><Icon name="plus" size={14} color={T.inkLight} /></button>
              </div>
              <span style={{ fontWeight: 700, fontSize: 15, color: T.ink, minWidth: 60, textAlign: "right" }}>${(item.price * item.qty).toFixed(2)}</span>
            </div>
          ))}
        </div>
        {cart.length > 0 && (
          <div style={{ padding: "8px 16px", display: "flex", gap: 8, flexWrap: "wrap" }}>
            {[{ label: "10% Clergy", pct: 0.10, id: "clergy" }, { label: "15% Volunteer", pct: 0.15, id: "vol" }, { label: "20% Bulk", pct: 0.20, id: "bulk" }].map(d => (
              <button key={d.id} onClick={() => setDiscount(discount?.id === d.id ? null : d)} style={{ padding: "6px 14px", borderRadius: 20, fontSize: 13, fontWeight: 600, fontFamily: T.font, cursor: "pointer", border: discount?.id === d.id ? `2px solid ${T.gold}` : `1.5px solid ${T.parchmentDark}`, background: discount?.id === d.id ? T.goldPale : T.white, color: discount?.id === d.id ? T.wine : T.warmGray }}>{d.label}</button>
            ))}
          </div>
        )}
        {cart.length > 0 && (
          <div style={{ padding: "0 16px 16px" }}>
            {discount && <div style={{ display: "flex", justifyContent: "space-between", padding: "4px 0", fontSize: 14, color: T.success }}><span>{discount.label} discount</span><span>-${discountAmt.toFixed(2)}</span></div>}
            <button onClick={() => onCheckout(total, cart)} style={{ width: "100%", padding: "18px 24px", borderRadius: T.radius, border: "none", cursor: "pointer", fontFamily: T.font, background: T.wine, color: T.white, display: "flex", alignItems: "center", justifyContent: "center", gap: 12, fontSize: 20, fontWeight: 700, boxShadow: `0 4px 12px rgba(107,39,55,0.25)` }}>
              <Icon name="cart" size={22} color={T.white} />CHECKOUT · ${total.toFixed(2)}
            </button>
          </div>
        )}
      </div>
    </div>
  );
};

// ═══════════════════════════════════════════
// SCREEN 3: PAYMENT SELECTION
// ═══════════════════════════════════════════
const PaymentScreen = ({ total = 35.95, onComplete, onBack }) => {
  const [method, setMethod] = useState(null);
  const [cashAmount, setCashAmount] = useState(null);
  const [customCash, setCustomCash] = useState("");
  const [tabName, setTabName] = useState("");
  const [donated, setDonated] = useState(false);
  const changeDue = cashAmount ? cashAmount - total : 0;

  if (method === "cash" && cashAmount) {
    return (
      <div style={{ minHeight: "100vh", background: T.parchment, fontFamily: T.font, display: "flex", flexDirection: "column" }}>
        <div style={{ background: T.wine, color: T.white, padding: "12px 16px", display: "flex", alignItems: "center", gap: 12 }}>
          <button onClick={() => setCashAmount(null)} style={{ background: "none", border: "none", cursor: "pointer" }}><Icon name="back" size={24} color={T.white} /></button>
          <span style={{ fontWeight: 600, fontSize: 16 }}>Cash Payment</span>
        </div>
        <div style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", padding: 24, gap: 16 }}>
          <div style={{ fontSize: 13, color: T.warmGray, textTransform: "uppercase", letterSpacing: 2 }}>{donated ? "Donation Received" : "Change Due"}</div>
          <div style={{ fontSize: donated ? 48 : 72, fontWeight: 800, color: donated ? T.success : T.ink, fontFamily: T.font, lineHeight: 1 }}>{donated ? "$0.00" : `$${changeDue.toFixed(2)}`}</div>
          {donated && <div style={{ background: T.successLight, borderRadius: T.radius, padding: "12px 24px", display: "flex", alignItems: "center", gap: 8, color: T.success, fontWeight: 600, border: `1px solid rgba(90,125,94,0.15)` }}><Icon name="heart" size={18} color={T.success} /> ${changeDue.toFixed(2)} donated to the church</div>}
          <div style={{ background: T.white, borderRadius: T.radius, padding: 20, width: "100%", maxWidth: 340, border: `1px solid ${T.parchmentDark}` }}>
            <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8, fontSize: 14 }}><span style={{ color: T.warmGray }}>Order Total</span><span style={{ fontWeight: 700 }}>${total.toFixed(2)}</span></div>
            <div style={{ display: "flex", justifyContent: "space-between", marginBottom: 8, fontSize: 14 }}><span style={{ color: T.warmGray }}>Amount Tendered</span><span style={{ fontWeight: 700 }}>${cashAmount.toFixed(2)}</span></div>
            {donated && <div style={{ display: "flex", justifyContent: "space-between", fontSize: 14, color: T.success }}><span>Donation</span><span style={{ fontWeight: 700 }}>+${changeDue.toFixed(2)}</span></div>}
          </div>
          {!donated && changeDue > 0 && (
            <button onClick={() => setDonated(true)} style={{ width: "100%", maxWidth: 340, padding: "16px 24px", borderRadius: T.radius, border: `1.5px dashed ${T.warmGrayLight}`, background: T.goldPale, cursor: "pointer", fontFamily: T.font, fontSize: 16, fontWeight: 600, color: T.wine, display: "flex", alignItems: "center", justifyContent: "center", gap: 8 }}>
              <Icon name="heart" size={18} color={T.wine} />Round Up / Donate ${changeDue.toFixed(2)}
            </button>
          )}
          <button onClick={onComplete} style={{ width: "100%", maxWidth: 340, padding: "18px 24px", borderRadius: T.radius, border: "none", cursor: "pointer", fontFamily: T.font, background: T.success, color: T.white, fontSize: 18, fontWeight: 700, boxShadow: `0 4px 12px rgba(90,125,94,0.3)`, marginTop: 8 }}>Complete Sale</button>
        </div>
      </div>
    );
  }

  return (
    <div style={{ minHeight: "100vh", background: T.parchment, fontFamily: T.font, display: "flex", flexDirection: "column" }}>
      <div style={{ background: T.wine, color: T.white, padding: "12px 16px", display: "flex", alignItems: "center", gap: 12 }}>
        <button onClick={onBack} style={{ background: "none", border: "none", cursor: "pointer" }}><Icon name="back" size={24} color={T.white} /></button>
        <span style={{ fontWeight: 600, fontSize: 16 }}>Payment</span>
      </div>
      <div style={{ flex: 1, padding: 24, display: "flex", flexDirection: "column", alignItems: "center" }}>
        <div style={{ textAlign: "center", marginBottom: 32 }}>
          <div style={{ fontSize: 13, color: T.warmGray, textTransform: "uppercase", letterSpacing: 2, marginBottom: 8 }}>Total Due</div>
          <div style={{ fontSize: 56, fontWeight: 800, color: T.ink, lineHeight: 1 }}>${total.toFixed(2)}</div>
        </div>
        {!method && (
          <div style={{ width: "100%", maxWidth: 360, display: "flex", flexDirection: "column", gap: 12 }}>
            {[
              { key: "card", icon: "card", color: T.blue, bg: T.blueLight, title: "Credit / Debit Card", sub: "Opens Square terminal" },
              { key: "cash", icon: "cash", color: T.success, bg: T.successLight, title: "Cash", sub: "Calculates change automatically" },
            ].map(opt => (
              <button key={opt.key} onClick={() => setMethod(opt.key)} style={{ display: "flex", alignItems: "center", gap: 16, padding: "20px 24px", borderRadius: T.radiusLg, border: `1px solid ${T.parchmentDark}`, background: T.white, cursor: "pointer", fontFamily: T.font, boxShadow: T.shadow }}>
                <div style={{ width: 50, height: 50, borderRadius: 12, background: opt.bg, display: "flex", alignItems: "center", justifyContent: "center" }}><Icon name={opt.icon} size={24} color={opt.color} /></div>
                <div style={{ textAlign: "left", flex: 1 }}><div style={{ fontSize: 17, fontWeight: 700, color: T.ink }}>{opt.title}</div><div style={{ fontSize: 13, color: T.warmGray }}>{opt.sub}</div></div>
                <Icon name="chevronRight" size={18} color={T.warmGrayLight} />
              </button>
            ))}
            <button onClick={() => setMethod("tab")} style={{ display: "flex", alignItems: "center", gap: 16, padding: "20px 24px", borderRadius: T.radiusLg, border: `1.5px dashed ${T.warmGrayLight}`, background: T.warningLight, cursor: "pointer", fontFamily: T.font }}>
              <div style={{ width: 50, height: 50, borderRadius: 12, background: "rgba(160,112,64,0.12)", display: "flex", alignItems: "center", justifyContent: "center" }}><Icon name="tab" size={24} color={T.warning} /></div>
              <div style={{ textAlign: "left", flex: 1 }}><div style={{ fontSize: 17, fontWeight: 700, color: T.ink }}>Put on Tab / IOU</div><div style={{ fontSize: 13, color: T.warmGray }}>Customer will pay later</div></div>
              <Icon name="chevronRight" size={18} color={T.warmGrayLight} />
            </button>
          </div>
        )}
        {method === "card" && (
          <div style={{ width: "100%", maxWidth: 360, textAlign: "center", display: "flex", flexDirection: "column", alignItems: "center", gap: 16 }}>
            <div style={{ width: 72, height: 72, borderRadius: "50%", background: T.blueLight, display: "flex", alignItems: "center", justifyContent: "center", animation: "pulse 1.5s ease-in-out infinite" }}><Icon name="card" size={32} color={T.blue} /></div>
            <style>{`@keyframes pulse { 0%,100% { transform: scale(1); } 50% { transform: scale(1.06); } }`}</style>
            <p style={{ fontSize: 18, fontWeight: 600, color: T.ink }}>Opening Square Terminal...</p>
            <p style={{ fontSize: 14, color: T.warmGray }}>Complete the ${total.toFixed(2)} payment in the Square app, then return here.</p>
            <button onClick={onComplete} style={{ width: "100%", padding: "16px", borderRadius: T.radius, border: "none", background: T.success, color: T.white, fontSize: 16, fontWeight: 700, cursor: "pointer", fontFamily: T.font, marginTop: 12 }}>Payment Received — Complete</button>
            <button onClick={() => setMethod(null)} style={{ background: "none", border: "none", color: T.warmGray, cursor: "pointer", fontSize: 14, fontFamily: T.font }}>Cancel — Go Back</button>
          </div>
        )}
        {method === "cash" && !cashAmount && (
          <div style={{ width: "100%", maxWidth: 360 }}>
            <p style={{ fontSize: 14, fontWeight: 600, color: T.inkLight, marginBottom: 12 }}>Amount Tendered:</p>
            <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: 10 }}>
              {[{ label: `$${total.toFixed(2)}`, sublabel: "Exact", amount: total }, { label: "$20.00", amount: 20 }, { label: "$50.00", amount: 50 }, { label: "$100.00", amount: 100 }].filter(b => b.amount >= total).map((b, i) => (
                <button key={i} onClick={() => setCashAmount(b.amount)} style={{ padding: "20px 16px", borderRadius: T.radius, border: b.sublabel ? `1.5px solid ${T.gold}` : `1px solid ${T.parchmentDark}`, background: b.sublabel ? T.goldPale : T.white, cursor: "pointer", fontFamily: T.font, display: "flex", flexDirection: "column", alignItems: "center", gap: 4 }}>
                  <span style={{ fontSize: 24, fontWeight: 800, color: T.ink }}>{b.label}</span>
                  {b.sublabel && <span style={{ fontSize: 12, color: T.gold, fontWeight: 600 }}>{b.sublabel}</span>}
                </button>
              ))}
            </div>
            <div style={{ marginTop: 16, display: "flex", gap: 8 }}>
              <input type="number" placeholder="Custom amount" value={customCash} onChange={e => setCustomCash(e.target.value)} style={{ flex: 1, padding: "14px 16px", borderRadius: T.radiusSm, border: `1.5px solid ${T.parchmentDark}`, fontSize: 18, fontFamily: T.font, outline: "none", fontWeight: 600 }} />
              <button onClick={() => customCash && setCashAmount(parseFloat(customCash))} style={{ padding: "14px 24px", borderRadius: T.radiusSm, border: "none", background: T.wine, color: T.white, fontWeight: 700, cursor: "pointer", fontFamily: T.font, fontSize: 16 }}>Go</button>
            </div>
            <button onClick={() => setMethod(null)} style={{ width: "100%", background: "none", border: "none", color: T.warmGray, cursor: "pointer", fontSize: 14, fontFamily: T.font, marginTop: 20, padding: 8 }}>← Back to payment methods</button>
          </div>
        )}
        {method === "tab" && (
          <div style={{ width: "100%", maxWidth: 360 }}>
            <div style={{ background: T.warningLight, borderRadius: T.radius, padding: 16, marginBottom: 20, display: "flex", alignItems: "center", gap: 12, border: `1px solid rgba(160,112,64,0.12)` }}>
              <Icon name="alert" size={20} color={T.warning} />
              <span style={{ fontSize: 14, color: T.warning, fontWeight: 500 }}>This order will be marked as <strong>Unpaid</strong> in the admin dashboard.</span>
            </div>
            <label style={{ fontSize: 14, fontWeight: 600, color: T.inkLight, display: "block", marginBottom: 8 }}>Customer Name *</label>
            <input type="text" placeholder="e.g. John Doe" value={tabName} onChange={e => setTabName(e.target.value)} style={{ width: "100%", padding: "16px", borderRadius: T.radiusSm, border: `1.5px solid ${T.parchmentDark}`, fontSize: 18, fontFamily: T.font, outline: "none", boxSizing: "border-box" }} />
            <button onClick={onComplete} disabled={!tabName.trim()} style={{ width: "100%", padding: "18px", borderRadius: T.radius, border: "none", background: tabName.trim() ? T.warning : T.parchmentDark, color: T.white, fontSize: 18, fontWeight: 700, cursor: tabName.trim() ? "pointer" : "default", fontFamily: T.font, marginTop: 16 }}>Record IOU — ${total.toFixed(2)}</button>
            <button onClick={() => setMethod(null)} style={{ width: "100%", background: "none", border: "none", color: T.warmGray, cursor: "pointer", fontSize: 14, fontFamily: T.font, marginTop: 16, padding: 8 }}>← Back to payment methods</button>
          </div>
        )}
      </div>
    </div>
  );
};

// ═══════════════════════════════════════════
// SCREEN 4: TRANSACTION COMPLETE
// ═══════════════════════════════════════════
const TransactionComplete = ({ total = 35.95, changeDue = 3.50, onNewSale }) => {
  const [email, setEmail] = useState("");
  const [sent, setSent] = useState(false);
  return (
    <div style={{ minHeight: "100vh", fontFamily: T.font, background: `linear-gradient(180deg, ${T.success} 0%, ${T.successAccent} 100%)`, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", padding: 24 }}>
      <div style={{ width: 84, height: 84, borderRadius: "50%", background: "rgba(255,255,255,0.15)", display: "flex", alignItems: "center", justifyContent: "center", marginBottom: 20 }}>
        <div style={{ width: 60, height: 60, borderRadius: "50%", background: T.white, display: "flex", alignItems: "center", justifyContent: "center" }}><Icon name="check" size={32} color={T.success} /></div>
      </div>
      <h1 style={{ fontSize: 30, fontWeight: 800, color: T.white, margin: 0, letterSpacing: 2 }}>SALE COMPLETE</h1>
      {changeDue > 0 && (
        <div style={{ background: "rgba(255,255,255,0.12)", borderRadius: T.radiusLg, padding: "20px 40px", marginTop: 24, textAlign: "center", border: "1px solid rgba(255,255,255,0.15)" }}>
          <div style={{ fontSize: 13, color: "rgba(255,255,255,0.6)", textTransform: "uppercase", letterSpacing: 2, marginBottom: 4 }}>Change Due</div>
          <div style={{ fontSize: 56, fontWeight: 800, color: T.white, lineHeight: 1 }}>${changeDue.toFixed(2)}</div>
        </div>
      )}
      <div style={{ background: "rgba(255,255,255,0.12)", borderRadius: T.radius, padding: 16, marginTop: 20, width: "100%", maxWidth: 320 }}>
        <div style={{ display: "flex", justifyContent: "space-between", color: "rgba(255,255,255,0.6)", fontSize: 14, marginBottom: 4 }}><span>Order Total</span><span style={{ color: T.white, fontWeight: 700 }}>${total.toFixed(2)}</span></div>
        <div style={{ display: "flex", justifyContent: "space-between", color: "rgba(255,255,255,0.6)", fontSize: 14 }}><span>Payment</span><span style={{ color: T.white, fontWeight: 700 }}>Cash</span></div>
      </div>
      <div style={{ width: "100%", maxWidth: 320, marginTop: 24 }}>
        {!sent ? (
          <div style={{ display: "flex", gap: 8 }}>
            <input type="email" placeholder="Email for receipt (optional)" value={email} onChange={e => setEmail(e.target.value)} style={{ flex: 1, padding: "14px 16px", borderRadius: T.radiusSm, border: "none", fontSize: 15, fontFamily: T.font, background: "rgba(255,255,255,0.9)" }} />
            <button onClick={() => email && setSent(true)} style={{ padding: "14px 16px", borderRadius: T.radiusSm, border: "none", background: T.white, cursor: "pointer" }}><Icon name="mail" size={20} color={T.success} /></button>
          </div>
        ) : <div style={{ textAlign: "center", color: "rgba(255,255,255,0.75)", fontSize: 14, fontWeight: 500 }}>Receipt sent to {email}</div>}
      </div>
      <button onClick={onNewSale} style={{ width: "100%", maxWidth: 320, padding: "20px 24px", borderRadius: T.radius, border: "none", cursor: "pointer", fontFamily: T.font, background: T.white, color: T.success, fontSize: 20, fontWeight: 800, letterSpacing: 1, boxShadow: "0 4px 16px rgba(0,0,0,0.15)", marginTop: 28 }}>START NEXT SALE →</button>
    </div>
  );
};

// ═══════════════════════════════════════════
// SCREEN 5: STOREFRONT (paginated)
// ═══════════════════════════════════════════
const Storefront = ({ onProduct }) => {
  const categories = ["All", "Books", "Icons", "Liturgical", "Gifts"];
  const [activeCat, setActiveCat] = useState("All");
  const [page, setPage] = useState(1);
  const perPage = 6;
  const filtered = activeCat === "All" ? sampleProducts : sampleProducts.filter(p => p.cat === activeCat);
  const totalPages = Math.max(1, Math.ceil(filtered.length / perPage));
  const pageItems = filtered.slice((page - 1) * perPage, page * perPage);
  const changeCat = (c) => { setActiveCat(c); setPage(1); };

  return (
    <div style={{ minHeight: "100vh", background: T.parchment, fontFamily: T.font }}>
      <div style={{ background: T.white, borderBottom: `1px solid ${T.parchmentDark}`, padding: "16px 24px" }}>
        <div style={{ maxWidth: 1100, margin: "0 auto", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 10 }}><span style={{ fontSize: 24 }}>☦</span><span style={{ fontFamily: T.fontDisplay, fontSize: 24, fontWeight: 600, color: T.wine }}>Scriptorium</span></div>
          <div style={{ position: "relative" }}><Icon name="cart" size={22} color={T.ink} /><div style={{ position: "absolute", top: -6, right: -8, width: 18, height: 18, borderRadius: "50%", background: T.wine, color: T.white, fontSize: 11, fontWeight: 700, display: "flex", alignItems: "center", justifyContent: "center" }}>2</div></div>
        </div>
      </div>
      <div style={{ maxWidth: 1100, margin: "0 auto", padding: "24px 24px" }}>
        <div style={{ background: `linear-gradient(135deg, ${T.wineDark} 0%, ${T.wine} 100%)`, borderRadius: T.radiusLg, padding: "40px 32px", marginBottom: 28, color: T.white }}>
          <h1 style={{ fontFamily: T.fontDisplay, fontSize: 36, fontWeight: 600, margin: "0 0 8px", color: T.goldLight }}>Feed your soul.</h1>
          <p style={{ fontSize: 16, opacity: 0.7, margin: "0 0 20px", maxWidth: 500 }}>Orthodox books, icons, liturgical supplies & gifts. All proceeds support our parish.</p>
          <div style={{ display: "flex", background: "rgba(255,255,255,0.12)", borderRadius: T.radiusSm, padding: "4px 4px 4px 16px", maxWidth: 460, alignItems: "center", border: "1px solid rgba(255,255,255,0.15)" }}>
            <Icon name="search" size={18} color="rgba(255,255,255,0.5)" />
            <input placeholder="Search by title, author, or ISBN..." style={{ flex: 1, padding: "10px 12px", border: "none", background: "transparent", fontSize: 15, fontFamily: T.font, color: T.white, outline: "none" }} />
            <button style={{ padding: "10px 20px", borderRadius: 6, border: "none", background: T.gold, color: T.white, fontWeight: 700, cursor: "pointer", fontFamily: T.font }}>Search</button>
          </div>
        </div>
        <div style={{ display: "flex", gap: 8, marginBottom: 24, flexWrap: "wrap" }}>
          {categories.map(c => (
            <button key={c} onClick={() => changeCat(c)} style={{ padding: "8px 20px", borderRadius: 24, fontFamily: T.font, fontSize: 14, fontWeight: 600, cursor: "pointer", border: activeCat === c ? `2px solid ${T.wine}` : `1px solid ${T.parchmentDark}`, background: activeCat === c ? T.wine : T.white, color: activeCat === c ? T.white : T.warmGray }}>{c}</button>
          ))}
          <span style={{ marginLeft: "auto", fontSize: 13, color: T.warmGrayLight, alignSelf: "center" }}>{filtered.length} items</span>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(240px, 1fr))", gap: 16 }}>
          {pageItems.map(p => (
            <Card key={p.id} style={{ cursor: "pointer", overflow: "hidden" }} onClick={() => onProduct(p)}>
              <div style={{ height: 160, background: T.parchmentDark, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 56, position: "relative" }}>
                {p.img}
                {p.stock === 0 && <div style={{ position: "absolute", top: 12, right: 12, background: T.danger, color: T.white, fontSize: 11, fontWeight: 700, padding: "3px 10px", borderRadius: 12 }}>Out of Stock</div>}
                {p.stock > 0 && p.stock <= 3 && <div style={{ position: "absolute", top: 12, right: 12, background: T.warning, color: T.white, fontSize: 11, fontWeight: 700, padding: "3px 10px", borderRadius: 12 }}>Only {p.stock} Left</div>}
              </div>
              <div style={{ padding: "14px 16px" }}>
                <div style={{ fontSize: 11, color: T.warmGray, textTransform: "uppercase", letterSpacing: 1 }}>{p.cat}</div>
                <div style={{ fontSize: 15, fontWeight: 700, color: T.ink, margin: "4px 0 2px" }}>{p.title}</div>
                <div style={{ fontSize: 13, color: T.warmGray }}>{p.author}</div>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginTop: 12 }}>
                  <span style={{ fontSize: 18, fontWeight: 800, color: T.wine }}>${p.price.toFixed(2)}</span>
                  {p.stock > 0 && <Btn variant="primary" size="sm">Add to Cart</Btn>}
                </div>
              </div>
            </Card>
          ))}
        </div>
        {totalPages > 1 && <Pagination page={page} totalPages={totalPages} onPageChange={setPage} />}
      </div>
    </div>
  );
};

// ═══════════════════════════════════════════
// SCREEN 6: PRODUCT DETAIL
// ═══════════════════════════════════════════
const ProductDetail = ({ product, onBack }) => {
  const p = product || sampleProducts[0];
  return (
    <div style={{ minHeight: "100vh", background: T.parchment, fontFamily: T.font }}>
      <div style={{ background: T.white, borderBottom: `1px solid ${T.parchmentDark}`, padding: "16px 24px" }}>
        <div style={{ maxWidth: 1100, margin: "0 auto", display: "flex", alignItems: "center", gap: 12 }}>
          <button onClick={onBack} style={{ background: "none", border: "none", cursor: "pointer" }}><Icon name="back" size={22} color={T.ink} /></button>
          <span style={{ fontFamily: T.fontDisplay, fontSize: 20, color: T.wine }}>☦ Scriptorium</span>
        </div>
      </div>
      <div style={{ maxWidth: 900, margin: "0 auto", padding: 24 }}>
        <div style={{ display: "flex", gap: 32, flexWrap: "wrap" }}>
          <div style={{ width: 320, minWidth: 240, height: 400, background: T.parchmentDark, borderRadius: T.radiusLg, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 100, flexShrink: 0 }}>{p.img}</div>
          <div style={{ flex: 1, minWidth: 280 }}>
            <Badge color={T.warmGray}>{p.cat}</Badge>
            <h1 style={{ fontFamily: T.fontDisplay, fontSize: 32, fontWeight: 600, color: T.ink, margin: "12px 0 4px" }}>{p.title}</h1>
            <p style={{ fontSize: 16, color: T.warmGray, margin: "0 0 16px" }}>by {p.author}</p>
            <div style={{ display: "flex", alignItems: "baseline", gap: 12, marginBottom: 20 }}>
              <span style={{ fontSize: 32, fontWeight: 800, color: T.wine }}>${p.price.toFixed(2)}</span>
              {p.stock > 0 ? <Badge color={T.success} bg={T.successLight}>{p.stock > 5 ? "In Stock" : `Only ${p.stock} Left`}</Badge> : <Badge color={T.danger} bg={T.dangerLight}>Out of Stock</Badge>}
            </div>
            <Card style={{ padding: 20, marginBottom: 20 }}>
              <h3 style={{ fontSize: 14, color: T.inkLight, margin: "0 0 8px", fontWeight: 600 }}>Description</h3>
              <p style={{ fontSize: 14, color: T.warmGray, lineHeight: 1.7, margin: 0 }}>A comprehensive resource for deepening your understanding of the Orthodox Christian faith. This edition includes detailed study notes, commentary from early Church Fathers, and full-color iconography throughout.</p>
            </Card>
            <Card style={{ padding: 20, marginBottom: 24 }}>
              <h3 style={{ fontSize: 14, color: T.inkLight, margin: "0 0 12px", fontWeight: 600 }}>Details</h3>
              {[["Publisher", "Thomas Nelson"], ["ISBN", "978-0-7180-0359-3"], ["Binding", "Hardcover"], ["Pages", "1,792"]].map(([k, v]) => (
                <div key={k} style={{ display: "flex", justifyContent: "space-between", padding: "6px 0", borderBottom: `1px solid ${T.parchmentDark}`, fontSize: 14 }}>
                  <span style={{ color: T.warmGray }}>{k}</span><span style={{ fontWeight: 600, color: T.ink }}>{v}</span>
                </div>
              ))}
            </Card>
            {p.stock > 0 && <button style={{ width: "100%", padding: "16px", borderRadius: T.radius, border: "none", background: T.wine, color: T.white, fontSize: 18, fontWeight: 700, cursor: "pointer", fontFamily: T.font, boxShadow: `0 4px 12px rgba(107,39,55,0.25)` }}>Add to Cart — ${p.price.toFixed(2)}</button>}
          </div>
        </div>
      </div>
    </div>
  );
};

// ═══════════════════════════════════════════
// SCREEN 7: ONLINE CHECKOUT
// ═══════════════════════════════════════════
const OnlineCheckout = ({ onBack }) => {
  const cartItems = [{ name: "The Orthodox Study Bible", qty: 1, price: 34.95 }, { name: "Beeswax Candles (12 pk)", qty: 2, price: 8.99 }];
  const subtotal = cartItems.reduce((s, i) => s + i.price * i.qty, 0);
  const tax = subtotal * 0.07; const shipping = 5.99;
  return (
    <div style={{ minHeight: "100vh", background: T.parchment, fontFamily: T.font }}>
      <div style={{ background: T.white, borderBottom: `1px solid ${T.parchmentDark}`, padding: "16px 24px" }}>
        <div style={{ maxWidth: 800, margin: "0 auto", display: "flex", alignItems: "center", gap: 12 }}>
          <button onClick={onBack} style={{ background: "none", border: "none", cursor: "pointer" }}><Icon name="back" size={22} color={T.ink} /></button>
          <span style={{ fontFamily: T.fontDisplay, fontSize: 20, color: T.wine }}>☦ Scriptorium — Checkout</span>
        </div>
      </div>
      <div style={{ maxWidth: 800, margin: "0 auto", padding: 24, display: "flex", gap: 24, flexWrap: "wrap" }}>
        <div style={{ flex: "1 1 360px" }}>
          <Card style={{ padding: 24, marginBottom: 16 }}>
            <h2 style={{ fontSize: 18, fontWeight: 700, color: T.ink, margin: "0 0 16px" }}>Contact & Shipping</h2>
            {[{ label: "Full Name", placeholder: "Maria Stavros" }, { label: "Email", placeholder: "maria@example.com" }, { label: "Address", placeholder: "123 Main St, Springfield, IL" }].map(f => (
              <div key={f.label} style={{ marginBottom: 16 }}>
                <label style={{ fontSize: 13, fontWeight: 600, color: T.inkLight, display: "block", marginBottom: 6 }}>{f.label}</label>
                <input placeholder={f.placeholder} style={{ width: "100%", padding: "12px 14px", borderRadius: T.radiusSm, border: `1px solid ${T.parchmentDark}`, fontSize: 15, fontFamily: T.font, outline: "none", boxSizing: "border-box" }} />
              </div>
            ))}
          </Card>
          <Card style={{ padding: 24 }}>
            <h2 style={{ fontSize: 18, fontWeight: 700, color: T.ink, margin: "0 0 16px" }}>Payment</h2>
            <div style={{ background: T.parchment, borderRadius: T.radiusSm, padding: 20, border: `1px solid ${T.parchmentDark}`, textAlign: "center" }}>
              <Icon name="card" size={28} color={T.warmGrayLight} />
              <p style={{ fontSize: 14, color: T.warmGray, margin: "8px 0 0" }}>Secure payment via Stripe</p>
              <div style={{ marginTop: 16, height: 44, background: T.white, borderRadius: T.radiusSm, border: `1px solid ${T.parchmentDark}`, display: "flex", alignItems: "center", padding: "0 14px", color: T.warmGray, fontSize: 14 }}>4242 4242 4242 4242 &nbsp;&nbsp; 12/28 &nbsp;&nbsp; 123</div>
            </div>
          </Card>
        </div>
        <div style={{ flex: "0 1 300px" }}>
          <Card style={{ padding: 24, position: "sticky", top: 24 }}>
            <h2 style={{ fontSize: 18, fontWeight: 700, color: T.ink, margin: "0 0 16px" }}>Order Summary</h2>
            {cartItems.map((item, i) => (
              <div key={i} style={{ display: "flex", justifyContent: "space-between", padding: "8px 0", borderBottom: `1px solid ${T.parchmentDark}`, fontSize: 14 }}>
                <span style={{ color: T.ink }}>{item.name} × {item.qty}</span><span style={{ fontWeight: 700 }}>${(item.price * item.qty).toFixed(2)}</span>
              </div>
            ))}
            <div style={{ marginTop: 12, fontSize: 14 }}>
              {[["Subtotal", subtotal], ["Shipping", shipping], ["Tax (7%)", tax]].map(([l, v]) => (
                <div key={l} style={{ display: "flex", justifyContent: "space-between", padding: "4px 0", color: T.inkLight }}><span>{l}</span><span>${v.toFixed(2)}</span></div>
              ))}
              <div style={{ display: "flex", justifyContent: "space-between", padding: "12px 0 0", borderTop: `2px solid ${T.parchmentDark}`, marginTop: 8, fontSize: 20, fontWeight: 800, color: T.ink }}><span>Total</span><span>${(subtotal + shipping + tax).toFixed(2)}</span></div>
            </div>
            <button style={{ width: "100%", padding: "16px", borderRadius: T.radius, border: "none", background: T.wine, color: T.white, fontSize: 16, fontWeight: 700, cursor: "pointer", fontFamily: T.font, marginTop: 20, boxShadow: `0 4px 12px rgba(107,39,55,0.25)` }}>Place Order — ${(subtotal + shipping + tax).toFixed(2)}</button>
          </Card>
        </div>
      </div>
    </div>
  );
};

// ═══════════════════════════════════════════
// SCREEN 8: ADMIN DASHBOARD
// ═══════════════════════════════════════════
const AdminDashboard = () => {
  const metrics = [
    { label: "Today's Sales", value: "$347.85", change: "+12% vs. last Sunday", icon: "trend", color: T.success },
    { label: "POS Revenue", value: "$247.40", sub: "14 transactions", icon: "cart", color: T.wine },
    { label: "Online Revenue", value: "$100.45", sub: "3 orders", icon: "package", color: T.blue },
    { label: "Open IOUs", value: "$56.95", sub: "2 unpaid", icon: "alert", color: T.warning },
  ];
  const lowStock = [
    { name: "Theotokos Icon (small)", stock: 2 }, { name: "Beginning to Pray", stock: 0 }, { name: "Icon of Christ Pantocrator", stock: 3 },
  ];
  return (
    <div style={{ minHeight: "100vh", background: T.parchment, fontFamily: T.font }}>
      <div style={{ background: T.white, borderBottom: `1px solid ${T.parchmentDark}`, padding: "16px 32px" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}><span style={{ fontSize: 22 }}>☦</span><span style={{ fontFamily: T.fontDisplay, fontSize: 22, fontWeight: 600, color: T.wine }}>Scriptorium</span><Badge color={T.warmGray}>Admin</Badge></div>
          <div style={{ display: "flex", gap: 12 }}><Btn variant="gold" size="sm"><Icon name="plus" size={16} color={T.white} /> Add Product</Btn><Btn variant="ghost" size="sm"><Icon name="orders" size={16} /> Orders</Btn></div>
        </div>
      </div>
      <div style={{ padding: "24px 32px", maxWidth: 1200, margin: "0 auto" }}>
        <h2 style={{ fontFamily: T.fontDisplay, fontSize: 28, color: T.ink, margin: "0 0 20px" }}>Good morning, Father Michael</h2>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(240px, 1fr))", gap: 16, marginBottom: 24 }}>
          {metrics.map((m, i) => (
            <Card key={i} style={{ padding: 20 }}>
              <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 8 }}>
                <span style={{ fontSize: 13, color: T.warmGray, fontWeight: 500 }}>{m.label}</span>
                <div style={{ width: 36, height: 36, borderRadius: 10, background: `${m.color}10`, display: "flex", alignItems: "center", justifyContent: "center" }}><Icon name={m.icon} size={18} color={m.color} /></div>
              </div>
              <div style={{ fontSize: 28, fontWeight: 800, color: T.ink }}>{m.value}</div>
              {m.change && <span style={{ fontSize: 13, color: T.success, fontWeight: 500 }}>{m.change}</span>}
              {m.sub && <span style={{ fontSize: 13, color: T.warmGray }}>{m.sub}</span>}
            </Card>
          ))}
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
          <Card style={{ padding: 24 }}>
            <h3 style={{ fontSize: 16, fontWeight: 700, color: T.ink, margin: "0 0 16px", display: "flex", alignItems: "center", gap: 8 }}><Icon name="alert" size={18} color={T.warning} /> Requires Attention</h3>
            <div style={{ fontSize: 12, color: T.warmGray, fontWeight: 600, marginBottom: 8, textTransform: "uppercase", letterSpacing: 1 }}>Unpaid IOUs</div>
            {[{ name: "John Doe", amount: "$34.95", date: "Mar 2" }, { name: "Anna T.", amount: "$22.00", date: "Feb 28" }].map((iou, i) => (
              <div key={i} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "10px 14px", background: T.warningLight, borderRadius: T.radiusSm, marginBottom: 6, border: `1px solid rgba(160,112,64,0.08)` }}>
                <div><div style={{ fontWeight: 600, fontSize: 14, color: T.ink }}>{iou.name}</div><div style={{ fontSize: 12, color: T.warmGray }}>{iou.date}</div></div>
                <div style={{ display: "flex", alignItems: "center", gap: 8 }}><span style={{ fontWeight: 700, color: T.warning, fontSize: 14 }}>{iou.amount}</span><Btn variant="success" size="sm">Mark Paid</Btn></div>
              </div>
            ))}
            <div style={{ fontSize: 12, color: T.warmGray, fontWeight: 600, marginBottom: 8, marginTop: 16, textTransform: "uppercase", letterSpacing: 1 }}>Low Stock</div>
            {lowStock.map((item, i) => (
              <div key={i} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "8px 0", borderBottom: `1px solid ${T.parchmentDark}`, fontSize: 14 }}>
                <span style={{ color: T.ink }}>{item.name}</span>
                <Badge color={item.stock === 0 ? T.danger : T.warning} bg={item.stock === 0 ? T.dangerLight : T.warningLight}>{item.stock === 0 ? "Out of stock" : `${item.stock} left`}</Badge>
              </div>
            ))}
          </Card>
          <Card style={{ padding: 24 }}>
            <h3 style={{ fontSize: 16, fontWeight: 700, color: T.ink, margin: "0 0 16px", display: "flex", alignItems: "center", gap: 8 }}><Icon name="clock" size={18} color={T.blue} /> Recent Orders</h3>
            {sampleOrders.slice(0, 5).map((o, i) => (
              <div key={i} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "10px 0", borderBottom: `1px solid ${T.parchmentDark}` }}>
                <div><div style={{ fontSize: 14, fontWeight: 600, color: T.ink }}>{o.id}</div><div style={{ fontSize: 12, color: T.warmGray }}>{o.date} · {o.customer}</div></div>
                <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <Badge color={o.channel === "POS" ? T.wineMuted : T.blue}>{o.channel}</Badge>
                  <Badge color={o.status === "Paid" ? T.success : o.status === "IOU" ? T.warning : T.danger} bg={o.status === "Paid" ? T.successLight : o.status === "IOU" ? T.warningLight : T.dangerLight}>{o.status}</Badge>
                  <span style={{ fontWeight: 700, fontSize: 14, color: T.ink, minWidth: 50, textAlign: "right" }}>${o.total.toFixed(2)}</span>
                </div>
              </div>
            ))}
          </Card>
        </div>
      </div>
    </div>
  );
};

// ═══════════════════════════════════════════
// SCREEN 9: ADD/EDIT PRODUCT
// ═══════════════════════════════════════════
const AddProduct = ({ onBack }) => {
  const [fetched, setFetched] = useState(false);
  return (
    <div style={{ minHeight: "100vh", background: T.parchment, fontFamily: T.font }}>
      <div style={{ background: T.white, borderBottom: `1px solid ${T.parchmentDark}`, padding: "16px 32px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <button onClick={onBack} style={{ background: "none", border: "none", cursor: "pointer" }}><Icon name="back" size={22} color={T.ink} /></button>
          <span style={{ fontFamily: T.fontDisplay, fontSize: 20, color: T.wine }}>Add New Product</span>
        </div>
      </div>
      <div style={{ maxWidth: 820, margin: "0 auto", padding: 24 }}>
        <Card style={{ padding: 24, marginBottom: 20, background: T.goldPale, border: `1px solid ${T.filledBorder}` }}>
          <h3 style={{ fontSize: 16, fontWeight: 700, color: T.ink, margin: "0 0 4px" }}>ISBN Auto-Lookup</h3>
          <p style={{ fontSize: 13, color: T.warmGray, margin: "0 0 16px" }}>Scan or type an ISBN — we'll fetch metadata automatically.</p>
          <div style={{ display: "flex", gap: 8 }}>
            <input defaultValue={fetched ? "978-0-7180-0359-3" : ""} placeholder="Enter ISBN-13 or ISBN-10..." style={{ flex: 1, padding: "14px 16px", borderRadius: T.radiusSm, border: `1.5px solid ${T.filledBorder}`, fontSize: 16, fontFamily: T.font, outline: "none", fontWeight: 500, background: T.white }} />
            <button onClick={() => setFetched(true)} style={{ padding: "14px 24px", borderRadius: T.radiusSm, border: "none", background: T.wine, color: T.white, fontWeight: 700, cursor: "pointer", fontFamily: T.font, fontSize: 15, display: "flex", alignItems: "center", gap: 8 }}><Icon name="search" size={18} color={T.white} /> Fetch</button>
            <button style={{ padding: "14px", borderRadius: T.radiusSm, border: `1px solid ${T.filledBorder}`, background: T.white, cursor: "pointer" }}><Icon name="scan" size={20} color={T.wine} /></button>
          </div>
          {fetched && <div style={{ marginTop: 12, padding: "8px 14px", background: T.filled, borderRadius: T.radiusSm, color: T.success, fontWeight: 600, fontSize: 13, display: "flex", alignItems: "center", gap: 8, border: `1px solid ${T.filledBorder}` }}><Icon name="check" size={16} color={T.success} /> Found — metadata auto-filled below.</div>}
        </Card>
        <div style={{ display: "grid", gridTemplateColumns: "200px 1fr", gap: 24 }}>
          <Card style={{ height: 280, display: "flex", alignItems: "center", justifyContent: "center", flexDirection: "column", gap: 8, background: fetched ? T.parchmentDark : T.white }}>
            {fetched ? <span style={{ fontSize: 80 }}>📖</span> : <><Icon name="plus" size={32} color={T.warmGrayLight} /><span style={{ fontSize: 13, color: T.warmGray }}>Upload Cover</span></>}
          </Card>
          <div>
            {[{ label: "Title", value: fetched ? "The Orthodox Study Bible" : "" }, { label: "Author", value: fetched ? "Thomas Nelson" : "" }, { label: "Publisher", value: fetched ? "Thomas Nelson" : "" }, { label: "Description", value: fetched ? "A comprehensive Orthodox study Bible with notes from the early Church Fathers..." : "", multi: true }].map(f => (
              <div key={f.label} style={{ marginBottom: 14 }}>
                <label style={{ fontSize: 13, fontWeight: 600, color: T.inkLight, display: "block", marginBottom: 4 }}>{f.label}</label>
                {f.multi ? (
                  <textarea defaultValue={f.value} rows={3} style={{ width: "100%", padding: "10px 14px", borderRadius: T.radiusSm, border: `1px solid ${fetched && f.value ? T.filledBorder : T.parchmentDark}`, fontSize: 14, fontFamily: T.font, outline: "none", resize: "vertical", boxSizing: "border-box", background: fetched && f.value ? T.filled : T.white }} />
                ) : (
                  <input defaultValue={f.value} style={{ width: "100%", padding: "10px 14px", borderRadius: T.radiusSm, border: `1px solid ${fetched && f.value ? T.filledBorder : T.parchmentDark}`, fontSize: 14, fontFamily: T.font, outline: "none", boxSizing: "border-box", background: fetched && f.value ? T.filled : T.white }} />
                )}
              </div>
            ))}
          </div>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fit, minmax(200px, 1fr))", gap: 16, marginTop: 8 }}>
          {[{ label: "Cost Price", prefix: "$", value: "18.50" }, { label: "Retail Price", prefix: "$", value: "34.95" }, { label: "Initial Stock", value: "12" }, { label: "Reorder Point", value: "3" }].map(f => (
            <div key={f.label}>
              <label style={{ fontSize: 13, fontWeight: 600, color: T.inkLight, display: "block", marginBottom: 4 }}>{f.label}</label>
              <div style={{ position: "relative" }}>
                {f.prefix && <span style={{ position: "absolute", left: 14, top: "50%", transform: "translateY(-50%)", color: T.warmGray, fontWeight: 600 }}>{f.prefix}</span>}
                <input defaultValue={f.value} style={{ width: "100%", padding: "10px 14px", paddingLeft: f.prefix ? 28 : 14, borderRadius: T.radiusSm, border: `1px solid ${T.parchmentDark}`, fontSize: 14, fontFamily: T.font, outline: "none", boxSizing: "border-box" }} />
              </div>
            </div>
          ))}
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginTop: 16 }}>
          <div>
            <label style={{ fontSize: 13, fontWeight: 600, color: T.inkLight, display: "block", marginBottom: 4 }}>Category</label>
            <select style={{ width: "100%", padding: "10px 14px", borderRadius: T.radiusSm, border: `1px solid ${T.parchmentDark}`, fontSize: 14, fontFamily: T.font, outline: "none", background: T.white }}><option>Books</option><option>Icons</option><option>Liturgical</option><option>Gifts</option></select>
          </div>
          <div>
            <label style={{ fontSize: 13, fontWeight: 600, color: T.inkLight, display: "block", marginBottom: 4 }}>Vendor (Consignment)</label>
            <select style={{ width: "100%", padding: "10px 14px", borderRadius: T.radiusSm, border: `1px solid ${T.parchmentDark}`, fontSize: 14, fontFamily: T.font, outline: "none", background: T.white }}><option>— None (church-owned) —</option><option>Holy Trinity Monastery</option><option>St. Herman Press</option></select>
          </div>
        </div>
        <div style={{ display: "flex", gap: 12, marginTop: 24, justifyContent: "flex-end" }}><Btn variant="ghost" onClick={onBack}>Cancel</Btn><Btn variant="primary" size="lg">Save Product</Btn></div>
      </div>
    </div>
  );
};

// ═══════════════════════════════════════════
// SCREEN 10: ORDER MANAGEMENT (paginated)
// ═══════════════════════════════════════════
const OrderManagement = ({ onBack }) => {
  const [filter, setFilter] = useState("All");
  const [page, setPage] = useState(1);
  const perPage = 5;
  const filtered = filter === "All" ? sampleOrders : filter === "IOU" ? sampleOrders.filter(o => o.status === "IOU") : sampleOrders.filter(o => o.channel === filter);
  const totalPages = Math.max(1, Math.ceil(filtered.length / perPage));
  const pageItems = filtered.slice((page - 1) * perPage, page * perPage);
  const changeFilter = (f) => { setFilter(f); setPage(1); };

  return (
    <div style={{ minHeight: "100vh", background: T.parchment, fontFamily: T.font }}>
      <div style={{ background: T.white, borderBottom: `1px solid ${T.parchmentDark}`, padding: "16px 32px" }}>
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <button onClick={onBack} style={{ background: "none", border: "none", cursor: "pointer" }}><Icon name="back" size={22} color={T.ink} /></button>
          <span style={{ fontFamily: T.fontDisplay, fontSize: 20, color: T.wine }}>Order Management</span>
        </div>
      </div>
      <div style={{ maxWidth: 1100, margin: "0 auto", padding: 24 }}>
        <div style={{ display: "flex", gap: 8, marginBottom: 20, flexWrap: "wrap", alignItems: "center" }}>
          {["All", "POS", "Online", "IOU"].map(f => (
            <button key={f} onClick={() => changeFilter(f)} style={{ padding: "8px 18px", borderRadius: 20, fontFamily: T.font, fontSize: 14, fontWeight: 600, cursor: "pointer", border: filter === f ? `2px solid ${T.wine}` : `1px solid ${T.parchmentDark}`, background: filter === f ? T.wine : T.white, color: filter === f ? T.white : T.warmGray }}>{f}{f === "IOU" ? ` (${sampleOrders.filter(o=>o.status==="IOU").length})` : ""}</button>
          ))}
          <div style={{ flex: 1 }} />
          <span style={{ fontSize: 13, color: T.warmGray }}>{filtered.length} orders</span>
          <Btn variant="ghost" size="sm"><Icon name="download" size={16} /> Export</Btn>
        </div>
        <Card style={{ overflow: "hidden" }}>
          <div style={{ overflowX: "auto" }}>
            <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 14 }}>
              <thead>
                <tr style={{ background: T.parchment }}>
                  {["Order ID", "Date", "Channel", "Customer", "Status", "Method", "Total", "Actions"].map(h => (
                    <th key={h} style={{ padding: "12px 16px", textAlign: "left", fontWeight: 600, color: T.warmGray, fontSize: 12, textTransform: "uppercase", letterSpacing: 1, borderBottom: `1px solid ${T.parchmentDark}` }}>{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {pageItems.map((o, i) => (
                  <tr key={i} style={{ borderBottom: `1px solid ${T.parchmentDark}` }}>
                    <td style={{ padding: "12px 16px", fontWeight: 700, color: T.ink }}>{o.id}</td>
                    <td style={{ padding: "12px 16px", color: T.warmGray }}>{o.date}</td>
                    <td style={{ padding: "12px 16px" }}><Badge color={o.channel === "POS" ? T.wineMuted : T.blue}>{o.channel}</Badge></td>
                    <td style={{ padding: "12px 16px", color: T.ink }}>{o.customer}</td>
                    <td style={{ padding: "12px 16px" }}><Badge color={o.status === "Paid" ? T.success : o.status === "IOU" ? T.warning : T.danger} bg={o.status === "Paid" ? T.successLight : o.status === "IOU" ? T.warningLight : T.dangerLight}>{o.status}</Badge></td>
                    <td style={{ padding: "12px 16px", color: T.warmGray }}>{o.method}</td>
                    <td style={{ padding: "12px 16px", fontWeight: 700, color: T.ink }}>${o.total.toFixed(2)}</td>
                    <td style={{ padding: "12px 16px" }}>
                      <div style={{ display: "flex", gap: 6 }}>
                        <button title="View" style={{ padding: "4px 8px", borderRadius: 6, border: `1px solid ${T.parchmentDark}`, background: T.white, cursor: "pointer" }}><Icon name="eye" size={14} color={T.inkLight} /></button>
                        <button title="Resend" style={{ padding: "4px 8px", borderRadius: 6, border: `1px solid ${T.parchmentDark}`, background: T.white, cursor: "pointer" }}><Icon name="mail" size={14} color={T.inkLight} /></button>
                        {o.status === "IOU" && <button style={{ padding: "4px 10px", borderRadius: 6, border: "none", background: T.success, color: T.white, cursor: "pointer", fontSize: 12, fontWeight: 600, fontFamily: T.font }}>Mark Paid</button>}
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {totalPages > 1 && <div style={{ borderTop: `1px solid ${T.parchmentDark}` }}><Pagination page={page} totalPages={totalPages} onPageChange={setPage} /></div>}
        </Card>
      </div>
    </div>
  );
};

// ═══════════════════════════════════════════
// MAIN APP
// ═══════════════════════════════════════════
export default function App() {
  const [screen, setScreen] = useState("nav");
  const [posFlow, setPosFlow] = useState("login");
  const [selectedProduct, setSelectedProduct] = useState(null);
  const [storeScreen, setStoreScreen] = useState("home");
  const [adminScreen, setAdminScreen] = useState("dashboard");
  const labels = { pos: "Mobile POS (1-4)", store: "Online Store (5-7)", admin: "Admin (8-10)" };

  if (screen === "nav") {
    return (
      <div style={{ minHeight: "100vh", fontFamily: T.font, background: `linear-gradient(160deg, ${T.parchment} 0%, ${T.goldPale} 100%)`, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", padding: 24 }}>
        <link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;500;600;700&family=DM+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet" />
        <div style={{ textAlign: "center", marginBottom: 40 }}>
          <div style={{ fontSize: 48, marginBottom: 8, opacity: 0.5 }}>☦</div>
          <h1 style={{ fontFamily: T.fontDisplay, fontSize: 44, fontWeight: 600, color: T.wine, margin: "0 0 4px", letterSpacing: 1 }}>Scriptorium</h1>
          <p style={{ color: T.warmGray, fontSize: 16, letterSpacing: 3, textTransform: "uppercase", margin: 0 }}>Church Bookstore & POS</p>
          <p style={{ color: T.warmGray, fontSize: 14, marginTop: 12, maxWidth: 500 }}>Interactive UX prototype — select a section to explore all 10 screens</p>
        </div>
        <div style={{ display: "flex", gap: 16, flexWrap: "wrap", justifyContent: "center", maxWidth: 800 }}>
          {[
            { key: "pos", icon: "scan", color: T.wine, title: "Mobile POS", sub: "Screens 1–4", desc: "PIN Login → Scanner / Quick Grid → Payment → Receipt" },
            { key: "store", icon: "book", color: T.blue, title: "Online Store", sub: "Screens 5–7", desc: "Homepage → Product Detail → Checkout" },
            { key: "admin", icon: "orders", color: T.gold, title: "Admin Office", sub: "Screens 8–10", desc: "Dashboard → Add Product → Order Management" },
          ].map(s => (
            <Card key={s.key} onClick={() => { setScreen(s.key); if (s.key === "pos") setPosFlow("login"); if (s.key === "store") setStoreScreen("home"); if (s.key === "admin") setAdminScreen("dashboard"); }} style={{ width: 240, padding: 24, cursor: "pointer" }}>
              <div style={{ width: 50, height: 50, borderRadius: 14, background: `${s.color}10`, display: "flex", alignItems: "center", justifyContent: "center", marginBottom: 14 }}><Icon name={s.icon} size={24} color={s.color} /></div>
              <h3 style={{ fontSize: 18, fontWeight: 700, color: T.ink, margin: "0 0 2px" }}>{s.title}</h3>
              <Badge color={s.color}>{s.sub}</Badge>
              <p style={{ fontSize: 13, color: T.warmGray, margin: "10px 0 0", lineHeight: 1.5 }}>{s.desc}</p>
            </Card>
          ))}
        </div>
        <p style={{ color: T.warmGrayLight, fontSize: 12, marginTop: 40 }}>Navigate between screens using the flow buttons within each section</p>
      </div>
    );
  }

  const NavOverlay = () => (
    <button onClick={() => setScreen("nav")} style={{ position: "fixed", bottom: 16, left: 16, zIndex: 100, padding: "8px 16px", borderRadius: 20, background: "rgba(44,24,16,0.8)", color: T.white, border: "none", cursor: "pointer", fontFamily: T.font, fontSize: 13, fontWeight: 600, display: "flex", alignItems: "center", gap: 6, boxShadow: "0 4px 12px rgba(0,0,0,0.15)" }}>
      ← {labels[screen]}
    </button>
  );

  const ScreenTag = ({ num, title }) => (
    <div style={{ position: "fixed", top: 12, right: 12, zIndex: 100, padding: "6px 14px", borderRadius: 20, background: "rgba(44,24,16,0.7)", color: T.white, fontFamily: T.font, fontSize: 12, fontWeight: 600 }}>Screen {num}: {title}</div>
  );

  const Fonts = () => <link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;500;600;700&family=DM+Sans:wght@400;500;600;700;800&display=swap" rel="stylesheet" />;

  if (screen === "pos") return (<><Fonts /><NavOverlay />{posFlow === "login" && <><ScreenTag num="1" title="PIN Login" /><PINLogin onLogin={() => setPosFlow("pos")} /></>}{posFlow === "pos" && <><ScreenTag num="2" title="Main POS" /><POSMain onCheckout={() => setPosFlow("payment")} /></>}{posFlow === "payment" && <><ScreenTag num="3" title="Payment" /><PaymentScreen total={35.95} onBack={() => setPosFlow("pos")} onComplete={() => setPosFlow("complete")} /></>}{posFlow === "complete" && <><ScreenTag num="4" title="Complete" /><TransactionComplete total={35.95} changeDue={4.05} onNewSale={() => setPosFlow("pos")} /></>}</>);

  if (screen === "store") return (<><Fonts /><NavOverlay />{storeScreen === "home" && <><ScreenTag num="5" title="Storefront" /><Storefront onProduct={(p) => { setSelectedProduct(p); setStoreScreen("detail"); }} /><button onClick={() => setStoreScreen("checkout")} style={{ position: "fixed", bottom: 16, right: 16, zIndex: 100, padding: "10px 20px", borderRadius: 20, background: T.wine, color: T.white, border: "none", cursor: "pointer", fontFamily: T.font, fontSize: 13, fontWeight: 600, boxShadow: `0 4px 12px rgba(107,39,55,0.25)` }}>Skip to Checkout →</button></>}{storeScreen === "detail" && <><ScreenTag num="6" title="Product Detail" /><ProductDetail product={selectedProduct} onBack={() => setStoreScreen("home")} /></>}{storeScreen === "checkout" && <><ScreenTag num="7" title="Checkout" /><OnlineCheckout onBack={() => setStoreScreen("home")} /></>}</>);

  if (screen === "admin") return (<><Fonts /><NavOverlay />{adminScreen === "dashboard" && <><ScreenTag num="8" title="Dashboard" /><AdminDashboard /><div style={{ position: "fixed", bottom: 16, right: 16, zIndex: 100, display: "flex", gap: 8 }}><button onClick={() => setAdminScreen("add")} style={{ padding: "10px 20px", borderRadius: 20, background: T.gold, color: T.white, border: "none", cursor: "pointer", fontFamily: T.font, fontSize: 13, fontWeight: 600 }}>Screen 9: Add Product →</button><button onClick={() => setAdminScreen("orders")} style={{ padding: "10px 20px", borderRadius: 20, background: T.blue, color: T.white, border: "none", cursor: "pointer", fontFamily: T.font, fontSize: 13, fontWeight: 600 }}>Screen 10: Orders →</button></div></>}{adminScreen === "add" && <><ScreenTag num="9" title="Add Product" /><AddProduct onBack={() => setAdminScreen("dashboard")} /></>}{adminScreen === "orders" && <><ScreenTag num="10" title="Orders" /><OrderManagement onBack={() => setAdminScreen("dashboard")} /></>}</>);

  return null;
}
