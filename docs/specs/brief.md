# Scriptorium: Church Bookstore & POS System
## Functional Project Brief

---

## 1. Project Overview

**Objective:** Build "Scriptorium," a unified system for managing church bookstores that functions simultaneously as a public-facing online store and a streamlined, mobile Point-of-Sale (POS) for in-person volunteers.
The platform must support multiple church bookshops (sister stores) from the beginning, with language internationalisation support from the beginning.

**Core Philosophy:** 
Sundays after the church service are chaotic, with a 30-minute window of high traffic. Volunteers are often elderly or not tech-savvy. Therefore, the in-person POS must be **ultra-simple, fast, and require minimal typing**. The system relies on volunteers using their own smartphones to scan barcodes and process payments, eliminating the need for bulky, expensive POS hardware.

---

## 2. User Roles

1.  **Volunteer (Cashier):** Needs fast access, large buttons, and foolproof workflows. No complex passwords; access is via a simple PIN.
2.  **Administrator (Manager):** Needs comprehensive tools for inventory management, reporting, and order reconciliation.
3.  **Customer (Online):** Needs standard e-commerce browsing, search, and secure checkout.
4.  **Bookshop Account Owner (Tenant Admin):** Manages one church bookshop account (catalog, users, settings, reporting) while data remains isolated from other sister bookshops.

---

## 3. Core Workflows & Scenarios (BDD Focus)

The following workflows outline the expected system behavior. They are designed to be directly translatable into Behavior-Driven Development (BDD) tests (e.g., *Given / When / Then*).

### Scenario A: The Sunday Rush (Standard Book Purchase)
1.  **Given** a Volunteer is at the POS login screen on their mobile phone.
2.  **When** they enter their 4-digit Shift PIN.
3.  **Then** they are immediately taken to the active POS screen.
4.  **When** they tap "Scan Item" and point the phone camera at a book's ISBN.
5.  **Then** the system instantly adds the book to the cart and displays the total.
6.  **When** the Volunteer taps "Pay via Credit Card".
7.  **Then** the system hands off the total to the integrated payment app (e.g., Square).
8.  **When** the payment is successful.
9.  **Then** the system automatically deducts the inventory, records the sale, and shows a "Sale Complete" screen with an optional "Email Receipt" input field.

### Scenario B: Un-barcoded Items (Candles & Cards)
1.  **Given** the Volunteer is on the active POS screen.
2.  **When** a customer wants to buy a 50-cent prayer card (which has no barcode).
3.  **Then** the Volunteer taps the "Quick Items" visual grid.
4.  **When** they tap the "Prayer Card (50¢)" button twice.
5.  **Then** $1.00 is added to the cart.

### Scenario C: Cash Payment with Donation / Exact Change
1.  **Given** the cart total is $16.50.
2.  **When** the Volunteer taps "Pay with Cash".
3.  **Then** the system displays quick-tap buttons: `$16.50 (Exact)`, `$20.00`, and `Custom`.
4.  **When** the Volunteer taps `$20.00`.
5.  **Then** the system displays a large prompt: "Change Due: $3.50".
6.  **When** the customer says "Keep the change for the church", the Volunteer taps "Round-up / Donate Change".
7.  **Then** the system records $16.50 for items, $3.50 as a donation, logs the sale, and displays $0.00 change due.

### Scenario D: The "I Forgot My Wallet" (IOU / Tab)
1.  **Given** the cart contains items and the customer realizes they have no money.
2.  **When** the Volunteer taps the "Put on Tab / IOU" payment method.
3.  **Then** the system prompts for a Customer Name.
4.  **When** the Volunteer types "John Doe" and completes the order.
5.  **Then** the inventory is deducted, the items are handed over, and the order is flagged as "Unpaid / IOU" in the Admin dashboard for later follow-up.

### Scenario E: Rapid Inventory Addition (Admin)
1.  **Given** an Admin receives a box of new books.
2.  **When** they use their mobile phone camera in "Add Inventory" to scan a book's ISBN barcode.
3.  **Then** the system queries an external metadata API (e.g., Google Books) and auto-fills the Title, Author, Cover Image, and Description.
4.  **When** the Admin enters the Cost Price, Retail Price, and Quantity (e.g., 5).
5.  **Then** the system saves the product and updates the stock levels.

### Scenario F: Profit Visibility for Treasurer
1.  **Given** sales and inventory costs have been recorded.
2.  **When** an Admin opens the financial report for a date range.
3.  **Then** the system shows revenue, cost of goods sold, and gross profit (revenue minus cost).

### Scenario G: Multi-Bookshop Isolation
1.  **Given** two sister bookshops use Scriptorium.
2.  **When** an Admin from Bookshop A logs in.
3.  **Then** they can only view and modify data belonging to Bookshop A.

---

## 4. Feature Specifications

### 4.1. Mobile POS Interface (For Volunteers)
*   **PIN Login:** Quick access for assigned volunteer shifts.
*   **Camera Barcode Scanner:** Utilizes the device's native camera to scan EAN-13, ISBN-13, ISBN-10, and custom system-generated barcodes.
*   **Quick-Tap Grid:** A visual menu for frequently sold, non-barcoded items (candles, charcoal, small icons).
*   **Discount Toggle:** Pre-set buttons to apply standard discounts (e.g., "10% Clergy", "15% Volunteer").
*   **Split Payments / App-Switching:** Seamless handoff to card reader apps (like Square POS) to avoid manual amount entry, plus cash handling with automated change calculation.
*   **Digital Receipts:** Post-purchase screen allows entering an email address to send a PDF receipt, skipping this if the customer declines.

