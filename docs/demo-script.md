# Scriptorium Demo Script

> **Format:** Screencast recording with voiceover narration
> **Total runtime:** ~8-10 minutes
> **App URL:** `http://localhost:8080` (run `cargo run` before recording)

---

## Section 1: Introduction & Problem Statement

**Duration:** 45 seconds

**Narration:**

> "Church bookstores have a unique problem. They operate in short, chaotic windows — typically 30 minutes after Sunday service — staffed by volunteers who may not be tech-savvy. Traditional POS systems are expensive, complex, and designed for full-time retail staff.
>
> Scriptorium is a unified bookstore platform built specifically for churches. It gives you a mobile point-of-sale that runs on any phone, a public online storefront, and an admin dashboard for financial visibility — all in one lightweight app."

**Screen instructions:**

- Show the Scriptorium logo / POS login screen at `/pos` as a static backdrop while narrating.
- No interaction needed — let the PIN screen sit with its wine-and-gold ecclesiastical design visible.

---

## Section 2: The Volunteer POS Experience

**Duration:** 2.5-3 minutes

This is the flagship feature. Spend the most time here.

### 2a. PIN Login (20 seconds)

**Narration:**

> "Volunteers start by entering their 4-digit PIN. No usernames, no passwords — just a quick PIN on a big numeric keypad designed for touchscreens."

**Screen instructions:**

1. Start at `/pos`.
2. Slowly tap `1`, `2`, `3`, `4` on the keypad — show each circle filling in.
3. POS main screen loads.

### 2b. Scanning Items (30 seconds)

**Narration:**

> "When a customer brings a book to the counter, the volunteer scans the barcode with their phone camera — or types the ISBN manually. The item appears in the basket instantly."

**Screen instructions:**

1. Show the Scan tab with the camera preview area.
2. Type an ISBN into the manual input field (e.g. `978-0310329060`).
3. Tap "Scan to cart."
4. Show the item appearing in the basket section below.

### 2c. Quick Items (30 seconds)

**Narration:**

> "Not everything has a barcode. Prayer cards, votive candles, incense — these are available as one-tap Quick Items. Big buttons with clear icons and prices. No typing required."

**Screen instructions:**

1. Tap the "Quick Items" tab.
2. Show the 2-column grid of emoji-labeled items.
3. Tap "Prayer Card" twice and "Votive Candle" once.
4. Show the basket updating with quantities and running total.

### 2d. Discounts (15 seconds)

**Narration:**

> "Clergy and volunteer discounts are built in — just tap the appropriate pill. No mental math, no manual percentage calculations."

**Screen instructions:**

1. Tap the "10% Clergy" discount pill.
2. Show the checkout total updating.

### 2e. Checkout — Cash Payment (45 seconds)

**Narration:**

> "At checkout, the volunteer picks a payment method. For cash, the system offers quick-tap amount buttons — exact change, twenty dollars, or a custom amount. It calculates change automatically.
>
> Here's a nice touch: if a customer says 'keep the change,' the volunteer taps 'Round-up / Donate Change' and the system records the overage as a parish donation — tracked separately for the treasurer."

**Screen instructions:**

1. Tap the "Checkout" button at the bottom.
2. Tap "Cash."
3. Show the total displayed (e.g. $2.40).
4. Tap the "$5.00" or "$20.00" quick button.
5. Show the change calculation appear.
6. Tap "Round-up / Donate Change."
7. Show the "Sale Complete" confirmation screen.

### 2f. Other Payment Methods (15 seconds)

**Narration:**

> "For card payments, Scriptorium hands off to your existing payment app — Square, Stripe Terminal, whatever you already use. And for regulars who forgot their wallet, there's an IOU option that records the debt for admin follow-up."

**Screen instructions:**

1. Briefly show (or mention) the Card and IOU payment options — no need to complete a full flow, just show the buttons exist.

---

## Section 3: The Online Storefront

**Duration:** 2-2.5 minutes

### 3a. Catalog Browsing (45 seconds)

**Narration:**

> "Scriptorium also gives your bookstore an online presence. Parishioners can browse the full catalog from home — filter by category, search by title or author, and see real-time stock levels."

**Screen instructions:**

1. Navigate to `/catalog`.
2. Slowly scroll through the product grid — show cover images, titles, prices.
3. Tap a category filter chip (e.g. "Icons").
4. Show the grid filtering.
5. Tap "All" to reset.
6. Type a search term (e.g. "Chesterton") into the search bar.
7. Show results filtering in real-time via HTMX.

### 3b. Product Detail (30 seconds)

**Narration:**

> "Each product page shows full metadata — pulled automatically from ISBN lookup when the item was added. Cover art, description, publisher details, and stock availability. The copy is parish-oriented, not generic retail."

**Screen instructions:**

