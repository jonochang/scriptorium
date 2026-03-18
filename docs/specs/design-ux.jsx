import { useState, useEffect } from "react";

// ─── Design tokens ───
const serif = "'Source Serif 4', Georgia, serif";
const sans = "'Source Sans 3', 'Segoe UI', system-ui, sans-serif";
const mono = "'JetBrains Mono', 'SF Mono', monospace";

const C = {
  dark: "#3a2f25", cream: "#f5f1ea", card: "#ffffff", bg: "#f0ebe2",
  border: "#e0d9cd", borderLight: "#ede8df",
  textDark: "#3a2f25", textMid: "#6b5e4f", textMuted: "#8a7e6b", textFaint: "#a89e8e",
  accent: "#6b1c2a", accentHover: "#7d2234", accentLight: "#f9f0f2",
  gold: "#8b6914", goldBg: "#faf3dc", goldBorder: "#e8d99b",
  green: "#2d6b3f", greenBg: "#eaf5ee", greenBorder: "#b4dbc3",
  red: "#9e2b2b", redBg: "#fdeaea",
  wine: "#4a1525", wineLight: "#6b2640", wineMid: "#8a4060",
  posBtn: "rgba(180,140,160,0.35)", posBtnBorder: "rgba(200,170,185,0.4)",
};

// ─── Shared components ───
function Btn({ children, variant = "outline", small, onClick, style, disabled }) {
  const base = {
    fontFamily: sans, fontSize: small ? 12 : 13, fontWeight: variant === "primary" || variant === "danger" ? 600 : 500,
    padding: small ? "5px 14px" : "8px 18px", borderRadius: 8, cursor: disabled ? "default" : "pointer",
    transition: "all 0.15s", opacity: disabled ? 0.5 : 1, display: "inline-flex", alignItems: "center", gap: 6,
  };
  const v = {
    outline: { background: "transparent", border: `1px solid ${C.border}`, color: C.textMid },
    primary: { background: C.accent, border: `1px solid ${C.accent}`, color: "#fff" },
    dark: { background: C.dark, border: "none", color: C.cream },
    danger: { background: C.accent, border: `1px solid ${C.accent}`, color: "#fff" },
    ghost: { background: "transparent", border: "1px solid transparent", color: C.textMuted },
    gold: { background: C.gold, border: `1px solid ${C.gold}`, color: "#fff" },
  };
  return <button onClick={onClick} disabled={disabled} style={{ ...base, ...v[variant], ...style }}>{children}</button>;
}

function Badge({ children, color = C.greenBg, text = C.green, border = C.greenBorder }) {
  return <span style={{ fontSize: 11, fontWeight: 600, padding: "3px 10px", borderRadius: 20, background: color, color: text, border: `1px solid ${border}` }}>{children}</span>;
}

function StatCard({ label, value, highlighted, wide, color }) {
  return (
    <div style={{
      flex: wide ? "1 1 280px" : "1 1 0", padding: "18px 20px", borderRadius: 10,
      background: highlighted ? C.dark : C.card, border: highlighted ? "none" : `1px solid ${C.borderLight}`,
      minWidth: 0,
    }}>
      <div style={{ fontSize: 10, fontWeight: 600, letterSpacing: 1.2, textTransform: "uppercase", color: highlighted ? "rgba(245,241,234,0.6)" : C.textMuted, marginBottom: 6 }}>{label}</div>
      <div style={{ fontSize: 22, fontWeight: 700, fontFamily: serif, color: color ? color : highlighted ? C.cream : C.textDark }}>{value}</div>
    </div>
  );
}

function SectionLabel({ children }) {
  return <div style={{ fontSize: 10, fontWeight: 600, letterSpacing: 1.4, textTransform: "uppercase", color: C.textMuted, marginBottom: 8 }}>{children}</div>;
}

function Card({ children, style }) {
  return <div style={{ background: C.card, borderRadius: 12, border: `1px solid ${C.borderLight}`, padding: "24px 28px", ...style }}>{children}</div>;
}

function TabBar({ tabs, active, onChange }) {
  return (
    <div style={{ display: "flex", background: C.borderLight, borderRadius: 10, padding: 3, gap: 2, width: "fit-content" }}>
      {tabs.map(t => (
        <button key={t.key} onClick={() => onChange(t.key)} style={{
          padding: "7px 18px", fontSize: 13, fontWeight: active === t.key ? 600 : 400,
          color: active === t.key ? "#fff" : C.textMid, background: active === t.key ? C.dark : "transparent",
          border: "none", borderRadius: 8, cursor: "pointer", fontFamily: sans, transition: "all 0.15s", whiteSpace: "nowrap",
        }}>{t.label}</button>
      ))}
    </div>
  );
}

function FilterPills({ items, active, onChange }) {
  return (
    <div style={{ display: "flex", gap: 4, flexWrap: "wrap" }}>
      {items.map(i => (
        <button key={i.key} onClick={() => onChange(i.key)} style={{
          padding: "5px 14px", fontSize: 12, fontWeight: active === i.key ? 600 : 400,
          color: active === i.key ? "#fff" : C.textMid, background: active === i.key ? C.dark : "transparent",
          border: `1px solid ${active === i.key ? C.dark : C.border}`, borderRadius: 20, cursor: "pointer", fontFamily: sans,
        }}>{i.label}</button>
      ))}
    </div>
  );
}

function Input({ label, placeholder, value, onChange, type = "text", style: s }) {
  return (
    <div style={s}>
      {label && <label style={{ display: "block", fontSize: 13, fontWeight: 600, color: C.textDark, marginBottom: 8 }}>{label}</label>}
      <input type={type} placeholder={placeholder} value={value} onChange={e => onChange?.(e.target.value)} style={{
        width: "100%", padding: "10px 14px", fontSize: 14, fontFamily: sans, border: `1px solid ${C.border}`,
        borderRadius: 8, background: C.cream, color: C.textDark,
      }} />
    </div>
  );
}

function EmptyState({ icon, text }) {
  return <div style={{ padding: "32px 20px", textAlign: "center", color: C.textMuted, fontSize: 13 }}>{icon && <div style={{ fontSize: 24, marginBottom: 8, opacity: 0.4 }}>{icon}</div>}{text}</div>;
}

function Divider() { return <div style={{ height: 1, background: C.borderLight, margin: "16px 0" }} />; }

// ═══════════════════════════════════════════════════
// NAV BARS — one per zone
// ═══════════════════════════════════════════════════

function StorefrontNav({ page, go, cartCount }) {
  const links = [
    { key: "catalog", label: "Catalog" },
    { key: "cart", label: "Cart" },
    { key: "checkout", label: "Checkout" },
  ];
  return (
    <header style={{ background: C.dark, color: C.cream, display: "flex", alignItems: "center", justifyContent: "space-between", padding: "0 32px", height: 52 }}>
      <div onClick={() => go("catalog")} style={{ display: "flex", alignItems: "center", gap: 10, fontFamily: serif, fontWeight: 700, fontSize: 16, letterSpacing: 0.5, cursor: "pointer" }}>
        <span style={{ fontSize: 18 }}>✝</span> SCRIPTORIUM
      </div>
      <nav style={{ display: "flex", alignItems: "center", gap: 6 }}>
        {links.map(l => {
          const isActive = page === l.key || (l.key === "catalog" && page.startsWith("product-"));
          return (
          <button key={l.key} onClick={() => go(l.key)} style={{
            fontSize: 13, fontWeight: isActive ? 600 : 500, fontFamily: sans,
            color: isActive ? C.cream : "rgba(245,241,234,0.5)",
            background: isActive ? "rgba(245,241,234,0.13)" : "transparent",
            padding: "5px 14px", borderRadius: 20, border: "none", cursor: "pointer", display: "flex", alignItems: "center", gap: 5,
          }}>
            {l.label}
            {l.key === "cart" && cartCount > 0 && <span style={{ background: C.accent, color: "#fff", fontSize: 10, fontWeight: 700, padding: "1px 6px", borderRadius: 10 }}>{cartCount}</span>}
          </button>
        )})}
        <div style={{ width: 1, height: 20, background: "rgba(245,241,234,0.15)", margin: "0 8px" }} />
        <button onClick={() => go("admin-signin")} style={{ fontSize: 12, fontFamily: sans, fontWeight: 500, color: "rgba(245,241,234,0.35)", background: "none", border: "none", cursor: "pointer", padding: "5px 10px" }}>Admin</button>
      </nav>
    </header>
  );
}