### 4.2. Online Storefront (For Customers)
*   **Catalog Browsing:** Browse by Category, Author, or Theme.
*   **Search:** Full-text search across Title, Author, Description, and ISBN.
*   **Product Detail Pages:** Display cover image, description, price, and real-time stock availability.
*   **E-commerce Checkout:** Standard shopping cart functionality with secure online payment integration.
*   **Automated Emails:** Instant order confirmation and PDF invoice generation upon successful checkout.

### 4.3. Inventory & Catalog Management (For Admins)
*   **Auto-Metadata Lookup:** Fetch book details via external APIs using the ISBN to drastically reduce data entry.
*   **Mobile Camera Inventory Intake:** Admins can add books directly from their phone by scanning ISBN with camera and confirming auto-populated metadata.
*   **Custom SKU Generation:** Ability to generate and print custom barcode labels for non-book items (e.g., local crafts).
*   **Bundles/Kits:** Ability to group multiple individual items under one purchasable button (e.g., "Baptismal Kit") that correctly deducts inventory from the constituent items.
*   **Vendor / Consignment Tracking:** Tag items to specific vendors (e.g., a local monastery) to run reports on how much is owed to third parties.
*   **Stock Operations:** Workflows for Receiving Stock, Adjusting Stock (Damage, Donation, Correction), and full Stocktakes.
*   **Low Stock Alerts:** System flags items falling below defined reorder thresholds.

### 4.4. Order & Financial Management (For Admins)
*   **Unified Order View:** See both POS and Online orders in one list, clearly distinguished.
*   **IOU Management:** A specific view for tracking, resolving, and settling "Tab/Unpaid" orders.
*   **Refunds & Adjustments:** Workflows to restock items and refund payments.
*   **Gross Profit Reporting:** Track and report Revenue, Cost of Goods Sold (COGS), and Gross Profit for treasurer/accounting review.
*   **Reporting:** Generate reports on:
    *   Sales by date range / Shift.
    *   Sales by Payment Method (Cash, Card, IOU).
    *   Gross Profit (Revenue - COGS) by date range / Category / Product.
    *   Consignment / Vendor liabilities.
    *   Inventory valuation (Cost vs. Retail value).
    *   Donations collected via POS round-ups.

### 4.5. Platform Capabilities (From Day 1)
*   **Internationalisation (i18n):** UI text, email templates, and product metadata presentation should support multiple languages from the start.
*   **Multi-Account / Multi-Tenant:** Multiple church bookshops can operate in one platform with strict tenant-level data isolation and account-specific users/settings.

### 4.6. Future Capability
*   **Library Module (Later Phase):** Support borrowing and returning books with due dates, borrower records, and overdue tracking.

---

## 5. Logical Data Model Requirements

To support the features above, the business logic must account for the following data structures (implementation independent):

**Product Entity:**
*   Identifiers (ID, Barcode/ISBN, SKU)
*   Metadata (Title, Author, Publisher, Description, Cover Image)
*   Financials (`cost_price`, `retail_price`, `is_taxable`)
*   Classification (Categories, `vendor_id` for consignment)
*   Status (`is_active` to hide seasonal items)

**Inventory Entity:**
*   Quantities (`on_hand`, `reserved`, `reorder_point`)

**Order Entity:**
*   Source (`channel`: Online vs. POS)
*   Personnel (`cashier_name` / `shift_id`)
*   Financials (Subtotal, Tax, Discount, `donation_amount`, Total, COGS, Gross Profit contribution)
*   Status (Paid, Unpaid/IOU, Refunded)
*   Customer details (Name, Email - optional for POS)

**Payment / Transaction Entity:**
*   Method (Cash, Online Card, Terminal Card, IOU)
*   Amounts (`amount_tendered`, `change_due`)
*   External Reference IDs (for reconciling with payment processors)

**Tenant / Account Entity:**
*   Identifiers (`tenant_id`, account name)
*   Settings (default language, currency, timezone)
*   Data ownership boundary (products, inventory, orders, users, reports scoped to tenant)

**Localisation Entity / Resources:**
*   Supported locales per tenant
*   Translation resources for UI and communication templates

**Library Loan Entity (Later Phase):**
*   Borrower, item copy, checkout date, due date, return date, status

---

## 6. Phased Rollout Plan

**Phase 1: MVP (Minimum Viable Product)**
*   Core Product Catalog & Inventory tracking.
*   Mobile POS with Camera Barcode Scanning & Quick-Tap Grid.
*   Admin mobile camera ISBN intake with auto metadata lookup.
*   Cash & Terminal/Card processing (manual or simple app-switch).
*   Online Cart & Checkout.
*   Automated PDF / Email Receipts.
*   Gross profit reporting basics (Revenue, COGS, Gross Profit).
*   Internationalisation foundations (multi-language architecture).
*   Multi-tenant foundations for sister church bookshops.

**Phase 2: Administrative & Workflow Enhancements**
*   IOU / "Put it on my tab" workflow.
*   Vendor/Consignment tracking & reports.
*   Custom Barcode generation and printing.
*   Expanded localisation coverage and translation management tools.
*   Advanced tenant/account administration.

**Phase 3: Advanced Features**
*   Customer Accounts / Loyalty programs.
*   Multi-location inventory (if the church has a secondary campus/kiosk).
*   Integration with primary church accounting software.
*   Library borrowing/loan module.