1. Click on a product (e.g. "Orthodoxy" by G.K. Chesterton).
2. Show the product detail page: cover art, title, author, description.
3. Scroll down to show the details table (Publisher, ISBN, Pages).
4. Show the "Related Products" section at the bottom.

### 3c. Cart & Checkout (45 seconds)

**Narration:**

> "The checkout flow is a clean three-step process: contact details, payment, and confirmation. Customers can choose pickup or shipping, and there's an optional parish support donation — small dollar amounts that add up over time."

**Screen instructions:**

1. On the product detail page, tap "Add to Cart."
2. Navigate to the cart (`/cart`). Show the cart with the item and the recommendations sidebar.
3. Tap "Checkout."
4. On step 1: fill in a name, email. Toggle "Pickup" delivery.
5. Tap a donation pill (e.g. "$5").
6. Advance to step 2: show the card payment form.
7. Advance to step 3: show the order confirmation summary with subtotal, donation, and total.

---

## Section 4: Admin Dashboard & Financial Visibility

**Duration:** 1.5-2 minutes

### 4a. Sign In & Dashboard Overview (45 seconds)

**Narration:**

> "The admin dashboard is where treasurers and pastors get visibility into bookstore operations. At a glance: total sales, POS versus online revenue, and outstanding IOUs. Everything filterable by date range and exportable to CSV."

**Screen instructions:**

1. Navigate to `/admin`.
2. Sign in (username: `admin`, password: `admin123`).
3. Show the greeting ("Good morning, Father Michael").
4. Pan across the four metric cards: Total Sales, POS Revenue, Online Revenue, Open IOUs.
5. Show the payment breakdown section (Cash, Card, Online, IOU).
6. Click through the tabs: Treasurer, Sunday close, Pastoral.

### 4b. Order Management (30 seconds)

**Narration:**

> "The Orders page lets you search and filter every transaction. POS sales, online orders, and IOUs are all in one place. Unpaid tabs can be marked as collected with a single click."

**Screen instructions:**

1. Click "Orders" in the admin nav.
2. Show the order table with columns.
3. Click a filter pill (e.g. "IOU") to filter.
4. Click "View" on an order to show the inline detail panel.

### 4c. Product Intake (45 seconds)

**Narration:**

> "Adding new inventory is a three-step process: scan, review, save. Scan a barcode with your phone camera or type the ISBN. Scriptorium fetches the book's metadata automatically — title, author, cover, description. The admin just sets the price and quantity, then saves."

**Screen instructions:**

1. Click "Intake" in the admin nav.
2. Show the step indicator: Scan > Review > Save.
3. Type an ISBN into the input field.
4. Tap "Fetch" — show the fields auto-populating with metadata.
5. Scroll through the review form: title, author, category, pricing fields.
6. Show the cover image upload area.
7. Tap "Save product."

---

## Section 5: Design Philosophy & Closing

**Duration:** 45 seconds

**Narration:**

> "Every design decision in Scriptorium prioritizes the people who actually use it. Minimum 13-pixel fonts for readability. Large touch targets for older volunteers. An ecclesiastical color palette — wine and gold — that feels appropriate for a parish setting, not like generic retail software.
>
> Scriptorium runs as a single lightweight binary. No app store installs, no expensive POS hardware. Just open a browser on any phone and start selling.
>
> If you run a church bookstore, Scriptorium was built for you."

**Screen instructions:**

1. Quick montage: flip between the POS screen on a phone-sized viewport, the storefront on a tablet/desktop viewport, and the admin dashboard.
2. End on the POS PIN login screen with the decorative cross and "SCRIPTORIUM" title.
3. Fade out.

---

## Recording Notes

### Before Recording

- Run the app: `cargo run` (ensure port 8080 is available).
- Seed data loads automatically — 12 products across 4 categories.
- POS PIN: `1234`. Admin credentials: `admin` / `admin123`.
- Clear any previous session/cart state by restarting the server.

### Browser Setup

- Use a clean browser profile (no extensions visible).
- For POS sections, use mobile device emulation (e.g. iPhone 14 viewport in DevTools) to show the phone-optimized layout.
- For storefront and admin sections, use a standard desktop viewport.
- Hide the browser bookmark bar and any toolbars.

### Pacing

- Move deliberately — this is a product demo, not a speed run.
- Pause briefly after each action so viewers can register what happened.
- Keep mouse movements smooth and intentional.

### Audio

- Record voiceover separately for cleaner audio, then sync.
- Alternatively, narrate live but use a quiet environment.

| Section | Topic | Duration |
|---------|-------|----------|
| 1 | Introduction & Problem | 0:45 |
| 2 | Volunteer POS | 2:30-3:00 |
| 3 | Online Storefront | 2:00-2:30 |
| 4 | Admin Dashboard | 1:30-2:00 |
| 5 | Design & Closing | 0:45 |
| **Total** | | **~8-10 min** |