function AdminNav({ page, go }) {
  const links = [
    { key: "dashboard", label: "Dashboard" },
    { key: "orders", label: "Orders" },
    { key: "intake", label: "Intake" },
  ];
  return (
    <header style={{ background: C.dark, color: C.cream, display: "flex", alignItems: "center", justifyContent: "space-between", padding: "0 32px", height: 52 }}>
      <div onClick={() => go("dashboard")} style={{ display: "flex", alignItems: "center", gap: 10, fontFamily: serif, fontWeight: 700, fontSize: 16, letterSpacing: 0.5, cursor: "pointer" }}>
        <span style={{ fontSize: 18 }}>✝</span> SCRIPTORIUM
        <span style={{ fontSize: 10, fontWeight: 600, letterSpacing: 1, background: "rgba(245,241,234,0.12)", padding: "3px 10px", borderRadius: 12, marginLeft: 4 }}>ADMIN</span>
      </div>
      <nav style={{ display: "flex", alignItems: "center", gap: 6 }}>
        {links.map(l => (
          <button key={l.key} onClick={() => go(l.key)} style={{
            fontSize: 13, fontWeight: page === l.key ? 600 : 500, fontFamily: sans,
            color: page === l.key ? C.cream : "rgba(245,241,234,0.5)",
            background: page === l.key ? "rgba(245,241,234,0.13)" : "transparent",
            padding: "5px 14px", borderRadius: 20, border: "none", cursor: "pointer",
          }}>{l.label}</button>
        ))}
        <div style={{ width: 1, height: 20, background: "rgba(245,241,234,0.15)", margin: "0 8px" }} />
        <button onClick={() => go("catalog")} style={{ fontSize: 12, fontFamily: sans, fontWeight: 500, color: "rgba(245,241,234,0.4)", background: "none", border: "none", cursor: "pointer", padding: "5px 8px" }}>Store</button>
        <button onClick={() => go("pos")} style={{ fontSize: 12, fontFamily: sans, fontWeight: 500, color: "rgba(245,241,234,0.4)", background: "none", border: "none", cursor: "pointer", padding: "5px 8px" }}>POS</button>
        <button onClick={() => go("admin-signin")} style={{ fontSize: 12, fontFamily: sans, fontWeight: 500, color: "rgba(245,241,234,0.4)", background: "none", border: "none", cursor: "pointer", padding: "5px 8px" }}>Sign out</button>
      </nav>
    </header>
  );
}

function GatewayNav({ go }) {
  return (
    <header style={{ background: C.dark, color: C.cream, display: "flex", alignItems: "center", justifyContent: "space-between", padding: "0 32px", height: 52 }}>
      <div style={{ display: "flex", alignItems: "center", gap: 10, fontFamily: serif, fontWeight: 700, fontSize: 16, letterSpacing: 0.5 }}>
        <span style={{ fontSize: 18 }}>✝</span> SCRIPTORIUM
      </div>
      <button onClick={() => go("catalog")} style={{
        background: "none", border: "1px solid rgba(245,241,234,0.2)", color: "rgba(245,241,234,0.7)",
        fontSize: 13, fontWeight: 500, fontFamily: sans, padding: "6px 16px", borderRadius: 8, cursor: "pointer",
        display: "flex", alignItems: "center", gap: 6,
      }}>
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"><path d="M10 12L6 8l4-4"/></svg>
        Back to store
      </button>
    </header>
  );
}

function Footer({ zone, go }) {
  return (
    <footer style={{ maxWidth: 1060, margin: "0 auto", padding: "20px 24px", borderTop: `1px solid ${C.border}`, display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: 12, color: C.textFaint }}>
      <span>Scriptorium 2026. Parish browsing, intake & Sunday-close reconciliation.</span>
      {zone === "storefront" && (
        <div style={{ display: "flex", gap: 16 }}>
          {["Catalog", "Cart"].map(l => <span key={l} onClick={() => go(l.toLowerCase())} style={{ cursor: "pointer", fontWeight: 500 }}>{l}</span>)}
        </div>
      )}
      {zone === "admin" && (
        <div style={{ display: "flex", gap: 16 }}>
          {["Dashboard", "Orders", "Intake"].map(l => <span key={l} onClick={() => go(l.toLowerCase())} style={{ cursor: "pointer", fontWeight: 500 }}>{l}</span>)}
        </div>
      )}
    </footer>
  );
}

// ═══════════════════════════════════════════════════
// PAGE: Catalog (Storefront)
// ═══════════════════════════════════════════════════
const PRODUCTS = [
  { id: 1, title: "The Purpose Driven Life", author: "Rick Warren", price: 18.99, cat: "Books", stock: 5, desc: "A practical invitation to reorder ordinary life around prayer, service, and long obedience.", publisher: "Zondervan", isbn: "9780310337508", binding: "Softcover", pages: 336 },
  { id: 2, title: "Knowing God", author: "J.I. Packer", price: 20.99, cat: "Books", stock: 8, desc: "A theology shelf staple for readers who want doctrine with warmth, confidence, and pastoral clarity.", publisher: "IVP Books", isbn: "9780830816507", binding: "Softcover", pages: 286 },
  { id: 3, title: "Celebration of Discipline", author: "Richard Foster", price: 16.99, cat: "Books", stock: 4, desc: "A steady guide to spiritual disciplines that serves parish reading groups, gifts, and personal devotion.", publisher: "HarperOne", isbn: "9780061239892", binding: "Softcover", pages: 272 },
  { id: 4, title: "Orthodoxy", author: "G.K. Chesterton", price: 17.99, cat: "Books", stock: 6, desc: "Chesterton's vivid defense of Christian belief, ideal for curious browsers and after-liturgy discussion circles.", publisher: "Ignatius Press", isbn: "9780898705522", binding: "Softcover", pages: 168 },
  { id: 5, title: "Pocket Prayer Rope", author: "Parish Workshop", price: 12.99, cat: "Gifts", stock: 2, desc: "A tactile devotional gift that sits well in prayer corners, chrismation baskets, and feast-day giving.", publisher: "Parish Workshop", isbn: "—", binding: "Handmade", pages: null },
  { id: 6, title: "Feast Day Greeting Card Set", author: "Scriptorium Press", price: 9.99, cat: "Gifts", stock: 0, desc: "A gentle stationery gift for feast days, hospital visits, and hand-written parish encouragement.", publisher: "Scriptorium Press", isbn: "—", binding: "Card set (12)", pages: null },
  { id: 7, title: "Hand-Painted Icon (Small)", author: "St. Sophia Studio", price: 45.00, cat: "Icons", stock: 3, desc: "A 6×8 inch hand-painted icon on wood panel, suitable for home prayer corners.", publisher: "St. Sophia Studio", isbn: "—", binding: "Wood panel", pages: null },
  { id: 8, title: "Theotokos Triptych", author: "Mount Tabor Iconography", price: 85.00, cat: "Icons", stock: 1, desc: "A folding triptych of the Theotokos, crafted for travel or bedside devotion.", publisher: "Mount Tabor Iconography", isbn: "—", binding: "Hinged wood", pages: null },
];

function CatalogPage({ go, addToCart }) {
  const [search, setSearch] = useState("");
  const [cat, setCat] = useState("all");
  const cats = ["all", ...new Set(PRODUCTS.map(p => p.cat))];
  const filtered = PRODUCTS.filter(p => (cat === "all" || p.cat === cat) && (search === "" || p.title.toLowerCase().includes(search.toLowerCase()) || p.author.toLowerCase().includes(search.toLowerCase())));

  return (
    <main style={{ maxWidth: 960, margin: "0 auto", padding: "0 24px 40px" }}>
      <div style={{ textAlign: "center", padding: "40px 0 32px" }}>
        <h1 style={{ fontFamily: serif, fontSize: 30, fontWeight: 700, color: C.textDark, marginBottom: 8 }}>Feed your soul.</h1>
        <p style={{ fontSize: 14, color: C.textMid }}>Find books for parish reading, gifting, and liturgical practice.</p>
      </div>
      <Card style={{ marginBottom: 24 }}>
        <div style={{ fontSize: 13, fontWeight: 600, color: C.textDark, marginBottom: 12 }}>Search catalog</div>
        <div style={{ display: "flex", gap: 10, marginBottom: 16 }}>
          <input placeholder="Try Discipline or Foster" value={search} onChange={e => setSearch(e.target.value)} style={{ flex: 1, padding: "10px 14px", fontSize: 14, fontFamily: sans, border: `1px solid ${C.border}`, borderRadius: 8, background: C.cream, color: C.textDark }} />
          <Btn variant="primary">Search</Btn>
        </div>
        <FilterPills items={cats.map(c => ({ key: c, label: c === "all" ? `All ${PRODUCTS.length}` : `${c} ${PRODUCTS.filter(p => p.cat === c).length}` }))} active={cat} onChange={setCat} />
      </Card>
      <div style={{ textAlign: "right", fontSize: 13, color: C.textMuted, marginBottom: 12 }}>{filtered.length} titles</div>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(200px, 1fr))", gap: 16 }}>
        {filtered.map(p => (
          <Card key={p.id} style={{ padding: 0, overflow: "hidden" }}>
            <div onClick={() => go(`product-${p.id}`)} style={{ height: 140, background: `linear-gradient(135deg, ${C.gold}33, ${C.accent}22)`, position: "relative", display: "flex", alignItems: "end", padding: 10, cursor: "pointer" }}>
              {p.stock > 3 && <Badge>In stock</Badge>}
              {p.stock > 0 && p.stock <= 3 && <Badge color={C.goldBg} text={C.gold} border={C.goldBorder}>Only {p.stock} left</Badge>}
              {p.stock === 0 && <Badge color={C.redBg} text={C.red} border="#f0c0c0">Out of stock</Badge>}
            </div>
            <div style={{ padding: "14px 16px 18px" }}>
              <div style={{ fontSize: 11, color: C.textMuted, marginBottom: 4 }}>{p.cat}</div>
              <div onClick={() => go(`product-${p.id}`)} style={{ fontSize: 15, fontWeight: 700, fontFamily: serif, color: C.textDark, lineHeight: 1.3, marginBottom: 4, cursor: "pointer" }}>{p.title}</div>
              <div style={{ fontSize: 12, color: C.textMid, marginBottom: 8 }}>{p.author}</div>
              <div style={{ fontSize: 12, color: C.textMuted, lineHeight: 1.5, marginBottom: 14 }}>{p.desc}</div>
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <span style={{ fontSize: 15, fontWeight: 700, color: C.textDark }}>${p.price.toFixed(2)}</span>
                {p.stock > 0 && <Btn variant="primary" small onClick={() => addToCart(p)}>Add</Btn>}
              </div>
              <div style={{ marginTop: 10 }}>
                <Btn small variant="outline" onClick={() => go(`product-${p.id}`)}>View details</Btn>
              </div>
            </div>
          </Card>
        ))}
      </div>
    </main>
  );
}

