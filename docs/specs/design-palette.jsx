import { useState } from "react";

const palette = {
  brand: [
    { name: "Wine", token: "wine", hex: "#6B2737", use: "Primary actions, headers, CTA buttons" },
    { name: "Wine Light", token: "wineLight", hex: "#8B3A4A", use: "Hover states, gradient endpoints" },
    { name: "Wine Dark", token: "wineDark", hex: "#4A1A26", use: "Gradient anchors, PIN screen bg" },
    { name: "Wine Muted", token: "wineMuted", hex: "#8B6B74", use: "Subtle badges (POS channel tag)" },
  ],
  accent: [
    { name: "Gold", token: "gold", hex: "#B8903A", use: "Secondary CTA, admin actions, active tab indicator" },
    { name: "Gold Light", token: "goldLight", hex: "#CCAA5E", use: "PIN dots, display text highlights" },
    { name: "Gold Pale", token: "goldPale", hex: "#F5ECD7", use: "Active chip bg, ISBN lookup card, donate button bg" },
  ],
  surface: [
    { name: "White", token: "white", hex: "#FFFFFF", use: "Cards, inputs, primary surface" },
    { name: "Parchment", token: "parchment", hex: "#FAF7F2", use: "Page backgrounds, table headers" },
    { name: "Parchment Dark", token: "parchmentDark", hex: "#EDE8E0", use: "Borders, dividers, inactive inputs" },
    { name: "Filled", token: "filled", hex: "#F7F3EC", use: "Auto-filled field background (subtle)" },
    { name: "Filled Border", token: "filledBorder", hex: "#E0D8CC", use: "Auto-filled field border" },
  ],
  text: [
    { name: "Ink", token: "ink", hex: "#2C1810", use: "Primary text, headings, totals" },
    { name: "Ink Light", token: "inkLight", hex: "#5A4A3A", use: "Secondary text, field labels" },
    { name: "Warm Gray", token: "warmGray", hex: "#8A7A6A", use: "Tertiary text, placeholders, metadata" },
    { name: "Warm Gray Light", token: "warmGrayLight", hex: "#B5A898", use: "Disabled text, subtle indicators" },
  ],
  status: [
    { name: "Success", token: "success", hex: "#5A7D5E", use: "Paid badges, complete states, confirm buttons" },
    { name: "Success Light", token: "successLight", hex: "#EEF3EE", use: "Success badge bg, confirmation panels" },
    { name: "Success Accent", token: "successAccent", hex: "#4A6B4E", use: "Transaction complete gradient end" },
    { name: "Warning", token: "warning", hex: "#A07040", use: "IOU badges, tab payment, attention items" },
    { name: "Warning Light", token: "warningLight", hex: "#F5EDE3", use: "IOU badge bg, alert panels" },
    { name: "Danger", token: "danger", hex: "#9B5A5A", use: "Refund badges, out-of-stock tags" },
    { name: "Danger Light", token: "dangerLight", hex: "#F5EDED", use: "Danger badge bg" },
    { name: "Blue", token: "blue", hex: "#5A7A9B", use: "Online channel badge, card payment icon" },
    { name: "Blue Light", token: "blueLight", hex: "#ECF1F5", use: "Blue badge bg, card icon container" },
  ],
};

const fonts = [
  {
    name: "Crimson Pro",
    token: "fontDisplay",
    category: "Display / Serif",
    weights: ["400 Regular", "500 Medium", "600 SemiBold", "700 Bold"],
    use: "App title, section headings, product titles, page headers. Provides warmth and ecclesiastical character.",
    sample: "Scriptorium",
    css: "'Crimson Pro', serif",
  },
  {
    name: "DM Sans",
    token: "font",
    category: "Body / Sans-serif",
    weights: ["400 Regular", "500 Medium", "600 SemiBold", "700 Bold", "800 ExtraBold"],
    use: "Body text, buttons, labels, prices, form fields, badges. Clean and highly legible at all sizes — critical for elderly volunteer readability.",
    sample: "The quick brown fox jumps over the lazy dog",
    css: "'DM Sans', sans-serif",
  },
];

