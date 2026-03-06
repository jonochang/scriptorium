use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context;
use async_trait::async_trait;
use bookstore_domain::{Book, Inventory, InventoryError, Money, seed_church_bookstore};
use tokio::sync::RwLock;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestContext {
    pub tenant_id: String,
    pub locale: String,
}

impl Default for RequestContext {
    fn default() -> Self {
        Self { tenant_id: "default".to_string(), locale: "en-AU".to_string() }
    }
}

#[derive(Clone, Default)]
pub struct CatalogService {
    inventory: Arc<RwLock<Inventory>>,
}

impl CatalogService {
    pub fn with_seed() -> Self {
        Self { inventory: Arc::new(RwLock::new(seed_church_bookstore())) }
    }

    pub fn from_inventory(inventory: Inventory) -> Self {
        Self { inventory: Arc::new(RwLock::new(inventory)) }
    }

    pub async fn list_books(&self) -> Vec<Book> {
        let inventory = self.inventory.read().await;
        inventory.books().to_vec()
    }

    pub async fn add_book(&self, book: Book) -> Result<(), InventoryError> {
        let mut inventory = self.inventory.write().await;
        inventory.add_book(book)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderLineCostSnapshot {
    pub tenant_id: String,
    pub revenue: Money,
    pub cost: Money,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProfitReport {
    pub revenue: Money,
    pub cogs: Money,
    pub gross_profit: Money,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductRecord {
    pub tenant_id: String,
    pub product_id: String,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryRecord {
    pub tenant_id: String,
    pub product_id: String,
    pub on_hand: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OrderRecord {
    pub tenant_id: String,
    pub order_id: String,
    pub total: Money,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PaymentRecord {
    pub tenant_id: String,
    pub payment_id: String,
    pub amount: Money,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShiftRecord {
    pub tenant_id: String,
    pub shift_id: String,
    pub volunteer_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TenantRecord {
    pub tenant_id: String,
    pub display_name: String,
    pub default_locale: String,
}

#[async_trait]
pub trait ProductRepository: Send + Sync {
    async fn upsert(&self, product: ProductRecord) -> anyhow::Result<()>;
}

#[async_trait]
pub trait InventoryRepository: Send + Sync {
    async fn upsert_level(&self, inventory: InventoryRecord) -> anyhow::Result<()>;
}

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn insert(&self, order: OrderRecord) -> anyhow::Result<()>;
}

#[async_trait]
pub trait PaymentRepository: Send + Sync {
    async fn insert(&self, payment: PaymentRecord) -> anyhow::Result<()>;
}

#[async_trait]
pub trait ShiftRepository: Send + Sync {
    async fn insert(&self, shift: ShiftRecord) -> anyhow::Result<()>;
}

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn insert(&self, tenant: TenantRecord) -> anyhow::Result<()>;
}

#[async_trait]
pub trait ProfitReportRepository: Send + Sync {
    async fn record(&self, snapshot: OrderLineCostSnapshot) -> anyhow::Result<()>;
    async fn profit_for_tenant(&self, tenant_id: &str) -> anyhow::Result<ProfitReport>;
}

#[derive(Clone, Default, Debug)]
pub struct InMemoryProfitReportRepository {
    snapshots: Arc<RwLock<Vec<OrderLineCostSnapshot>>>,
}

impl InMemoryProfitReportRepository {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ProfitReportRepository for InMemoryProfitReportRepository {
    async fn record(&self, snapshot: OrderLineCostSnapshot) -> anyhow::Result<()> {
        let mut snapshots = self.snapshots.write().await;
        snapshots.push(snapshot);
        Ok(())
    }

    async fn profit_for_tenant(&self, tenant_id: &str) -> anyhow::Result<ProfitReport> {
        let snapshots = self.snapshots.read().await;
        let mut revenue_cents = 0_i64;
        let mut cogs_cents = 0_i64;
        let mut currency: Option<String> = None;

        for snapshot in snapshots.iter().filter(|s| s.tenant_id == tenant_id) {
            revenue_cents += snapshot.revenue.minor_units;
            cogs_cents += snapshot.cost.minor_units;
            if currency.is_none() {
                currency = Some(snapshot.revenue.currency.clone());
            }
        }

        let currency = currency.unwrap_or_else(|| "AUD".to_string());
        let revenue = Money::from_minor(&currency, revenue_cents).context("build revenue money")?;
        let cogs = Money::from_minor(&currency, cogs_cents).context("build cogs money")?;
        let gross_profit = Money::from_minor(&currency, revenue_cents - cogs_cents)
            .context("build gross money")?;

        Ok(ProfitReport { revenue, cogs, gross_profit })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PosCartItem {
    pub item_id: String,
    pub title: String,
    pub unit_price_cents: i64,
    pub quantity: i64,
    pub is_quick_item: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PosCartSnapshot {
    pub items: Vec<PosCartItem>,
    pub total_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PosPaymentOutcome {
    Paid,
    UnpaidIou,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PosCheckoutReceipt {
    pub outcome: PosPaymentOutcome,
    pub total_cents: i64,
    pub change_due_cents: i64,
    pub donation_cents: i64,
}

#[derive(Clone, Debug)]
struct PosCatalogItem {
    item_id: String,
    title: String,
    price_cents: i64,
    stock_on_hand: i64,
}

#[derive(Clone, Debug, Default)]
struct PosSession {
    cart: Vec<PosCartItem>,
}

#[derive(Default)]
struct PosStore {
    sessions: std::collections::HashMap<String, PosSession>,
    catalog_by_barcode: std::collections::HashMap<String, PosCatalogItem>,
    quick_items: std::collections::HashMap<String, PosCatalogItem>,
}

#[derive(Clone, Default)]
pub struct PosService {
    store: Arc<RwLock<PosStore>>,
    sequence: Arc<AtomicU64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckoutSession {
    pub session_id: String,
    pub total_cents: i64,
    pub email: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WebhookFinalizeStatus {
    Processed,
    Duplicate,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebhookFinalizeResult {
    pub status: WebhookFinalizeStatus,
    pub receipt_sent: bool,
}

#[derive(Default)]
struct StorefrontStore {
    sessions: std::collections::HashMap<String, CheckoutSession>,
    finalized_refs: std::collections::HashSet<String>,
    sent_receipts: std::collections::HashSet<String>,
}

#[derive(Clone, Default)]
pub struct StorefrontService {
    store: Arc<RwLock<StorefrontStore>>,
    sequence: Arc<AtomicU64>,
}

impl StorefrontService {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn create_checkout_session(
        &self,
        total_cents: i64,
        email: String,
    ) -> anyhow::Result<CheckoutSession> {
        if total_cents <= 0 {
            anyhow::bail!("checkout total must be positive");
        }
        let session_id = format!("chk-{}", self.sequence.fetch_add(1, Ordering::Relaxed));
        let session = CheckoutSession { session_id: session_id.clone(), total_cents, email };
        let mut store = self.store.write().await;
        store.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    pub async fn finalize_webhook(
        &self,
        external_ref: &str,
        session_id: &str,
    ) -> anyhow::Result<WebhookFinalizeResult> {
        let mut store = self.store.write().await;
        if store.finalized_refs.contains(external_ref) {
            return Ok(WebhookFinalizeResult {
                status: WebhookFinalizeStatus::Duplicate,
                receipt_sent: false,
            });
        }
        let session = store
            .sessions
            .get(session_id)
            .cloned()
            .with_context(|| format!("unknown checkout session {session_id}"))?;
        store.finalized_refs.insert(external_ref.to_string());
        store.sent_receipts.insert(session.email);
        Ok(WebhookFinalizeResult { status: WebhookFinalizeStatus::Processed, receipt_sent: true })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsbnMetadata {
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryReceipt {
    pub tenant_id: String,
    pub isbn: String,
    pub on_hand: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdminRole {
    Admin,
    Volunteer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminAuthSession {
    pub token: String,
    pub tenant_id: String,
    pub role: AdminRole,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminProduct {
    pub tenant_id: String,
    pub product_id: String,
    pub title: String,
    pub isbn: String,
    pub category: String,
    pub vendor: String,
    pub cost_cents: i64,
    pub retail_cents: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StockMovement {
    pub tenant_id: String,
    pub isbn: String,
    pub delta: i64,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminReportSummary {
    pub tenant_id: String,
    pub sales_cents: i64,
    pub donations_cents: i64,
    pub cogs_cents: i64,
    pub gross_profit_cents: i64,
    pub sales_by_payment: Vec<(String, i64)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SalesEvent {
    pub tenant_id: String,
    pub payment_method: String,
    pub sales_cents: i64,
    pub donations_cents: i64,
    pub cogs_cents: i64,
    pub occurred_on: String,
}

#[derive(Debug, Default)]
struct AdminStore {
    users: std::collections::HashMap<String, (String, String, AdminRole)>,
    auth_sessions: std::collections::HashMap<String, AdminAuthSession>,
    products: std::collections::HashMap<(String, String), AdminProduct>,
    inventory: std::collections::HashMap<(String, String), i64>,
    movements: Vec<StockMovement>,
    sales_events: Vec<SalesEvent>,
    session_seq: u64,
}

#[derive(Clone, Debug, Default)]
pub struct AdminService {
    store: Arc<RwLock<AdminStore>>,
}

impl AdminService {
    pub fn new() -> Self {
        let mut users = std::collections::HashMap::new();
        users.insert(
            "admin".to_string(),
            ("admin123".to_string(), "church-a".to_string(), AdminRole::Admin),
        );
        let store = AdminStore { users, ..AdminStore::default() };
        Self { store: Arc::new(RwLock::new(store)) }
    }

    pub async fn login(&self, username: &str, password: &str) -> anyhow::Result<AdminAuthSession> {
        let mut store = self.store.write().await;
        let (expected_password, tenant_id, role) =
            store.users.get(username).cloned().context("unknown account")?;
        if expected_password != password {
            anyhow::bail!("invalid password");
        }
        store.session_seq += 1;
        let token = format!("adm-{}", store.session_seq);
        let session = AdminAuthSession { token: token.clone(), tenant_id, role };
        store.auth_sessions.insert(token.clone(), session.clone());
        Ok(session)
    }

    pub async fn require_admin(&self, token: &str) -> anyhow::Result<AdminAuthSession> {
        let store = self.store.read().await;
        let session = store.auth_sessions.get(token).cloned().context("invalid auth token")?;
        if session.role != AdminRole::Admin {
            anyhow::bail!("admin role required");
        }
        Ok(session)
    }

    pub async fn lookup_isbn(&self, isbn: &str) -> anyhow::Result<IsbnMetadata> {
        let metadata = match isbn {
            "9780060652937" => IsbnMetadata {
                isbn: isbn.to_string(),
                title: "Celebration of Discipline".to_string(),
                author: "Richard Foster".to_string(),
                description: "Classic work on spiritual disciplines.".to_string(),
            },
            _ => IsbnMetadata {
                isbn: isbn.to_string(),
                title: "Unknown Title".to_string(),
                author: "Unknown Author".to_string(),
                description: "No metadata available.".to_string(),
            },
        };
        Ok(metadata)
    }

    pub async fn receive_inventory(
        &self,
        tenant_id: &str,
        isbn: &str,
        quantity: i64,
    ) -> anyhow::Result<InventoryReceipt> {
        if quantity <= 0 {
            anyhow::bail!("quantity must be positive");
        }
        let mut store = self.store.write().await;
        let key = (tenant_id.to_string(), isbn.to_string());
        let on_hand = {
            let updated =
                store.inventory.entry(key).and_modify(|qty| *qty += quantity).or_insert(quantity);
            *updated
        };
        store.movements.push(StockMovement {
            tenant_id: tenant_id.to_string(),
            isbn: isbn.to_string(),
            delta: quantity,
            reason: "receive".to_string(),
        });
        Ok(InventoryReceipt { tenant_id: tenant_id.to_string(), isbn: isbn.to_string(), on_hand })
    }

    pub async fn adjust_inventory(
        &self,
        tenant_id: &str,
        isbn: &str,
        delta: i64,
        reason: &str,
    ) -> anyhow::Result<InventoryReceipt> {
        if delta == 0 {
            anyhow::bail!("delta cannot be zero");
        }
        let mut store = self.store.write().await;
        let key = (tenant_id.to_string(), isbn.to_string());
        let current = store.inventory.get(&key).copied().unwrap_or(0);
        let on_hand = current + delta;
        if on_hand < 0 {
            anyhow::bail!("stock cannot be negative");
        }
        store.inventory.insert(key, on_hand);
        store.movements.push(StockMovement {
            tenant_id: tenant_id.to_string(),
            isbn: isbn.to_string(),
            delta,
            reason: reason.to_string(),
        });
        Ok(InventoryReceipt { tenant_id: tenant_id.to_string(), isbn: isbn.to_string(), on_hand })
    }

    pub async fn movement_journal(&self, tenant_id: &str) -> Vec<StockMovement> {
        let store = self.store.read().await;
        store.movements.iter().filter(|m| m.tenant_id == tenant_id).cloned().collect()
    }

    pub async fn upsert_product(&self, product: AdminProduct) -> anyhow::Result<()> {
        let mut store = self.store.write().await;
        store.products.insert((product.tenant_id.clone(), product.product_id.clone()), product);
        Ok(())
    }

    pub async fn list_products(&self, tenant_id: &str) -> Vec<AdminProduct> {
        let store = self.store.read().await;
        store.products.values().filter(|product| product.tenant_id == tenant_id).cloned().collect()
    }

    pub async fn delete_product(&self, tenant_id: &str, product_id: &str) -> anyhow::Result<()> {
        let mut store = self.store.write().await;
        let removed =
            store.products.remove(&(tenant_id.to_string(), product_id.to_string())).is_some();
        if !removed {
            anyhow::bail!("product not found");
        }
        Ok(())
    }

    pub async fn list_categories(&self, tenant_id: &str) -> Vec<String> {
        let store = self.store.read().await;
        let mut categories = store
            .products
            .values()
            .filter(|product| product.tenant_id == tenant_id)
            .map(|product| product.category.clone())
            .collect::<Vec<_>>();
        categories.sort();
        categories.dedup();
        categories
    }

    pub async fn list_vendors(&self, tenant_id: &str) -> Vec<String> {
        let store = self.store.read().await;
        let mut vendors = store
            .products
            .values()
            .filter(|product| product.tenant_id == tenant_id)
            .map(|product| product.vendor.clone())
            .collect::<Vec<_>>();
        vendors.sort();
        vendors.dedup();
        vendors
    }

    pub async fn record_sales_event(&self, event: SalesEvent) {
        let mut store = self.store.write().await;
        store.sales_events.push(event);
    }

    pub async fn report_summary(&self, tenant_id: &str) -> AdminReportSummary {
        self.report_summary_range(tenant_id, None, None).await
    }

    pub async fn report_summary_range(
        &self,
        tenant_id: &str,
        from: Option<&str>,
        to: Option<&str>,
    ) -> AdminReportSummary {
        let store = self.store.read().await;
        let mut sales = 0_i64;
        let mut donations = 0_i64;
        let mut cogs = 0_i64;
        let mut by_payment = std::collections::HashMap::<String, i64>::new();
        for event in store
            .sales_events
            .iter()
            .filter(|ev| ev.tenant_id == tenant_id)
            .filter(|ev| from.is_none_or(|min| ev.occurred_on.as_str() >= min))
            .filter(|ev| to.is_none_or(|max| ev.occurred_on.as_str() <= max))
        {
            sales += event.sales_cents;
            donations += event.donations_cents;
            cogs += event.cogs_cents;
            *by_payment.entry(event.payment_method.clone()).or_default() += event.sales_cents;
        }
        let gross_profit = sales - cogs;
        let mut sales_by_payment = by_payment.into_iter().collect::<Vec<_>>();
        sales_by_payment.sort_by(|a, b| a.0.cmp(&b.0));
        AdminReportSummary {
            tenant_id: tenant_id.to_string(),
            sales_cents: sales,
            donations_cents: donations,
            cogs_cents: cogs,
            gross_profit_cents: gross_profit,
            sales_by_payment,
        }
    }
}

impl PosService {
    pub fn with_seed() -> Self {
        let mut store = PosStore::default();
        store.catalog_by_barcode.insert(
            "9780060652937".to_string(),
            PosCatalogItem {
                item_id: "bk-900".to_string(),
                title: "Celebration of Discipline".to_string(),
                price_cents: 1699,
                stock_on_hand: 10,
            },
        );
        store.quick_items.insert(
            "prayer-card-50c".to_string(),
            PosCatalogItem {
                item_id: "prayer-card-50c".to_string(),
                title: "Prayer Card".to_string(),
                price_cents: 50,
                stock_on_hand: 100,
            },
        );
        Self { store: Arc::new(RwLock::new(store)), sequence: Arc::new(AtomicU64::new(1)) }
    }

    pub async fn login_with_pin(&self, pin: &str) -> anyhow::Result<String> {
        if pin != "1234" {
            anyhow::bail!("invalid shift pin");
        }
        let token = format!("pos-{}", self.sequence.fetch_add(1, Ordering::Relaxed));
        let mut store = self.store.write().await;
        store.sessions.insert(token.clone(), PosSession::default());
        Ok(token)
    }

    pub async fn scan_item(&self, token: &str, barcode: &str) -> anyhow::Result<PosCartSnapshot> {
        let mut store = self.store.write().await;
        let catalog_item = store
            .catalog_by_barcode
            .get(barcode)
            .cloned()
            .with_context(|| format!("unknown barcode {barcode}"))?;
        Self::add_to_cart(
            store.sessions.get_mut(token).context("invalid session token")?,
            &catalog_item,
            1,
            false,
        );
        Ok(Self::snapshot(store.sessions.get(token).expect("session exists")))
    }

    pub async fn add_quick_item(
        &self,
        token: &str,
        item_id: &str,
        quantity: i64,
    ) -> anyhow::Result<PosCartSnapshot> {
        let mut store = self.store.write().await;
        let item = store.quick_items.get(item_id).cloned().context("unknown quick item")?;
        Self::add_to_cart(
            store.sessions.get_mut(token).context("invalid session token")?,
            &item,
            quantity,
            true,
        );
        Ok(Self::snapshot(store.sessions.get(token).expect("session exists")))
    }

    pub async fn checkout_external_card(
        &self,
        token: &str,
        _external_ref: &str,
    ) -> anyhow::Result<PosCheckoutReceipt> {
        self.finalize_paid_sale(token, 0, 0).await
    }

    pub async fn checkout_cash(
        &self,
        token: &str,
        tendered_cents: i64,
        donate_change: bool,
    ) -> anyhow::Result<PosCheckoutReceipt> {
        let total = {
            let store = self.store.read().await;
            Self::cart_total(store.sessions.get(token).context("invalid session token")?)
        };
        let mut change_due = (tendered_cents - total).max(0);
        let mut donation = 0;
        if donate_change {
            donation = change_due;
            change_due = 0;
        }
        self.finalize_paid_sale(token, change_due, donation).await
    }

    pub async fn checkout_iou(
        &self,
        token: &str,
        customer_name: &str,
    ) -> anyhow::Result<PosCheckoutReceipt> {
        if customer_name.trim().is_empty() {
            anyhow::bail!("customer name is required");
        }
        let mut store = self.store.write().await;
        let session = store.sessions.get_mut(token).context("invalid session token")?;
        let total = Self::cart_total(session);
        if total <= 0 {
            anyhow::bail!("cart is empty");
        }
        session.cart.clear();
        Ok(PosCheckoutReceipt {
            outcome: PosPaymentOutcome::UnpaidIou,
            total_cents: total,
            change_due_cents: 0,
            donation_cents: 0,
        })
    }

    fn add_to_cart(
        session: &mut PosSession,
        item: &PosCatalogItem,
        quantity: i64,
        is_quick_item: bool,
    ) {
        if let Some(existing) = session.cart.iter_mut().find(|entry| entry.item_id == item.item_id)
        {
            existing.quantity += quantity;
            return;
        }
        session.cart.push(PosCartItem {
            item_id: item.item_id.clone(),
            title: item.title.clone(),
            unit_price_cents: item.price_cents,
            quantity,
            is_quick_item,
        });
    }

    fn cart_total(session: &PosSession) -> i64 {
        session.cart.iter().map(|line| line.unit_price_cents * line.quantity).sum()
    }

    fn snapshot(session: &PosSession) -> PosCartSnapshot {
        PosCartSnapshot { items: session.cart.clone(), total_cents: Self::cart_total(session) }
    }

    async fn finalize_paid_sale(
        &self,
        token: &str,
        change_due_cents: i64,
        donation_cents: i64,
    ) -> anyhow::Result<PosCheckoutReceipt> {
        let mut store = self.store.write().await;
        let lines = store.sessions.get(token).cloned().context("invalid session token")?.cart;
        let total = lines.iter().map(|line| line.unit_price_cents * line.quantity).sum::<i64>();
        if total <= 0 {
            anyhow::bail!("cart is empty");
        }

        for line in lines.iter().filter(|line| !line.is_quick_item) {
            if let Some(item) =
                store.catalog_by_barcode.values_mut().find(|item| item.item_id == line.item_id)
            {
                item.stock_on_hand -= line.quantity;
            }
        }
        if let Some(session) = store.sessions.get_mut(token) {
            session.cart.clear();
        }

        Ok(PosCheckoutReceipt {
            outcome: PosPaymentOutcome::Paid,
            total_cents: total,
            change_due_cents,
            donation_cents,
        })
    }
}