// ═══════════════════════════════════════════════════
// PAGE: Product Detail (Storefront)
// ═══════════════════════════════════════════════════
function ProductDetailPage({ productId, go, addToCart }) {
  const [qty, setQty] = useState(1);
  const product = PRODUCTS.find(p => p.id === productId);
  if (!product) return <main style={{ maxWidth: 960, margin: "0 auto", padding: "60px 24px", textAlign: "center" }}><p style={{ color: C.textMuted }}>Product not found.</p><Btn variant="outline" onClick={() => go("catalog")}>Back to catalog</Btn></main>;

  const related = PRODUCTS.filter(p => p.id !== product.id && p.cat === product.cat).slice(0, 2);
  const stockBadge = product.stock > 3 ? <Badge>In stock</Badge>
    : product.stock > 0 ? <Badge color={C.goldBg} text={C.gold} border={C.goldBorder}>Only {product.stock} left</Badge>
    : <Badge color={C.redBg} text={C.red} border="#f0c0c0">Out of stock</Badge>;

  const details = [
    ["Publisher", product.publisher],
    ["ISBN", product.isbn],
    ["Binding", product.binding],
    product.pages ? ["Pages", `${product.pages} pages`] : null,
  ].filter(Boolean);

  return (
    <main style={{ maxWidth: 960, margin: "0 auto", padding: "0 24px 40px" }}>
      <div style={{ padding: "32px 0 24px" }}>
        <button onClick={() => go("catalog")} style={{ display: "inline-flex", alignItems: "center", gap: 5, fontSize: 13, color: C.textMuted, background: "none", border: "none", cursor: "pointer", fontFamily: sans, padding: 0, marginBottom: 12 }}>
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"><path d="M10 12L6 8l4-4"/></svg>
          Back to catalog
        </button>
        <h1 style={{ fontFamily: serif, fontSize: 28, fontWeight: 700, color: C.textDark }}>{product.title}</h1>
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 24 }}>
        {/* Left: cover image area */}
        <Card style={{ padding: 0, overflow: "hidden" }}>
          <div style={{
            height: 440, background: `linear-gradient(145deg, ${C.gold}44, ${C.accent}33)`,
            display: "flex", alignItems: "flex-end", padding: 32, position: "relative",
          }}>
            {/* Faux cover overlay */}
            <div style={{
              background: "rgba(58,47,37,0.45)", backdropFilter: "blur(4px)", borderRadius: 10,
              padding: "20px 24px", maxWidth: 280,
            }}>
              <div style={{ fontSize: 9, fontWeight: 600, letterSpacing: 1.4, textTransform: "uppercase", color: "rgba(245,241,234,0.6)", marginBottom: 8 }}>Parish shelf edition</div>
              <div style={{ fontFamily: serif, fontSize: 20, fontWeight: 700, color: C.cream, lineHeight: 1.25, marginBottom: 6 }}>{product.title}</div>
              <div style={{ fontSize: 13, color: "rgba(245,241,234,0.75)" }}>{product.author}</div>
            </div>
          </div>
        </Card>

        {/* Right: product info */}
        <Card>
          <div style={{ marginBottom: 16 }}>
            <span style={{ display: "inline-block", fontSize: 11, fontWeight: 600, color: C.textMid, padding: "4px 12px", background: C.cream, border: `1px solid ${C.borderLight}`, borderRadius: 20, marginBottom: 14 }}>{product.cat}</span>
            <h2 style={{ fontFamily: serif, fontSize: 22, fontWeight: 700, color: C.textDark, lineHeight: 1.3, marginBottom: 6 }}>{product.title}</h2>
            <div style={{ fontSize: 14, color: C.textMid, marginBottom: 16 }}>{product.author}</div>
            <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20 }}>
              <span style={{ fontSize: 22, fontWeight: 700, fontFamily: serif, color: C.accent }}>${product.price.toFixed(2)}</span>
              {stockBadge}
            </div>
          </div>

          <Divider />

          {/* Description */}
          <div style={{ marginBottom: 20 }}>
            <div style={{ fontSize: 14, fontWeight: 700, color: C.textDark, marginBottom: 8 }}>Description</div>
            <p style={{ fontSize: 14, color: C.textMid, lineHeight: 1.65 }}>{product.desc}</p>
          </div>

          {/* Details table */}
          <div style={{ marginBottom: 20 }}>
            <div style={{ fontSize: 14, fontWeight: 700, color: C.textDark, marginBottom: 10 }}>Details</div>
            {details.map(([label, value]) => (
              <div key={label} style={{ display: "flex", justifyContent: "space-between", padding: "8px 0", borderBottom: `1px solid ${C.borderLight}` }}>
                <span style={{ fontSize: 13, color: C.textMid }}>{label}</span>
                <span style={{ fontSize: 13, fontWeight: 600, color: C.textDark, fontFamily: label === "ISBN" ? mono : sans }}>{value}</span>
              </div>
            ))}
          </div>

          {/* Quantity + add to cart */}
          {product.stock > 0 && <>
            <div style={{ fontSize: 13, fontWeight: 600, color: C.textDark, marginBottom: 8 }}>Quantity</div>
            <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 16 }}>
              <button onClick={() => setQty(Math.max(1, qty - 1))} style={{ width: 36, height: 36, borderRadius: 8, border: `1px solid ${C.border}`, background: "transparent", fontSize: 16, cursor: "pointer", color: C.textMid }}>−</button>
              <input value={qty} onChange={e => { const v = parseInt(e.target.value); if (!isNaN(v) && v > 0) setQty(Math.min(v, product.stock)); }}
                style={{ width: 56, padding: "8px 12px", fontSize: 14, fontFamily: sans, border: `1px solid ${C.border}`, borderRadius: 8, background: C.cream, textAlign: "center", color: C.textDark }} />
              <button onClick={() => setQty(Math.min(qty + 1, product.stock))} style={{ width: 36, height: 36, borderRadius: 8, border: `1px solid ${C.border}`, background: "transparent", fontSize: 16, cursor: "pointer", color: C.textMid }}>+</button>
            </div>
            <button onClick={() => { addToCart(product, qty); }} style={{
              width: "100%", padding: "13px", borderRadius: 10, background: C.accent, border: "none",
              color: "#fff", fontSize: 15, fontWeight: 700, fontFamily: sans, cursor: "pointer", marginBottom: 10,
              transition: "background 0.15s",
            }}>Add to Cart — ${(product.price * qty).toFixed(2)}</button>
            <button onClick={() => { addToCart(product, qty); go("checkout"); }} style={{
              width: "100%", padding: "12px", borderRadius: 10, background: "transparent",
              border: `1px solid ${C.border}`, color: C.textMid, fontSize: 14, fontWeight: 500,
              fontFamily: sans, cursor: "pointer", marginBottom: 16,
            }}>Proceed to checkout</button>
          </>}

          {product.stock === 0 && <div style={{ padding: "14px 20px", background: C.redBg, borderRadius: 10, fontSize: 13, color: C.red, fontWeight: 500, marginBottom: 16 }}>This title is currently out of stock. Check back soon.</div>}

          <div style={{ padding: "10px 14px", background: C.cream, borderRadius: 8, fontSize: 13, color: C.textMuted }}>
            {product.stock > 0 ? "Ready to add this title to the cart." : "Not available for purchase at this time."}
          </div>
        </Card>
      </div>

      {/* Related titles */}
      {related.length > 0 && <div style={{ marginTop: 28 }}>
        <SectionLabel>Related titles</SectionLabel>
        <div style={{ display: "flex", flexDirection: "column", gap: 0 }}>
          {related.map(r => (
            <div key={r.id} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "16px 20px", background: C.card, border: `1px solid ${C.borderLight}`, borderRadius: 10, marginBottom: 8 }}>
              <div>
                <div style={{ fontSize: 14, fontWeight: 700, color: C.textDark }}>{r.title}</div>
                <div style={{ fontSize: 12, color: C.textMuted }}>{r.author} · {r.cat}</div>
              </div>
              <div style={{ display: "flex", gap: 8 }}>
                <Btn small variant="outline" onClick={() => go(`product-${r.id}`)}>View</Btn>
                {r.stock > 0 && <Btn small variant="primary" onClick={() => addToCart(r)}>Add</Btn>}
              </div>
            </div>
          ))}
        </div>
      </div>}
    </main>
  );
}