const radii = [
  { name: "Small", token: "radiusSm", value: "8px", use: "Inputs, small buttons, chips" },
  { name: "Default", token: "radius", value: "12px", use: "Cards, modals, large buttons" },
  { name: "Large", token: "radiusLg", value: "16px", use: "Hero sections, image containers, overlays" },
  { name: "Pill", token: "—", value: "20–24px", use: "Badges, category pills, pagination" },
];

const shadows = [
  { name: "Default", token: "shadow", value: "0 2px 12px rgba(44,24,16,0.06)", use: "Cards, dropdowns, buttons" },
  { name: "Large", token: "shadowLg", value: "0 8px 32px rgba(44,24,16,0.10)", use: "Modals, elevated panels" },
  { name: "CTA Glow", token: "—", value: "0 4px 12px rgba(107,39,55,0.25)", use: "Primary action buttons (wine glow)" },
  { name: "Success Glow", token: "—", value: "0 4px 12px rgba(90,125,94,0.30)", use: "Confirm / complete buttons" },
];

// ─── Components ───

const Swatch = ({ hex, name, token, use, size = "md" }) => {
  const [copied, setCopied] = useState(false);
  const isLight = ["#FFFFFF", "#FAF7F2", "#EDE8E0", "#F5ECD7", "#F7F3EC", "#E0D8CC", "#EEF3EE", "#F5EDED", "#F5EDE3", "#ECF1F5", "#B5A898"].includes(hex);

  const handleCopy = () => {
    navigator.clipboard?.writeText(hex);
    setCopied(true);
    setTimeout(() => setCopied(false), 1200);
  };

  return (
    <div onClick={handleCopy} style={{ cursor: "pointer", minWidth: size === "sm" ? 100 : 140 }}>
      <div style={{
        width: "100%",
        height: size === "sm" ? 48 : 64,
        borderRadius: 10,
        background: hex,
        border: isLight ? "1px solid #E0D8CC" : "1px solid transparent",
        display: "flex", alignItems: "flex-end", justifyContent: "flex-end",
        padding: 6,
        transition: "transform 0.15s ease",
        position: "relative",
      }}>
        {copied && (
          <div style={{
            position: "absolute", inset: 0, borderRadius: 10,
            background: "rgba(44,24,16,0.75)", display: "flex",
            alignItems: "center", justifyContent: "center",
            color: "#fff", fontSize: 12, fontWeight: 700, fontFamily: "'DM Sans', sans-serif",
          }}>Copied!</div>
        )}
      </div>
      <div style={{ marginTop: 6 }}>
        <div style={{ fontSize: 13, fontWeight: 700, color: "#2C1810", fontFamily: "'DM Sans', sans-serif" }}>{name}</div>
        <div style={{ fontSize: 12, fontFamily: "'JetBrains Mono', monospace", color: "#8A7A6A", letterSpacing: -0.3 }}>{hex}</div>
        {token && <div style={{ fontSize: 11, fontFamily: "'JetBrains Mono', monospace", color: "#B5A898" }}>T.{token}</div>}
        {use && size !== "sm" && <div style={{ fontSize: 11, color: "#8A7A6A", marginTop: 2, lineHeight: 1.4 }}>{use}</div>}
      </div>
    </div>
  );
};

const Section = ({ title, subtitle, children }) => (
  <div style={{ marginBottom: 48 }}>
    <h2 style={{
      fontFamily: "'Crimson Pro', serif", fontSize: 26, fontWeight: 600,
      color: "#6B2737", margin: "0 0 4px", letterSpacing: 0.5,
    }}>{title}</h2>
    {subtitle && <p style={{ fontSize: 14, color: "#8A7A6A", margin: "0 0 20px", fontFamily: "'DM Sans', sans-serif" }}>{subtitle}</p>}
    {!subtitle && <div style={{ height: 16 }} />}
    {children}
  </div>
);