// ═══════════════════════════════════════════════════
// PAGE: Cart (Storefront)
// ═══════════════════════════════════════════════════
function CartPage({ cart, updateQty, removeItem, go }) {
  const total = cart.reduce((s, i) => s + i.price * i.qty, 0);
  return (
    <main style={{ maxWidth: 960, margin: "0 auto", padding: "0 24px 40px" }}>
      <div style={{ padding: "40px 0 32px" }}>
        <h1 style={{ fontFamily: serif, fontSize: 28, fontWeight: 700, color: C.textDark, marginBottom: 8 }}>Review your basket</h1>
        <p style={{ fontSize: 14, color: C.textMid }}>Confirm quantities, keep gifting simple, and move smoothly into checkout.</p>
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 20 }}>
        <Card>
          <div style={{ fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 16 }}>Cart items</div>
          {cart.length === 0 && <EmptyState text="Your basket is empty. Browse the catalog to add items." />}
          {cart.map(item => (
            <div key={item.id} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "12px 0", borderBottom: `1px solid ${C.borderLight}` }}>
              <div>
                <div style={{ fontSize: 14, fontWeight: 600, color: C.textDark }}>{item.title}</div>
                <div style={{ fontSize: 12, color: C.textMuted }}>{item.author} · Qty {item.qty}</div>
              </div>
              <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
                <Btn small variant="outline" onClick={() => updateQty(item.id, -1)}>−</Btn>
                <Btn small variant="outline" onClick={() => updateQty(item.id, 1)}>+</Btn>
                <Btn small variant="outline" onClick={() => removeItem(item.id)}>Remove</Btn>
                <span style={{ fontSize: 14, fontWeight: 600, color: C.textDark, minWidth: 60, textAlign: "right" }}>${(item.price * item.qty).toFixed(2)}</span>
              </div>
            </div>
          ))}
          {cart.length > 0 && <>
            <div style={{ marginTop: 16, padding: "10px 14px", background: C.greenBg, borderRadius: 8, fontSize: 13, fontWeight: 600, color: C.green }}>Cart total: ${total.toFixed(2)}</div>
            <div style={{ marginTop: 16, display: "flex", gap: 10 }}>
              <Btn variant="outline" onClick={() => removeItem("all")}>Clear basket</Btn>
              <Btn variant="primary" onClick={() => go("checkout")}>Proceed to checkout</Btn>
            </div>
          </>}
        </Card>
        <Card>
          <div style={{ fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 16 }}>Recommended titles</div>
          {PRODUCTS.filter(p => !cart.find(c => c.id === p.id)).slice(0, 3).map(p => (
            <div key={p.id} style={{ display: "flex", alignItems: "center", justifyContent: "space-between", padding: "10px 0", borderBottom: `1px solid ${C.borderLight}` }}>
              <div><div style={{ fontSize: 14, fontWeight: 600, color: C.textDark }}>{p.title}</div><div style={{ fontSize: 12, color: C.textMuted }}>{p.author} · {p.cat}</div></div>
              <Btn small variant="outline" onClick={() => go("catalog")}>View</Btn>
            </div>
          ))}
        </Card>
      </div>
    </main>
  );
}

// ═══════════════════════════════════════════════════
// PAGE: Checkout (Storefront)
// ═══════════════════════════════════════════════════
function CheckoutPage({ cart, go }) {
  const [step, setStep] = useState(1);
  const [delivery, setDelivery] = useState("pickup");
  const [donation, setDonation] = useState(0);
  const subtotal = cart.reduce((s, i) => s + i.price * i.qty, 0);
  const shipping = delivery === "ship" ? 5.99 : 0;
  const tax = +(subtotal * 0.07).toFixed(2);
  const total = subtotal + shipping + tax + donation;
  const steps = [{ n: 1, label: "Details" }, { n: 2, label: "Payment" }, { n: 3, label: "Confirmation" }];

  return (
    <main style={{ maxWidth: 960, margin: "0 auto", padding: "0 24px 40px" }}>
      <div style={{ padding: "40px 0 12px" }}>
        <h1 style={{ fontFamily: serif, fontSize: 28, fontWeight: 700, color: C.textDark, marginBottom: 8 }}>Finish your order</h1>
        <p style={{ fontSize: 14, color: C.textMid, marginBottom: 24 }}>Confirm your contact details, choose any extra parish support, and place the order with confidence.</p>
        <div style={{ display: "flex", alignItems: "center", gap: 8, marginBottom: 32 }}>
          {steps.map((s, i) => (<>
            <div key={s.n} style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <div style={{ width: 24, height: 24, borderRadius: 12, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 11, fontWeight: 700, background: step >= s.n ? C.accent : C.borderLight, color: step >= s.n ? "#fff" : C.textMuted }}>{step > s.n ? "✓" : s.n}</div>
              <span style={{ fontSize: 13, fontWeight: step === s.n ? 600 : 400, color: step === s.n ? C.textDark : C.textMuted }}>{s.label}</span>
            </div>
            {i < steps.length - 1 && <div style={{ width: 40, height: 1, background: C.border }} />}
          </>))}
        </div>
      </div>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 360px", gap: 24 }}>
        <div>
          {step === 1 && <Card>
            <div style={{ fontSize: 18, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 20 }}>Contact & delivery</div>
            <Input label="Full name" placeholder="Full name" style={{ marginBottom: 16 }} />
            <Input label="Receipt email" placeholder="reader@example.com" style={{ marginBottom: 20 }} />
            <div style={{ fontSize: 13, fontWeight: 600, color: C.textDark, marginBottom: 10 }}>Delivery method</div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10, marginBottom: 20 }}>
              {[{ key: "pickup", label: "Pick up from church", sub: "Free after liturgy", price: "Free" }, { key: "ship", label: "Ship to my address", sub: "Tracked parcel delivery", price: "$5.99" }].map(d => (
                <div key={d.key} onClick={() => setDelivery(d.key)} style={{ padding: 16, borderRadius: 10, border: `2px solid ${delivery === d.key ? C.accent : C.borderLight}`, cursor: "pointer", background: delivery === d.key ? C.accentLight : "transparent" }}>
                  <div style={{ fontSize: 13, fontWeight: 600, color: C.textDark }}>{d.label}</div>
                  <div style={{ fontSize: 12, color: C.textMuted }}>{d.sub}</div>
                  <div style={{ fontSize: 13, fontWeight: 700, color: delivery === d.key ? C.accent : C.textMid, marginTop: 8 }}>{d.price}</div>
                </div>
              ))}
            </div>
            <Input label="Order note" placeholder="Parish pickup note, gifting instructions, or follow-up…" style={{ marginBottom: 20 }} />
            <div style={{ padding: "16px 20px", background: C.cream, borderRadius: 10, marginBottom: 20 }}>
              <div style={{ fontSize: 13, fontWeight: 600, color: C.textDark, marginBottom: 4 }}>Parish support</div>
              <div style={{ fontSize: 12, color: C.textMuted, marginBottom: 10 }}>Add a voluntary contribution to help keep the bookstore running.</div>
              <div style={{ display: "flex", gap: 6 }}>
                {[0, 2, 5, 10].map(a => (
                  <button key={a} onClick={() => setDonation(a)} style={{
                    padding: "6px 14px", fontSize: 12, fontWeight: donation === a ? 600 : 400, borderRadius: 20, cursor: "pointer", fontFamily: sans,
                    background: donation === a ? C.dark : "transparent", color: donation === a ? "#fff" : C.textMid, border: `1px solid ${donation === a ? C.dark : C.border}`,
                  }}>{a === 0 ? "None" : `$${a}`}</button>
                ))}
              </div>
            </div>
            <div style={{ display: "flex", gap: 10 }}>
              <Btn variant="outline" onClick={() => go("cart")}>Back to cart</Btn>
              <Btn variant="primary" onClick={() => setStep(2)}>Continue to payment</Btn>
            </div>
          </Card>}
          {step === 2 && <Card>
            <div style={{ fontSize: 18, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 20 }}>Payment details</div>
            <Input label="Card number" placeholder="1234 5678 9012 3456" style={{ marginBottom: 16 }} />
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 24 }}>
              <Input label="Expiry" placeholder="MM / YY" />
              <Input label="CVC" placeholder="123" />
            </div>
            <div style={{ display: "flex", gap: 10 }}>
              <Btn variant="outline" onClick={() => setStep(1)}>Back</Btn>
              <Btn variant="primary" onClick={() => setStep(3)}>Place order · ${total.toFixed(2)}</Btn>
            </div>
          </Card>}
          {step === 3 && <Card style={{ background: C.greenBg, border: `1px solid ${C.greenBorder}` }}>
            <div style={{ fontSize: 22, fontWeight: 700, fontFamily: serif, color: C.green, marginBottom: 8 }}>Order confirmed!</div>
            <div style={{ fontSize: 14, color: C.green, marginBottom: 16 }}>Order #SCR-2026-0042. A receipt has been sent to your email.</div>
            <div style={{ display: "flex", gap: 10 }}>
              <Btn variant="dark" onClick={() => go("catalog")}>Continue shopping</Btn>
            </div>
          </Card>}
        </div>
        <Card style={{ height: "fit-content", position: "sticky", top: 20 }}>
          <div style={{ fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 16 }}>Order summary</div>
          {cart.map(item => (
            <div key={item.id} style={{ display: "flex", justifyContent: "space-between", marginBottom: 12 }}>
              <div><div style={{ fontSize: 13, fontWeight: 600, color: C.textDark }}>{item.title}</div><div style={{ fontSize: 12, color: C.textMuted }}>{item.author} · Qty {item.qty}</div></div>
              <span style={{ fontSize: 13, fontWeight: 600 }}>${(item.price * item.qty).toFixed(2)}</span>
            </div>
          ))}
          <Divider />
          {[["Subtotal", subtotal], ["Shipping", shipping || "Free"], ["Tax", tax], ["Parish support", donation || "$0.00"]].map(([l, v]) => (
            <div key={l} style={{ display: "flex", justifyContent: "space-between", fontSize: 13, color: C.textMid, marginBottom: 8 }}>
              <span>{l}</span><span style={{ fontWeight: 500 }}>{typeof v === "number" ? `$${v.toFixed(2)}` : v}</span>
            </div>
          ))}
          <Divider />
          <div style={{ display: "flex", justifyContent: "space-between", fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark }}>
            <span>Total</span><span>${total.toFixed(2)}</span>
          </div>
        </Card>
      </div>
    </main>
  );
}