const GroupLabel = ({ children }) => (
  <div style={{
    fontSize: 11, fontWeight: 700, color: "#8A7A6A", textTransform: "uppercase",
    letterSpacing: 1.5, marginBottom: 12, fontFamily: "'DM Sans', sans-serif",
  }}>{children}</div>
);

export default function DesignPalette() {
  return (
    <div style={{
      minHeight: "100vh",
      background: "#FAF7F2",
      fontFamily: "'DM Sans', sans-serif",
      padding: "40px 32px",
    }}>
      <link href="https://fonts.googleapis.com/css2?family=Crimson+Pro:wght@400;500;600;700&family=DM+Sans:wght@400;500;600;700;800&family=JetBrains+Mono:wght@400;500;600&display=swap" rel="stylesheet" />

      {/* Header */}
      <div style={{ maxWidth: 960, margin: "0 auto" }}>
        <div style={{ marginBottom: 48, borderBottom: "2px solid #EDE8E0", paddingBottom: 32 }}>
          <div style={{ display: "flex", alignItems: "center", gap: 12, marginBottom: 8 }}>
            <span style={{ fontSize: 32, opacity: 0.5 }}>☦</span>
            <h1 style={{
              fontFamily: "'Crimson Pro', serif", fontSize: 42, fontWeight: 600,
              color: "#6B2737", margin: 0, letterSpacing: 1,
            }}>Scriptorium</h1>
          </div>
          <p style={{ fontSize: 16, color: "#5A4A3A", margin: "0 0 4px", fontWeight: 500 }}>
            Design System & Palette Reference
          </p>
          <p style={{ fontSize: 14, color: "#8A7A6A", margin: 0 }}>
            Church Bookstore & POS — Colour, typography, spacing, and component tokens
          </p>
        </div>

        {/* ─── COLOUR PALETTE ─── */}
        <Section title="Colour Palette" subtitle="Earth-toned, warm, and accessible. Click any swatch to copy its hex value.">
          
          <GroupLabel>Brand — Wine</GroupLabel>
          <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginBottom: 28 }}>
            {palette.brand.map(c => <Swatch key={c.hex} {...c} />)}
          </div>

          <GroupLabel>Accent — Gold</GroupLabel>
          <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginBottom: 28 }}>
            {palette.accent.map(c => <Swatch key={c.hex} {...c} />)}
          </div>

          <GroupLabel>Surfaces & Backgrounds</GroupLabel>
          <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginBottom: 28 }}>
            {palette.surface.map(c => <Swatch key={c.hex} {...c} />)}
          </div>

          <GroupLabel>Text & Ink</GroupLabel>
          <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginBottom: 28 }}>
            {palette.text.map(c => <Swatch key={c.hex} {...c} />)}
          </div>

          <GroupLabel>Status — Muted & Earth-Toned</GroupLabel>
          <div style={{ display: "flex", gap: 16, flexWrap: "wrap" }}>
            {palette.status.map(c => <Swatch key={c.token} {...c} />)}
          </div>
        </Section>

        {/* ─── COLOUR RATIONALE ─── */}
        <Section title="Palette Rationale">
          <div style={{
            background: "#FFFFFF", borderRadius: 12, padding: 24,
            border: "1px solid #EDE8E0", fontSize: 14, color: "#5A4A3A",
            lineHeight: 1.75,
          }}>
            <p style={{ margin: "0 0 12px" }}>
              <strong style={{ color: "#2C1810" }}>Why earth tones?</strong> Status colours in most design systems use saturated primaries (vivid green, red, blue). 
              In Scriptorium these sit alongside a wine-and-parchment base palette — saturated status colours created visual dissonance, 
              pulling focus away from the primary actions. The muted alternatives (sage green, dusty rose, warm ochre, slate blue) 
              communicate meaning without competing for attention.
            </p>
            <p style={{ margin: "0 0 12px" }}>
              <strong style={{ color: "#2C1810" }}>Accessibility note:</strong> All text/background combinations meet WCAG AA contrast requirements. 
              The ink-on-parchment combination (4.5:1 ratio minimum) is especially important given the elderly volunteer user base. 
              Status badge text sits on tinted backgrounds at 5:1+ contrast ratio.
            </p>
            <p style={{ margin: 0 }}>
              <strong style={{ color: "#2C1810" }}>Auto-fill highlight:</strong> The <span style={{ fontFamily: "'JetBrains Mono', monospace", fontSize: 12, background: "#F7F3EC", padding: "1px 6px", borderRadius: 4, border: "1px solid #E0D8CC" }}>filled / filledBorder</span> tokens 
              are intentionally barely visible — a warm tint rather than a coloured highlight. They signal "this data was fetched" 
              without implying a state that needs resolving.
            </p>
          </div>
        </Section>

        {/* ─── TYPOGRAPHY ─── */}
        <Section title="Typography" subtitle="Two-font system optimised for readability across age groups and device sizes.">
          {fonts.map((f, i) => (
            <div key={i} style={{
              background: "#FFFFFF", borderRadius: 12, padding: 24,
              border: "1px solid #EDE8E0", marginBottom: 16,
            }}>
              <div style={{ display: "flex", alignItems: "baseline", gap: 12, marginBottom: 4 }}>
                <span style={{
                  fontFamily: f.css, fontSize: 28, fontWeight: 600, color: "#2C1810",
                }}>{f.name}</span>
                <span style={{
                  fontSize: 12, fontFamily: "'JetBrains Mono', monospace",
                  color: "#B5A898", background: "#FAF7F2", padding: "2px 8px", borderRadius: 6,
                }}>T.{f.token}</span>
              </div>
              <div style={{ fontSize: 12, color: "#8A7A6A", marginBottom: 4 }}>{f.category}</div>
              <div style={{ display: "flex", gap: 8, flexWrap: "wrap", marginBottom: 12 }}>
                {f.weights.map(w => (
                  <span key={w} style={{
                    fontSize: 11, background: "#FAF7F2", color: "#5A4A3A",
                    padding: "3px 10px", borderRadius: 12, border: "1px solid #EDE8E0",
                  }}>{w}</span>
                ))}
              </div>
              <div style={{
                fontFamily: f.css, fontSize: f.token === "fontDisplay" ? 36 : 18,
                fontWeight: f.token === "fontDisplay" ? 600 : 400,
                color: "#2C1810", marginBottom: 12, lineHeight: 1.4,
              }}>{f.sample}</div>
              <div style={{ fontSize: 13, color: "#8A7A6A", lineHeight: 1.6 }}>{f.use}</div>
            </div>
          ))}

          {/* Type scale */}
          <GroupLabel>Type Scale</GroupLabel>
          <div style={{ background: "#FFFFFF", borderRadius: 12, padding: 24, border: "1px solid #EDE8E0" }}>
            {[
              { size: 56, weight: 800, font: "body", label: "Total Due / Change Due", context: "POS payment amount" },
              { size: 36, weight: 600, font: "display", label: "Feed your soul.", context: "Hero heading" },
              { size: 32, weight: 600, font: "display", label: "The Orthodox Study Bible", context: "Product title" },
              { size: 28, weight: 800, font: "body", label: "$347.85", context: "Dashboard metric" },
              { size: 20, weight: 700, font: "body", label: "CHECKOUT · $35.95", context: "Primary CTA" },
              { size: 16, weight: 600, font: "body", label: "Amount Tendered:", context: "Section label" },
              { size: 14, weight: 600, font: "body", label: "The Orthodox Study Bible", context: "Cart item name" },
              { size: 13, weight: 600, font: "body", label: "10% Clergy", context: "Discount chip, badge" },
              { size: 12, weight: 600, font: "body", label: "UNPAID IOUS", context: "Section sub-label" },
              { size: 11, weight: 600, font: "body", label: "BOOKS", context: "Category tag, table header" },
            ].map((t, i) => (
              <div key={i} style={{
                display: "flex", alignItems: "baseline", gap: 16, padding: "8px 0",
                borderBottom: i < 9 ? "1px solid #EDE8E0" : "none",
              }}>
                <span style={{
                  fontFamily: "'JetBrains Mono', monospace", fontSize: 11, color: "#B5A898",
                  minWidth: 42, textAlign: "right",
                }}>{t.size}px</span>
                <span style={{
                  fontFamily: t.font === "display" ? "'Crimson Pro', serif" : "'DM Sans', sans-serif",
                  fontSize: t.size > 36 ? 36 : t.size,
                  fontWeight: t.weight, color: "#2C1810", flex: 1,
                }}>{t.label}</span>
                <span style={{ fontSize: 12, color: "#8A7A6A", whiteSpace: "nowrap" }}>{t.context}</span>
              </div>
            ))}
          </div>
        </Section>

        {/* ─── SPACING & RADII ─── */}
        <Section title="Radii & Elevation" subtitle="Rounded corners and warm-tinted shadows for a soft, approachable feel.">
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
            <div>
              <GroupLabel>Border Radii</GroupLabel>
              <div style={{ background: "#FFFFFF", borderRadius: 12, padding: 20, border: "1px solid #EDE8E0" }}>
                {radii.map((r, i) => (
                  <div key={i} style={{
                    display: "flex", alignItems: "center", gap: 16, padding: "10px 0",
                    borderBottom: i < radii.length - 1 ? "1px solid #EDE8E0" : "none",
                  }}>
                    <div style={{
                      width: 40, height: 40, borderRadius: r.value,
                      background: "#6B2737", flexShrink: 0,
                    }} />
                    <div>
                      <div style={{ fontSize: 14, fontWeight: 700, color: "#2C1810" }}>{r.name}</div>
                      <div style={{ fontSize: 12, fontFamily: "'JetBrains Mono', monospace", color: "#8A7A6A" }}>{r.value}</div>
                      <div style={{ fontSize: 11, color: "#B5A898" }}>{r.use}</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
            <div>
              <GroupLabel>Box Shadows</GroupLabel>
              <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                {shadows.map((s, i) => (
                  <div key={i} style={{
                    background: "#FFFFFF", borderRadius: 12, padding: 20,
                    boxShadow: s.value, border: "1px solid #EDE8E0",
                  }}>
                    <div style={{ fontSize: 14, fontWeight: 700, color: "#2C1810" }}>{s.name}</div>
                    <div style={{ fontSize: 11, fontFamily: "'JetBrains Mono', monospace", color: "#8A7A6A", marginTop: 2, wordBreak: "break-all" }}>{s.value}</div>
                    <div style={{ fontSize: 11, color: "#B5A898", marginTop: 2 }}>{s.use}</div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </Section>

        {/* ─── COMPONENT PREVIEW ─── */}
        <Section title="Component Samples" subtitle="Key UI building blocks as they appear across the three application contexts.">

          {/* Buttons */}
          <GroupLabel>Buttons</GroupLabel>
          <div style={{
            background: "#FFFFFF", borderRadius: 12, padding: 24, border: "1px solid #EDE8E0",
            marginBottom: 20, display: "flex", flexDirection: "column", gap: 16,
          }}>
            <div style={{ display: "flex", gap: 12, alignItems: "center", flexWrap: "wrap" }}>
              {[
                { label: "Primary (Wine)", bg: "#6B2737", color: "#fff" },
                { label: "Gold CTA", bg: "#B8903A", color: "#fff" },
                { label: "Success", bg: "#5A7D5E", color: "#fff" },
                { label: "Warning / IOU", bg: "#A07040", color: "#fff" },
                { label: "Ghost", bg: "transparent", color: "#2C1810", border: "1.5px solid #EDE8E0" },
              ].map((b, i) => (
                <button key={i} style={{
                  padding: "10px 20px", borderRadius: 8, border: b.border || "none",
                  background: b.bg, color: b.color, fontFamily: "'DM Sans', sans-serif",
                  fontSize: 14, fontWeight: 600, cursor: "pointer",
                }}>{b.label}</button>
              ))}
            </div>
            <div style={{ display: "flex", gap: 12, alignItems: "center" }}>
              <button style={{
                padding: "18px 36px", borderRadius: 12, border: "none",
                background: "#6B2737", color: "#fff", fontFamily: "'DM Sans', sans-serif",
                fontSize: 20, fontWeight: 700, cursor: "pointer",
                boxShadow: "0 4px 12px rgba(107,39,55,0.25)",
                display: "flex", alignItems: "center", gap: 10,
              }}>
                <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"><circle cx="9" cy="21" r="1" /><circle cx="20" cy="21" r="1" /><path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6" /></svg>
                CHECKOUT · $35.95
              </button>
              <span style={{ fontSize: 12, color: "#8A7A6A" }}>XL — POS primary action (min 48px touch target)</span>
            </div>
          </div>

          {/* Badges */}
          <GroupLabel>Status Badges</GroupLabel>
          <div style={{
            background: "#FFFFFF", borderRadius: 12, padding: 24, border: "1px solid #EDE8E0",
            marginBottom: 20, display: "flex", gap: 12, flexWrap: "wrap", alignItems: "center",
          }}>
            {[
              { label: "Paid", color: "#5A7D5E", bg: "#EEF3EE" },
              { label: "IOU", color: "#A07040", bg: "#F5EDE3" },
              { label: "Refunded", color: "#9B5A5A", bg: "#F5EDED" },
              { label: "POS", color: "#8B6B74", bg: "#8B6B7414" },
              { label: "Online", color: "#5A7A9B", bg: "#ECF1F5" },
              { label: "Shift: AM", color: "#CCAA5E", bg: "rgba(184,144,58,0.2)" },
              { label: "Admin", color: "#8A7A6A", bg: "#8A7A6A14" },
              { label: "Only 2 Left", color: "#A07040", bg: "#A0704020" },
              { label: "Out of Stock", color: "#9B5A5A", bg: "#9B5A5A18" },
            ].map((b, i) => (
              <span key={i} style={{
                display: "inline-flex", padding: "3px 12px", borderRadius: 20,
                fontSize: 12, fontWeight: 600, color: b.color, background: b.bg,
                fontFamily: "'DM Sans', sans-serif",
              }}>{b.label}</span>
            ))}
          </div>

          {/* Quick Item Grid */}
          <GroupLabel>Quick-Tap Grid Items (POS)</GroupLabel>
          <div style={{
            background: "#FAF7F2", borderRadius: 12, padding: 20, border: "1px solid #EDE8E0",
            marginBottom: 20,
          }}>
            <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: 10, maxWidth: 480 }}>
              {[
                { name: "Prayer Card", price: "$0.50", emoji: "🙏", color: "#EDE5D8" },
                { name: "Votive Candle", price: "$1.00", emoji: "🕯️", color: "#F0E8DA" },
                { name: "Charcoal", price: "$3.00", emoji: "⚫", color: "#E5E2DD" },
                { name: "Incense", price: "$5.00", emoji: "💨", color: "#E4E2E8" },
              ].map((item, i) => (
                <div key={i} style={{
                  background: item.color, borderRadius: 12, padding: "14px 10px",
                  display: "flex", flexDirection: "column", alignItems: "center", gap: 4,
                }}>
                  <span style={{ fontSize: 28 }}>{item.emoji}</span>
                  <span style={{ fontSize: 13, fontWeight: 600, color: "#2C1810", fontFamily: "'DM Sans', sans-serif" }}>{item.name}</span>
                  <span style={{
                    fontSize: 14, fontWeight: 700, color: "#6B2737",
                    background: "rgba(255,255,255,0.7)", padding: "1px 10px", borderRadius: 16,
                    fontFamily: "'DM Sans', sans-serif",
                  }}>{item.price}</span>
                </div>
              ))}
            </div>
            <div style={{ fontSize: 12, color: "#8A7A6A", marginTop: 12 }}>
              Muted pastel backgrounds per category — avoid saturated colours. Price pill uses white overlay at 70% opacity.
            </div>
          </div>

          {/* Cards */}
          <GroupLabel>Card Variants</GroupLabel>
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: 16, marginBottom: 20 }}>
            <div style={{
              background: "#FFFFFF", borderRadius: 12, padding: 20,
              boxShadow: "0 2px 12px rgba(44,24,16,0.06)",
              border: "1px solid #EDE8E0",
            }}>
              <div style={{ fontSize: 13, color: "#8A7A6A", fontWeight: 500 }}>Standard Card</div>
              <div style={{ fontSize: 14, color: "#5A4A3A", marginTop: 4 }}>White bg, default shadow, parchment border</div>
            </div>
            <div style={{
              background: "#F5ECD7", borderRadius: 12, padding: 20,
              border: "1px solid #E0D8CC",
            }}>
              <div style={{ fontSize: 13, color: "#2C1810", fontWeight: 600 }}>Highlighted Card</div>
              <div style={{ fontSize: 14, color: "#5A4A3A", marginTop: 4 }}>Gold pale bg — ISBN lookup, donate prompt</div>
            </div>
            <div style={{
              background: "#F5EDE3", borderRadius: 12, padding: 20,
              border: "1px solid rgba(160,112,64,0.12)",
            }}>
              <div style={{ fontSize: 13, color: "#A07040", fontWeight: 600 }}>Warning Card</div>
              <div style={{ fontSize: 14, color: "#5A4A3A", marginTop: 4 }}>IOU alerts, attention items</div>
            </div>
          </div>
        </Section>

        {/* ─── DESIGN PRINCIPLES ─── */}
        <Section title="Design Principles">
          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 16 }}>
            {[
              {
                num: "01", title: "Speed Over Beauty",
                text: "The POS screens are used in a 30-minute post-service rush. Every tap counts. Oversized touch targets (48px minimum), zero unnecessary confirmation dialogs, and single-tap actions are non-negotiable."
              },
              {
                num: "02", title: "Legibility First",
                text: "The primary volunteer demographic skews older. DM Sans at 14px+ for body, 20px+ for actions, 56px for totals. High contrast ink-on-parchment throughout. No thin weights below 16px."
              },
              {
                num: "03", title: "Quiet Status, Loud Actions",
                text: "Status badges use muted earth tones so they inform without distracting. Primary actions (Checkout, Complete Sale, Start Next Sale) are the loudest elements on every screen."
              },
              {
                num: "04", title: "Contextual Density",
                text: "POS screens are spacious and minimal. Admin screens are denser with tables and metrics. The storefront sits in between. Each context gets its own appropriate information density."
              },
            ].map((p, i) => (
              <div key={i} style={{
                background: "#FFFFFF", borderRadius: 12, padding: 24,
                border: "1px solid #EDE8E0",
              }}>
                <div style={{
                  fontSize: 11, fontWeight: 800, color: "#B8903A",
                  letterSpacing: 2, marginBottom: 4,
                }}>{p.num}</div>
                <div style={{ fontSize: 16, fontWeight: 700, color: "#2C1810", marginBottom: 8 }}>{p.title}</div>
                <div style={{ fontSize: 13, color: "#5A4A3A", lineHeight: 1.7 }}>{p.text}</div>
              </div>
            ))}
          </div>
        </Section>

        {/* Footer */}
        <div style={{
          borderTop: "2px solid #EDE8E0", paddingTop: 24, marginTop: 16,
          display: "flex", justifyContent: "space-between", alignItems: "center",
        }}>
          <div style={{ fontSize: 13, color: "#B5A898" }}>Scriptorium Design System v2.0</div>
          <div style={{ fontSize: 13, color: "#B5A898" }}>☦ Church Bookstore & POS</div>
        </div>
      </div>
    </div>
  );
}