// ═══════════════════════════════════════════════════
// PAGE: Admin Sign-In (Gateway)
// ═══════════════════════════════════════════════════
function AdminSignInPage({ go }) {
  const [status, setStatus] = useState("idle");
  return (
    <main style={{ maxWidth: 520, margin: "0 auto", padding: "0 24px 80px" }}>
      <div style={{ textAlign: "center", paddingTop: 56, paddingBottom: 40 }}>
        <h1 style={{ fontFamily: serif, fontSize: 28, fontWeight: 700, color: C.textDark, lineHeight: 1.25, marginBottom: 10 }}>Sign in to the admin office</h1>
        <p style={{ fontSize: 14, color: C.textMid, lineHeight: 1.6, maxWidth: 380, margin: "0 auto" }}>Manage the dashboard, intake new stock, and review orders and inventory.</p>
      </div>
      <div style={{ background: C.card, borderRadius: 12, border: `1px solid ${C.borderLight}`, boxShadow: "0 1px 3px rgba(58,47,37,0.04), 0 4px 16px rgba(58,47,37,0.03)", overflow: "hidden" }}>
        <div style={{ padding: "32px 32px 28px" }}>
          <Input label="Username" placeholder="Parish admin username" style={{ marginBottom: 16 }} />
          <Input label="Password" placeholder="Password" type="password" style={{ marginBottom: 28 }} />
          <div style={{ display: "flex", gap: 10, alignItems: "center" }}>
            <Btn variant="primary" onClick={() => go("dashboard")}>Sign in</Btn>
            <Btn variant="outline" onClick={() => go("catalog")}>Cancel</Btn>
          </div>
        </div>
        <div style={{ borderTop: `1px solid ${C.borderLight}`, padding: "13px 32px", background: C.cream, fontSize: 13, color: C.textMuted, display: "flex", alignItems: "center", gap: 6 }}>
          <svg width="13" height="13" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round"><rect x="3" y="7" width="10" height="7" rx="2"/><path d="M5 7V5a3 3 0 016 0v2"/></svg>
          Credentials are stored locally. No data is sent to third parties.
        </div>
      </div>
      <div style={{ marginTop: 20, background: C.card, borderRadius: 12, border: `1px solid ${C.borderLight}`, padding: "20px 32px", display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <div>
          <div style={{ fontSize: 14, fontWeight: 600, color: C.textDark, marginBottom: 4 }}>Open the POS terminal</div>
          <div style={{ fontSize: 13, color: C.textMuted, lineHeight: 1.5 }}>Volunteers sign in with a 4-digit PIN — no admin account needed.</div>
        </div>
        <Btn variant="dark" onClick={() => go("pos")} style={{ flexShrink: 0, marginLeft: 20 }}>Launch POS</Btn>
      </div>
      <div style={{ marginTop: 40, textAlign: "center", fontSize: 12, color: C.textFaint }}>Scriptorium — parish bookstore management</div>
    </main>
  );
}

// ═══════════════════════════════════════════════════
// PAGE: Dashboard (Admin)
// ═══════════════════════════════════════════════════
function DashboardPage({ go }) {
  const [view, setView] = useState("treasurer");
  const hour = new Date().getHours();
  const greeting = hour < 12 ? "Good morning" : hour < 17 ? "Good afternoon" : "Good evening";

  return (
    <main style={{ maxWidth: 1060, margin: "0 auto", padding: "0 24px 40px" }}>
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", padding: "32px 0 20px", borderBottom: `1px solid ${C.border}`, marginBottom: 24 }}>
        <div>
          <h1 style={{ fontFamily: serif, fontSize: 26, fontWeight: 700, color: C.textDark, marginBottom: 6 }}>{greeting}, Father Michael</h1>
          <p style={{ fontSize: 13, color: C.textMuted }}>Review finances, close the table, and track parish follow-up from one dashboard.</p>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <TabBar tabs={[{ key: "treasurer", label: "Treasurer" }, { key: "sunday", label: "Sunday close" }, { key: "pastoral", label: "Pastoral" }]} active={view} onChange={setView} />
        </div>
      </div>

      {view === "treasurer" && <>
        <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20, flexWrap: "wrap" }}>
          <span style={{ fontSize: 13, fontWeight: 600, color: C.textMuted }}>Reporting window</span>
          <input type="date" defaultValue="2026-03-01" style={{ padding: "8px 12px", fontSize: 13, fontFamily: sans, border: `1px solid ${C.border}`, borderRadius: 8, background: C.cream }} />
          <span style={{ fontSize: 13, color: C.textMuted }}>to</span>
          <input type="date" defaultValue="2026-03-31" style={{ padding: "8px 12px", fontSize: 13, fontFamily: sans, border: `1px solid ${C.border}`, borderRadius: 8, background: C.cream }} />
          <Btn variant="primary" small>Refresh</Btn>
          <Btn variant="outline" small>Export CSV</Btn>
        </div>
        <div style={{ display: "flex", gap: 12, marginBottom: 24 }}>
          <StatCard label="Total sales" value="$0.00" highlighted />
          <StatCard label="POS revenue" value="$0.00" />
          <StatCard label="Online revenue" value="$0.00" />
          <StatCard label="Open IOUs" value="0 open" />
        </div>
        <Card style={{ marginBottom: 20 }}>
          <SectionLabel>Payment breakdown</SectionLabel>
          <div style={{ fontSize: 18, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 16 }}>Revenue by method</div>
          <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 12, marginBottom: 12 }}>
            {["Cash", "Card", "Online", "IOU"].map(m => (
              <div key={m} style={{ padding: "14px 16px", textAlign: "center", borderRadius: 8, background: C.cream, border: `1px solid ${C.borderLight}` }}>
                <div style={{ fontSize: 12, color: C.textMuted }}>{m}</div>
                <div style={{ fontSize: 18, fontWeight: 700, fontFamily: serif, color: C.textDark }}>$0</div>
              </div>
            ))}
          </div>
          <div style={{ fontSize: 13, color: C.textMuted }}>Payment method totals will appear here.</div>
        </Card>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 20 }}>
          <Card>
            <SectionLabel>Orders</SectionLabel>
            <div style={{ fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 12 }}>Recent orders</div>
            <FilterPills items={[{ key: "all", label: "All" }, { key: "pos", label: "POS" }, { key: "online", label: "Online" }, { key: "iou", label: "IOU (0)" }]} active="all" onChange={() => {}} />
            <EmptyState text="No orders found for this filter." />
            <div style={{ textAlign: "center" }}><Btn variant="outline" onClick={() => go("orders")}>Open full page</Btn></div>
          </Card>
          <Card>
            <SectionLabel>Inventory</SectionLabel>
            <div style={{ fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 12 }}>Stock & categories</div>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 10, marginBottom: 12 }}>
              <div style={{ padding: "12px 16px", textAlign: "center", borderRadius: 8, border: `1px solid ${C.borderLight}` }}><div style={{ fontSize: 12, color: C.textMuted }}>Products</div><div style={{ fontSize: 18, fontWeight: 700, fontFamily: serif }}>{PRODUCTS.length}</div></div>
              <div style={{ padding: "12px 16px", textAlign: "center", borderRadius: 8, border: `1px solid ${C.borderLight}` }}><div style={{ fontSize: 12, color: C.textMuted }}>Low stock</div><div style={{ fontSize: 18, fontWeight: 700, fontFamily: serif, color: C.gold }}>{PRODUCTS.filter(p => p.stock > 0 && p.stock <= 3).length}</div></div>
            </div>
          </Card>
        </div>
        <div style={{ marginTop: 16, padding: "12px 20px", background: C.goldBg, borderRadius: 8, border: `1px solid ${C.goldBorder}`, fontSize: 13, color: C.gold }}>No paid sales were recorded in the selected window.</div>
      </>}

      {view === "sunday" && <>
        <Card>
          <div style={{ fontSize: 18, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 20 }}>Sunday close checklist</div>
          {["Count the register", "Reconcile POS totals", "Settle outstanding IOUs", "Check low-stock titles", "Leave a note for the next volunteer", "Sign off & lock register"].map((task, i) => (
            <label key={i} style={{ display: "flex", alignItems: "center", gap: 12, padding: "14px 0", borderBottom: i < 5 ? `1px solid ${C.borderLight}` : "none", cursor: "pointer" }}>
              <input type="checkbox" style={{ width: 18, height: 18, accentColor: C.accent }} />
              <span style={{ fontSize: 14, color: C.textDark }}>{task}</span>
            </label>
          ))}
        </Card>
      </>}

      {view === "pastoral" && <>
        <Card>
          <div style={{ fontSize: 18, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 16 }}>Follow-ups due</div>
          <EmptyState text="No follow-ups scheduled. Pastoral notes will appear here when added." />
        </Card>
      </>}
    </main>
  );
}

// ═══════════════════════════════════════════════════
// PAGE: Orders / Inventory (Admin)
// ═══════════════════════════════════════════════════
function OrdersPage({ go }) {
  const [tab, setTab] = useState("orders");
  const [filter, setFilter] = useState("all");
  const [stockFilter, setStockFilter] = useState("all");

  return (
    <main style={{ maxWidth: 1060, margin: "0 auto", padding: "0 24px 40px" }}>
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", padding: "32px 0 20px" }}>
        <div>
          <h1 style={{ fontFamily: serif, fontSize: 26, fontWeight: 700, color: C.textDark, marginBottom: 6 }}>Order Management</h1>
          <p style={{ fontSize: 13, color: C.textMid }}>Track paid orders, open tabs, and follow-up actions from one dedicated table.</p>
        </div>
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 10, marginBottom: 24 }}>
        <TabBar tabs={[{ key: "orders", label: "Orders" }, { key: "inventory", label: "Inventory" }]} active={tab} onChange={setTab} />
      </div>

      {tab === "orders" && <>
        <div style={{ display: "flex", gap: 12, marginBottom: 20 }}>
          <StatCard label="Orders in range" value="0" highlighted wide />
          <StatCard label="Revenue (paid)" value="$0.00" wide />
          <StatCard label="Outstanding IOUs" value="$0.00" color={C.red} wide />
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20, flexWrap: "wrap" }}>
          <input placeholder="Search orders…" style={{ padding: "9px 14px", fontSize: 13, fontFamily: sans, border: `1px solid ${C.border}`, borderRadius: 8, background: C.cream, width: 200 }} />
          <FilterPills items={[{ key: "all", label: "All" }, { key: "pos", label: "POS" }, { key: "online", label: "Online" }, { key: "iou", label: "IOU (0)" }]} active={filter} onChange={setFilter} />
          <div style={{ flex: 1 }} />
          <input type="date" defaultValue="2026-03-01" style={{ padding: "8px 12px", fontSize: 13, fontFamily: sans, border: `1px solid ${C.border}`, borderRadius: 8, background: C.cream }} />
          <span style={{ fontSize: 13, color: C.textMuted }}>to</span>
          <input type="date" defaultValue="2026-03-31" style={{ padding: "8px 12px", fontSize: 13, fontFamily: sans, border: `1px solid ${C.border}`, borderRadius: 8, background: C.cream }} />
          <Btn variant="outline" small>Export CSV</Btn>
        </div>
        <Card>
          <SectionLabel>Orders</SectionLabel>
          <div style={{ fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 16 }}>Order management</div>
          <EmptyState text="No orders found for this filter." />
          <Divider />
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", fontSize: 13, color: C.textMuted }}>
            <span>Showing 0 of 0 orders</span>
            <div style={{ width: 28, height: 28, borderRadius: 14, background: C.dark, color: "#fff", display: "flex", alignItems: "center", justifyContent: "center", fontSize: 12, fontWeight: 600 }}>1</div>
          </div>
        </Card>
      </>}

      {tab === "inventory" && <>
        <div style={{ display: "flex", gap: 12, marginBottom: 20 }}>
          <StatCard label="Total products" value={PRODUCTS.length} highlighted wide />
          <StatCard label="Retail value on shelf" value={`$${PRODUCTS.reduce((s, p) => s + p.price * p.stock, 0).toFixed(2)}`} wide />
          <StatCard label="Low stock" value={PRODUCTS.filter(p => p.stock > 0 && p.stock <= 3).length} color={C.gold} wide />
          <StatCard label="Out of stock" value={PRODUCTS.filter(p => p.stock === 0).length} color={C.red} wide />
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 20, flexWrap: "wrap" }}>
          <input placeholder="Search products…" style={{ padding: "9px 14px", fontSize: 13, fontFamily: sans, border: `1px solid ${C.border}`, borderRadius: 8, background: C.cream, width: 200 }} />
          <FilterPills items={[{ key: "all", label: "All" }].concat([...new Set(PRODUCTS.map(p => p.cat))].map(c => ({ key: c, label: c })))} active="all" onChange={() => {}} />
          <div style={{ flex: 1 }} />
          <FilterPills items={[{ key: "all", label: "All stock" }, { key: "low", label: "Low" }, { key: "out", label: "Out" }]} active={stockFilter} onChange={setStockFilter} />
          <Btn variant="primary" small onClick={() => go("intake")}>Add product</Btn>
        </div>
        <Card>
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: 16 }}>
            <div><SectionLabel>Inventory</SectionLabel><div style={{ fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark }}>Inventory management</div></div>
            <div style={{ display: "flex", gap: 8 }}>
              <Badge color={C.cream} text={C.textMuted} border={C.border}>Use +/− to adjust stock</Badge>
              <Badge color={C.cream} text={C.textMuted} border={C.border}>Reorder threshold shown per title</Badge>
            </div>
          </div>
          <div style={{ overflowX: "auto" }}>
            <table style={{ width: "100%", borderCollapse: "collapse", fontSize: 13 }}>
              <thead>
                <tr style={{ borderBottom: `2px solid ${C.borderLight}` }}>
                  {["Title", "Category", "Price", "Stock", "Status", ""].map(h => (
                    <th key={h} style={{ textAlign: "left", padding: "10px 12px", fontWeight: 600, color: C.textMuted, fontSize: 11, textTransform: "uppercase", letterSpacing: 0.8 }}>{h}</th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {PRODUCTS.filter(p => stockFilter === "all" || (stockFilter === "low" && p.stock > 0 && p.stock <= 3) || (stockFilter === "out" && p.stock === 0)).map(p => (
                  <tr key={p.id} style={{ borderBottom: `1px solid ${C.borderLight}` }}>
                    <td style={{ padding: "12px" }}><div style={{ fontWeight: 600, color: C.textDark }}>{p.title}</div><div style={{ fontSize: 11, color: C.textMuted }}>{p.author}</div></td>
                    <td style={{ padding: "12px", color: C.textMid }}>{p.cat}</td>
                    <td style={{ padding: "12px", fontWeight: 600 }}>${p.price.toFixed(2)}</td>
                    <td style={{ padding: "12px" }}>
                      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
                        <Btn small variant="outline" style={{ padding: "2px 8px", fontSize: 14, lineHeight: 1 }}>−</Btn>
                        <span style={{ fontWeight: 600, minWidth: 20, textAlign: "center" }}>{p.stock}</span>
                        <Btn small variant="outline" style={{ padding: "2px 8px", fontSize: 14, lineHeight: 1 }}>+</Btn>
                      </div>
                    </td>
                    <td style={{ padding: "12px" }}>
                      {p.stock > 3 && <Badge>In stock</Badge>}
                      {p.stock > 0 && p.stock <= 3 && <Badge color={C.goldBg} text={C.gold} border={C.goldBorder}>Low</Badge>}
                      {p.stock === 0 && <Badge color={C.redBg} text={C.red} border="#f0c0c0">Out</Badge>}
                    </td>
                    <td style={{ padding: "12px" }}><Btn small variant="ghost">Edit</Btn></td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <Divider />
          <div style={{ fontSize: 13, color: C.textMuted }}>Showing {PRODUCTS.length} of {PRODUCTS.length} products</div>
        </Card>
      </>}
    </main>
  );
}

// ═══════════════════════════════════════════════════
// PAGE: Intake (Admin)
// ═══════════════════════════════════════════════════
function IntakePage({ go }) {
  const [step, setStep] = useState(1);
  const [isbn, setIsbn] = useState("");
  const [fetched, setFetched] = useState(false);

  return (
    <main style={{ maxWidth: 800, margin: "0 auto", padding: "0 24px 40px" }}>
      <div style={{ display: "flex", alignItems: "flex-start", justifyContent: "space-between", padding: "32px 0 20px" }}>
        <div>
          <h1 style={{ fontFamily: serif, fontSize: 26, fontWeight: 700, color: C.textDark, marginBottom: 6 }}>Add New Product</h1>
          <p style={{ fontSize: 13, color: C.textMid }}>Scan or type an ISBN, review the metadata, then save a shelf-ready product record.</p>
        </div>
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          {[{ n: 1, l: "Scan" }, { n: 2, l: "Review" }, { n: 3, l: "Save" }].map((s, i) => (<>
            <div key={s.n} style={{ display: "flex", alignItems: "center", gap: 5 }}>
              <div style={{ width: 24, height: 24, borderRadius: 12, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 11, fontWeight: 700, background: step >= s.n ? C.accent : C.borderLight, color: step >= s.n ? "#fff" : C.textMuted }}>{s.n}</div>
              <span style={{ fontSize: 13, fontWeight: step === s.n ? 600 : 400, color: step === s.n ? C.textDark : C.textMuted }}>{s.l}</span>
            </div>
            {i < 2 && <div style={{ width: 30, height: 1, background: C.border }} />}
          </>))}
        </div>
      </div>

      {step === 1 && <Card>
        <div style={{ fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 20 }}>ISBN & Cover</div>
        <div style={{ display: "grid", gridTemplateColumns: "240px 1fr", gap: 24 }}>
          <div style={{ height: 300, background: C.dark, borderRadius: 10, display: "flex", alignItems: "center", justifyContent: "center", position: "relative" }}>
            <div style={{ border: "2px dashed rgba(245,241,234,0.3)", borderRadius: 8, width: "70%", height: "60%", display: "flex", alignItems: "center", justifyContent: "center" }}>
              <div style={{ width: "50%", height: 1, background: C.gold }} />
            </div>
            <div style={{ position: "absolute", bottom: 12, fontSize: 12, color: "rgba(245,241,234,0.5)" }}>Hold barcode steady</div>
          </div>
          <div>
            <div style={{ fontSize: 13, fontWeight: 600, color: C.textDark, marginBottom: 8 }}>ISBN</div>
            <div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
              <input placeholder="978…" value={isbn} onChange={e => setIsbn(e.target.value)} style={{ flex: 1, padding: "10px 14px", fontSize: 14, fontFamily: mono, border: `1px solid ${C.border}`, borderRadius: 8, background: C.cream }} />
              <Btn variant="primary" onClick={() => { setFetched(true); setIsbn("9780060652937"); }}>Fetch</Btn>
            </div>
            <div style={{ display: "flex", gap: 8, marginBottom: 16 }}>
              <Btn variant="gold">Scanning…</Btn>
              <Btn variant="outline">Stop</Btn>
            </div>
            <p style={{ fontSize: 13, color: C.textMuted, marginBottom: 16 }}>Scanner live. Hold the ISBN barcode steady in frame.</p>
            <div style={{ padding: "10px 14px", background: C.goldBg, borderRadius: 8, border: `1px solid ${C.goldBorder}`, fontSize: 13, color: C.gold, marginBottom: 10 }}>Signed in. You can fetch metadata and save a product.</div>
            {fetched && <div style={{ padding: "10px 14px", background: C.greenBg, borderRadius: 8, border: `1px solid ${C.greenBorder}`, fontSize: 13, color: C.green }}>Metadata found. Proceed to review.</div>}
            {fetched && <div style={{ marginTop: 16 }}><Btn variant="primary" onClick={() => setStep(2)}>Continue to review</Btn></div>}
          </div>
        </div>
      </Card>}

      {step === 2 && <Card>
        <div style={{ fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 20 }}>Review metadata</div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16, marginBottom: 16 }}>
          <Input label="Title" placeholder="Book title" value="Knowing God" />
          <Input label="Author" placeholder="Author" value="J.I. Packer" />
          <Input label="Publisher" placeholder="Publisher" value="IVP Books" />
          <Input label="Category" placeholder="Category" value="Books" />
        </div>
        <Input label="Description" placeholder="Short description…" style={{ marginBottom: 16 }} />
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr 1fr", gap: 16, marginBottom: 24 }}>
          <Input label="Cost" placeholder="$0.00" value="12.50" />
          <Input label="Retail price" placeholder="$0.00" value="20.99" />
          <Input label="Initial stock" placeholder="0" value="10" />
          <Input label="Reorder point" placeholder="3" value="3" />
        </div>
        <div style={{ display: "flex", gap: 10 }}>
          <Btn variant="outline" onClick={() => setStep(1)}>Back</Btn>
          <Btn variant="primary" onClick={() => setStep(3)}>Save product</Btn>
        </div>
      </Card>}

      {step === 3 && <Card style={{ background: C.greenBg, border: `1px solid ${C.greenBorder}` }}>
        <div style={{ fontSize: 22, fontWeight: 700, fontFamily: serif, color: C.green, marginBottom: 8 }}>Product saved!</div>
        <div style={{ fontSize: 14, color: C.green, marginBottom: 16 }}>Knowing God has been added to the catalog with 10 units in stock.</div>
        <div style={{ display: "flex", gap: 10 }}>
          <Btn variant="dark" onClick={() => { setStep(1); setFetched(false); setIsbn(""); }}>Add another</Btn>
          <Btn variant="outline" onClick={() => go("orders")}>View inventory</Btn>
        </div>
      </Card>}

      <div style={{ marginTop: 20, padding: "16px 24px", borderLeft: `3px solid ${C.gold}`, background: C.cream, borderRadius: "0 8px 8px 0" }}>
        <div style={{ fontSize: 13, fontWeight: 700, color: C.gold, marginBottom: 4 }}>Volunteer flow</div>
        <div style={{ fontSize: 13, color: C.textMid, lineHeight: 1.6 }}>Start the scanner and hold the book barcode in frame. The ISBN will auto-fill, then press <strong>Fetch</strong> to pull metadata. Confirm the details, optionally upload a cover, and hit <strong>Save Product</strong>.</div>
      </div>
    </main>
  );
}

// ═══════════════════════════════════════════════════
// PAGE: POS PIN (POS zone)
// ═══════════════════════════════════════════════════
function POSPinPage({ go }) {
  const [pin, setPin] = useState("");
  const addDigit = d => { if (pin.length < 4) { const p = pin + d; setPin(p); if (p.length === 4) setTimeout(() => go("pos-sell"), 400); } };

  return (
    <div style={{ minHeight: "100vh", background: `linear-gradient(160deg, ${C.wine} 0%, #2a0a14 100%)`, display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", fontFamily: sans }}>
      <div style={{ fontSize: 20, color: C.gold, marginBottom: 4 }}>✦</div>
      <div style={{ fontFamily: serif, fontSize: 28, fontWeight: 700, color: C.gold, letterSpacing: 2, marginBottom: 4 }}>SCRIPTORIUM</div>
      <div style={{ fontSize: 13, color: "rgba(245,241,234,0.5)", marginBottom: 36 }}>Point of Sale</div>
      <div style={{ background: "rgba(180,140,160,0.15)", border: "1px solid rgba(200,170,185,0.2)", borderRadius: 16, padding: "28px 32px 24px", width: 300 }}>
        <div style={{ display: "flex", justifyContent: "center", gap: 12, marginBottom: 24 }}>
          {[0, 1, 2, 3].map(i => (
            <div key={i} style={{ width: 14, height: 14, borderRadius: 7, border: `2px solid ${C.gold}`, background: i < pin.length ? C.gold : "transparent", transition: "background 0.15s" }} />
          ))}
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: 8 }}>
          {[1, 2, 3, 4, 5, 6, 7, 8, 9, null, 0, "del"].map((d, i) => (
            d === null ? <div key={i} /> :
            <button key={i} onClick={() => d === "del" ? setPin(pin.slice(0, -1)) : addDigit(String(d))} style={{
              height: 56, borderRadius: 10, fontSize: d === "del" ? 16 : 22, fontWeight: 600, fontFamily: sans,
              background: C.posBtn, border: `1px solid ${C.posBtnBorder}`, color: C.cream, cursor: "pointer",
            }}>{d === "del" ? "⌫" : d}</button>
          ))}
        </div>
      </div>
      <div style={{ marginTop: 24, padding: "14px 24px", background: "rgba(245,241,234,0.08)", borderRadius: 10, maxWidth: 300, width: "100%" }}>
        <div style={{ fontSize: 13, fontWeight: 700, color: C.gold, marginBottom: 4 }}>Shift not started</div>
        <div style={{ fontSize: 13, color: "rgba(245,241,234,0.55)" }}>Enter the four-digit PIN to open the parish till.</div>
      </div>
      <div style={{ marginTop: 20, display: "flex", gap: 12 }}>
        <Btn variant="outline" small style={{ borderColor: "rgba(245,241,234,0.2)", color: "rgba(245,241,234,0.5)" }}>Forgot PIN?</Btn>
        <button onClick={() => go("admin-signin")} style={{ fontSize: 12, fontFamily: sans, fontWeight: 500, color: "rgba(245,241,234,0.4)", background: "none", border: "none", cursor: "pointer" }}>Admin login</button>
      </div>
    </div>
  );
}

// ═══════════════════════════════════════════════════
// PAGE: POS Selling (POS zone)
// ═══════════════════════════════════════════════════
function POSSellPage({ go }) {
  const [mode, setMode] = useState("scan");
  const [discount, setDiscount] = useState("none");

  return (
    <div style={{ minHeight: "100vh", background: `linear-gradient(160deg, ${C.wine} 0%, #2a0a14 100%)`, fontFamily: sans, display: "flex", flexDirection: "column" }}>
      {/* POS Header */}
      <div style={{ padding: "16px 20px", display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <div style={{ fontSize: 12, color: "rgba(245,241,234,0.4)" }}>✝</div>
          <div style={{ fontFamily: serif, fontWeight: 700, fontSize: 18, color: C.gold, letterSpacing: 1 }}>SCRIPTORIUM</div>
          <div style={{ fontSize: 10, letterSpacing: 1.5, color: "rgba(245,241,234,0.4)", textTransform: "uppercase" }}>Point of sale</div>
        </div>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 6 }}>
          <Badge color="rgba(245,241,234,0.1)" text="rgba(245,241,234,0.6)" border="rgba(245,241,234,0.15)">Shift pos-1</Badge>
          <Badge color="rgba(245,241,234,0.1)" text="rgba(245,241,234,0.6)" border="rgba(245,241,234,0.15)">Awaiting first item</Badge>
        </div>
      </div>

      <div style={{ flex: 1, padding: "0 16px 16px", maxWidth: 480, margin: "0 auto", width: "100%" }}>
        {/* Scan / Quick toggle */}
        <div style={{ display: "flex", background: "rgba(180,140,160,0.15)", borderRadius: 10, padding: 3, marginBottom: 12 }}>
          {["scan", "quick"].map(m => (
            <button key={m} onClick={() => setMode(m)} style={{
              flex: 1, padding: "10px", fontSize: 13, fontWeight: mode === m ? 600 : 400, fontFamily: sans,
              color: mode === m ? C.dark : "rgba(245,241,234,0.6)", background: mode === m ? C.cream : "transparent",
              border: "none", borderRadius: 8, cursor: "pointer",
            }}>{m === "scan" ? "Scan Item" : "Quick Items"}</button>
          ))}
        </div>

        {mode === "scan" && <>
          {/* Camera viewport */}
          <div style={{ background: C.dark, borderRadius: 12, height: 160, display: "flex", alignItems: "center", justifyContent: "center", marginBottom: 12, position: "relative" }}>
            <div style={{ border: "2px dashed rgba(245,241,234,0.2)", borderRadius: 8, width: "70%", height: "65%", display: "flex", alignItems: "center", justifyContent: "center" }}>
              <div style={{ width: "50%", height: 1, background: C.gold }} />
            </div>
            <div style={{ position: "absolute", bottom: 8, fontSize: 11, color: "rgba(245,241,234,0.4)" }}>Point camera at ISBN, EAN-13, or typed barcode</div>
          </div>
          <div style={{ fontSize: 12, fontWeight: 600, color: C.cream, marginBottom: 6 }}>ISBN / barcode</div>
          <input placeholder="9780060652937" style={{ width: "100%", padding: "10px 14px", fontSize: 14, fontFamily: mono, border: `1px solid rgba(245,241,234,0.2)`, borderRadius: 8, background: "rgba(245,241,234,0.08)", color: C.cream, marginBottom: 10 }} />
          <button style={{ width: "100%", padding: "12px", borderRadius: 10, background: C.gold, border: "none", color: "#fff", fontSize: 14, fontWeight: 600, fontFamily: sans, cursor: "pointer", marginBottom: 8 }}>Scan to cart</button>
          <div style={{ fontSize: 11, color: "rgba(245,241,234,0.4)", textAlign: "center", marginBottom: 16 }}>Use the camera lane or type the barcode when labels are faint.</div>
        </>}

        {mode === "quick" && <div style={{ display: "grid", gridTemplateColumns: "repeat(2, 1fr)", gap: 8, marginBottom: 16 }}>
          {PRODUCTS.slice(0, 6).map(p => (
            <button key={p.id} style={{ padding: "14px 12px", borderRadius: 10, background: "rgba(245,241,234,0.08)", border: "1px solid rgba(245,241,234,0.15)", cursor: "pointer", textAlign: "left" }}>
              <div style={{ fontSize: 13, fontWeight: 600, color: C.cream }}>{p.title}</div>
              <div style={{ fontSize: 11, color: "rgba(245,241,234,0.5)" }}>${p.price.toFixed(2)}</div>
            </button>
          ))}
        </div>}

        {/* Basket */}
        <div style={{ background: C.cream, borderRadius: 12, padding: "16px 20px", marginBottom: 12 }}>
          <div style={{ fontSize: 16, fontWeight: 700, fontFamily: serif, color: C.textDark, marginBottom: 12 }}>Basket</div>
          <div style={{ padding: "12px 16px", background: C.bg, borderRadius: 8, fontSize: 13, color: C.textMuted, textAlign: "center", marginBottom: 12 }}>Cart empty. Scan an item or use a quick tile to start the sale.</div>
          <div style={{ display: "flex", flexDirection: "column", gap: 6, marginBottom: 14 }}>
            {[["Current total", "$0.00"], ["Amount due", "$0.00"], ["Checkout path", "Card, cash, or IOU"]].map(([l, v]) => (
              <div key={l} style={{ display: "flex", justifyContent: "space-between", fontSize: 13, padding: "4px 0" }}>
                <span style={{ color: C.textMid }}>{l}</span><span style={{ fontWeight: 600, color: C.textDark }}>{v}</span>
              </div>
            ))}
          </div>
          <div style={{ display: "flex", gap: 6, flexWrap: "wrap" }}>
            {[{ key: "none", label: "No discount" }, { key: "clergy", label: "10% Clergy" }, { key: "volunteer", label: "15% Volunteer" }, { key: "bulk", label: "20% Bulk" }].map(d => (
              <button key={d.key} onClick={() => setDiscount(d.key)} style={{
                padding: "6px 14px", fontSize: 12, fontWeight: discount === d.key ? 600 : 400, borderRadius: 20, cursor: "pointer", fontFamily: sans,
                background: discount === d.key ? C.accent : "transparent", color: discount === d.key ? "#fff" : C.textMid,
                border: `1px solid ${discount === d.key ? C.accent : C.border}`,
              }}>{d.label}</button>
            ))}
          </div>
        </div>

        {/* Session status */}
        <div style={{ padding: "12px 16px", background: "rgba(46,107,63,0.15)", borderRadius: 10, marginBottom: 12 }}>
          <div style={{ fontSize: 12, fontWeight: 700, color: "#5cb87a" }}>Shift started</div>
          <div style={{ fontSize: 12, color: "rgba(245,241,234,0.6)" }}>Session pos-1 is ready for scanning, baskets, and payment.</div>
        </div>

        {/* Bottom checkout bar */}
        <button style={{ width: "100%", padding: "14px", borderRadius: 12, background: C.accent, border: "none", color: "#fff", fontSize: 15, fontWeight: 700, fontFamily: sans, cursor: "pointer" }}>Checkout · $0.00</button>

        {/* Exit */}
        <div style={{ textAlign: "center", marginTop: 16 }}>
          <button onClick={() => go("pos")} style={{ fontSize: 12, fontFamily: sans, color: "rgba(245,241,234,0.35)", background: "none", border: "none", cursor: "pointer" }}>Lock register & end shift</button>
        </div>
      </div>
    </div>
  );
}

// ═══════════════════════════════════════════════════
// ROOT APP — routing + state
// ═══════════════════════════════════════════════════
export default function Scriptorium() {
  const [page, setPage] = useState("catalog");
  const [cart, setCart] = useState([{ id: 2, title: "Knowing God", author: "J.I. Packer", price: 20.99, qty: 1 }]);

  const go = (p) => setPage(p);
  const addToCart = (product, quantity = 1) => {
    setCart(prev => {
      const existing = prev.find(i => i.id === product.id);
      if (existing) return prev.map(i => i.id === product.id ? { ...i, qty: i.qty + quantity } : i);
      return [...prev, { ...product, qty: quantity }];
    });
  };
  const updateQty = (id, delta) => setCart(prev => prev.map(i => i.id === id ? { ...i, qty: Math.max(1, i.qty + delta) } : i));
  const removeItem = (id) => id === "all" ? setCart([]) : setCart(prev => prev.filter(i => i.id !== id));

  const zone = ["catalog", "cart", "checkout"].includes(page) || page.startsWith("product-") ? "storefront"
    : ["dashboard", "orders", "intake"].includes(page) ? "admin"
    : ["pos", "pos-sell"].includes(page) ? "pos"
    : "gateway";

  return (
    <div style={{ minHeight: "100vh", background: zone === "pos" ? "transparent" : C.bg, fontFamily: sans }}>
      <style>{`
        @import url('https://fonts.googleapis.com/css2?family=Source+Serif+4:opsz,wght@8..60,400;8..60,600;8..60,700&family=Source+Sans+3:wght@400;500;600&family=JetBrains+Mono:wght@400;500&display=swap');
        * { margin: 0; padding: 0; box-sizing: border-box; }
        input::placeholder { color: #b0a898; }
        input:focus { outline: none; border-color: ${C.accent} !important; box-shadow: 0 0 0 3px rgba(107,28,42,0.08); }
        table { font-family: ${sans}; }
      `}</style>

      {zone === "storefront" && <StorefrontNav page={page} go={go} cartCount={cart.reduce((s, i) => s + i.qty, 0)} />}
      {zone === "admin" && <AdminNav page={page} go={go} />}
      {zone === "gateway" && <GatewayNav go={go} />}

      {page === "catalog" && <CatalogPage go={go} addToCart={addToCart} />}
      {page.startsWith("product-") && <ProductDetailPage productId={parseInt(page.split("-")[1])} go={go} addToCart={addToCart} />}
      {page === "cart" && <CartPage cart={cart} updateQty={updateQty} removeItem={removeItem} go={go} />}
      {page === "checkout" && <CheckoutPage cart={cart} go={go} />}
      {page === "admin-signin" && <AdminSignInPage go={go} />}
      {page === "dashboard" && <DashboardPage go={go} />}
      {page === "orders" && <OrdersPage go={go} />}
      {page === "intake" && <IntakePage go={go} />}
      {page === "pos" && <POSPinPage go={go} />}
      {page === "pos-sell" && <POSSellPage go={go} />}

      {zone !== "pos" && zone !== "gateway" && <Footer zone={zone} go={go} />}
    </div>
  );
}
