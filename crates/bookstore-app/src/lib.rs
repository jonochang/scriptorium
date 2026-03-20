pub mod seed;

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Context;
use async_trait::async_trait;
use bookstore_domain::{Book, Inventory, InventoryError, Money, OrderChannel, OrderStatus, PaymentMethod};
use chrono::NaiveDateTime;
use tokio::sync::RwLock;

use seed::SeedData;

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
        Self::from_seed(&SeedData::default())
    }

    pub fn from_seed(seed: &SeedData) -> Self {
        let mut inventory = Inventory::new();
        for book in &seed.catalog.books {
            let _ = inventory.add_book(Book {
                id: book.id.clone(),
                title: book.title.clone(),
                author: book.author.clone(),
                category: book.category.clone(),
                price_cents: book.price_cents,
            });
        }
        Self { inventory: Arc::new(RwLock::new(inventory)) }
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
    pub discount_cents: i64,
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
    pin: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckoutSession {
    pub session_id: String,
    pub tenant_id: String,
    pub sales_cents: i64,
    pub shipping_cents: i64,
    pub tax_cents: i64,
    pub donation_cents: i64,
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
    pub session: CheckoutSession,
}

#[derive(Default)]
struct StorefrontStore {
    sessions: std::collections::HashMap<String, CheckoutSession>,
    finalized_refs: std::collections::HashSet<String>,
    sent_receipts: std::collections::HashSet<String>,
    order_created_sessions: std::collections::HashSet<String>,
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
        tenant_id: String,
        sales_cents: i64,
        shipping_cents: i64,
        tax_cents: i64,
        donation_cents: i64,
        email: String,
    ) -> anyhow::Result<CheckoutSession> {
        if sales_cents <= 0 {
            anyhow::bail!("checkout subtotal must be positive");
        }
        if shipping_cents < 0 || tax_cents < 0 || donation_cents < 0 {
            anyhow::bail!("checkout amounts cannot be negative");
        }
        let total_cents = sales_cents + donation_cents;
        if total_cents <= 0 {
            anyhow::bail!("checkout total must be positive");
        }
        let session_id = format!("chk-{}", self.sequence.fetch_add(1, Ordering::Relaxed));
        let session = CheckoutSession {
            session_id: session_id.clone(),
            tenant_id,
            sales_cents,
            shipping_cents,
            tax_cents,
            donation_cents,
            total_cents,
            email,
        };
        let mut store = self.store.write().await;
        store.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    /// Mark a session as having its order already created (e.g. in demo mode).
    /// Subsequent webhook calls for this session will return Duplicate.
    pub async fn mark_order_created(&self, session_id: &str) {
        let mut store = self.store.write().await;
        store.order_created_sessions.insert(session_id.to_string());
    }

    pub async fn finalize_webhook(
        &self,
        external_ref: &str,
        session_id: &str,
    ) -> anyhow::Result<WebhookFinalizeResult> {
        let mut store = self.store.write().await;
        if store.finalized_refs.contains(external_ref)
            || store.order_created_sessions.contains(session_id)
        {
            let session = store
                .sessions
                .get(session_id)
                .cloned()
                .with_context(|| format!("unknown checkout session {session_id}"))?;
            return Ok(WebhookFinalizeResult {
                status: WebhookFinalizeStatus::Duplicate,
                receipt_sent: false,
                session,
            });
        }
        let session = store
            .sessions
            .get(session_id)
            .cloned()
            .with_context(|| format!("unknown checkout session {session_id}"))?;
        store.finalized_refs.insert(external_ref.to_string());
        store.sent_receipts.insert(session.email.clone());
        Ok(WebhookFinalizeResult {
            status: WebhookFinalizeStatus::Processed,
            receipt_sent: true,
            session,
        })
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
pub struct AdminBootstrap {
    pub username: String,
    pub password: String,
    pub tenant_id: String,
}

impl AdminBootstrap {
    pub fn from_seed(seed: &seed::SeedData) -> Self {
        Self {
            username: seed.defaults.admin_username.clone(),
            password: seed.defaults.admin_password.clone(),
            tenant_id: seed.defaults.tenant_id.clone(),
        }
    }

    pub fn local_defaults() -> Self {
        Self::from_seed(&seed::SeedData::default())
    }

    pub fn from_env() -> Self {
        let defaults = Self::from_seed(&seed::SeedData::default());
        Self {
            username: std::env::var("SCRIPTORIUM_ADMIN_USERNAME")
                .unwrap_or(defaults.username),
            password: std::env::var("SCRIPTORIUM_ADMIN_PASSWORD")
                .unwrap_or(defaults.password),
            tenant_id: std::env::var("SCRIPTORIUM_DEFAULT_TENANT_ID")
                .unwrap_or(defaults.tenant_id),
        }
    }
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
    pub cover_image_key: Option<String>,
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
    pub sales_by_payment: Vec<(PaymentMethod, i64)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SalesEvent {
    pub tenant_id: String,
    pub payment_method: PaymentMethod,
    pub sales_cents: i64,
    pub donations_cents: i64,
    pub cogs_cents: i64,
    pub occurred_at: NaiveDateTime,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdminOrder {
    pub order_id: String,
    pub tenant_id: String,
    pub customer_name: String,
    pub channel: OrderChannel,
    pub status: OrderStatus,
    pub payment_method: PaymentMethod,
    pub total_cents: i64,
    pub created_at: NaiveDateTime,
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
    order_seq: u64,
    orders: Vec<AdminOrder>,
}

#[derive(Clone, Debug)]
pub struct AdminService {
    bootstrap: AdminBootstrap,
    store: Arc<RwLock<AdminStore>>,
}

impl AdminService {
    pub fn with_bootstrap(bootstrap: AdminBootstrap) -> Self {
        Self::with_bootstrap_and_seed(bootstrap, &SeedData::default())
    }

    pub fn with_bootstrap_and_seed(bootstrap: AdminBootstrap, seed: &SeedData) -> Self {
        let mut users = std::collections::HashMap::new();
        users.insert(
            bootstrap.username.clone(),
            (bootstrap.password.clone(), bootstrap.tenant_id.clone(), AdminRole::Admin),
        );
        let mut store = AdminStore { users, ..AdminStore::default() };

        for product in &seed.admin.products {
            store.products.insert(
                (bootstrap.tenant_id.clone(), format!("prd-{}", product.isbn)),
                AdminProduct {
                    tenant_id: bootstrap.tenant_id.clone(),
                    product_id: format!("prd-{}", product.isbn),
                    title: product.title.clone(),
                    isbn: product.isbn.clone(),
                    category: product.category.clone(),
                    vendor: product.vendor.clone(),
                    cost_cents: product.cost_cents,
                    retail_cents: product.retail_cents,
                    cover_image_key: None,
                },
            );
        }

        Self { bootstrap, store: Arc::new(RwLock::new(store)) }
    }

    pub fn with_local_defaults() -> Self {
        Self::with_bootstrap(AdminBootstrap::local_defaults())
    }

    pub fn new() -> Self {
        Self::with_bootstrap(AdminBootstrap::from_env())
    }

    pub fn bootstrap(&self) -> &AdminBootstrap {
        &self.bootstrap
    }

    pub fn default_tenant_id(&self) -> &str {
        &self.bootstrap.tenant_id
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
        let normalized = isbn.chars().filter(|ch| ch.is_ascii_digit()).collect::<String>();
        let store = self.store.read().await;
        let product = store
            .products
            .values()
            .find(|p| p.isbn == normalized)
            .context("no product found for that ISBN")?;
        Ok(IsbnMetadata {
            isbn: normalized,
            title: product.title.clone(),
            author: String::new(),
            description: format!("{} — {}", product.category, product.vendor),
        })
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

    pub async fn inventory_on_hand(&self, tenant_id: &str, isbn: &str) -> i64 {
        let store = self.store.read().await;
        store.inventory.get(&(tenant_id.to_string(), isbn.to_string())).copied().unwrap_or(0)
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
        from: Option<NaiveDateTime>,
        to: Option<NaiveDateTime>,
    ) -> AdminReportSummary {
        let store = self.store.read().await;
        let mut sales = 0_i64;
        let mut donations = 0_i64;
        let mut cogs = 0_i64;
        let mut by_payment = std::collections::HashMap::<PaymentMethod, i64>::new();
        for event in store
            .sales_events
            .iter()
            .filter(|ev| ev.tenant_id == tenant_id)
            .filter(|ev| from.is_none_or(|min| ev.occurred_at >= min))
            .filter(|ev| to.is_none_or(|max| ev.occurred_at <= max))
        {
            sales += event.sales_cents;
            donations += event.donations_cents;
            cogs += event.cogs_cents;
            *by_payment.entry(event.payment_method).or_default() += event.sales_cents;
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

    pub async fn create_order(
        &self,
        tenant_id: &str,
        customer_name: &str,
        channel: OrderChannel,
        status: OrderStatus,
        payment_method: PaymentMethod,
        total_cents: i64,
        created_at: NaiveDateTime,
    ) -> AdminOrder {
        let mut store = self.store.write().await;
        store.order_seq += 1;
        let order = AdminOrder {
            order_id: format!("ORD-{}", 1000 + store.order_seq),
            tenant_id: tenant_id.to_string(),
            customer_name: customer_name.to_string(),
            channel,
            status,
            payment_method,
            total_cents,
            created_at,
        };
        store.orders.push(order.clone());
        order
    }

    pub async fn list_orders(&self, tenant_id: &str) -> Vec<AdminOrder> {
        let store = self.store.read().await;
        let mut orders = store
            .orders
            .iter()
            .filter(|order| order.tenant_id == tenant_id)
            .cloned()
            .collect::<Vec<_>>();
        orders.sort_by(|a, b| b.order_id.cmp(&a.order_id));
        orders
    }

    pub async fn mark_order_paid(
        &self,
        tenant_id: &str,
        order_id: &str,
    ) -> anyhow::Result<AdminOrder> {
        let mut store = self.store.write().await;
        let order = store
            .orders
            .iter_mut()
            .find(|order| order.tenant_id == tenant_id && order.order_id == order_id)
            .context("order not found")?;
        order.status = OrderStatus::Paid;
        order.payment_method = PaymentMethod::IouSettled;
        Ok(order.clone())
    }
}

impl Default for AdminService {
    fn default() -> Self {
        Self::with_local_defaults()
    }
}

impl PosService {
    pub fn with_seed() -> Self {
        Self::from_seed(&SeedData::default())
    }

    pub fn from_seed(seed: &SeedData) -> Self {
        let mut store = PosStore::default();
        for item in &seed.pos.barcode_items {
            store.catalog_by_barcode.insert(
                item.barcode.clone(),
                PosCatalogItem {
                    item_id: item.item_id.clone(),
                    title: item.title.clone(),
                    price_cents: item.price_cents,
                    stock_on_hand: item.stock_on_hand,
                },
            );
        }
        for item in &seed.pos.quick_items {
            store.quick_items.insert(
                item.item_id.clone(),
                PosCatalogItem {
                    item_id: item.item_id.clone(),
                    title: item.title.clone(),
                    price_cents: item.price_cents,
                    stock_on_hand: item.stock_on_hand,
                },
            );
        }
        let pin = seed.defaults.pos_pin.clone();
        Self {
            store: Arc::new(RwLock::new(store)),
            sequence: Arc::new(AtomicU64::new(1)),
            pin,
        }
    }

    pub async fn login_with_pin(&self, pin: &str) -> anyhow::Result<String> {
        if pin != self.pin {
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
        let session = store.sessions.get_mut(token).context("invalid session token")?;
        Self::add_to_cart(session, &catalog_item, 1, false)?;
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
        let session = store.sessions.get_mut(token).context("invalid session token")?;
        Self::add_to_cart(session, &item, quantity, true)?;
        Ok(Self::snapshot(store.sessions.get(token).expect("session exists")))
    }

    pub async fn set_cart_quantity(
        &self,
        token: &str,
        item_id: &str,
        quantity: i64,
    ) -> anyhow::Result<PosCartSnapshot> {
        if quantity < 0 {
            anyhow::bail!("quantity cannot be negative");
        }
        let mut store = self.store.write().await;
        if quantity == 0 {
            let session = store.sessions.get_mut(token).context("invalid session token")?;
            let before = session.cart.len();
            session.cart.retain(|entry| entry.item_id != item_id);
            if session.cart.len() == before {
                anyhow::bail!("cart item not found");
            }
        } else {
            let is_quick_item = store
                .sessions
                .get(token)
                .context("invalid session token")?
                .cart
                .iter()
                .find(|entry| entry.item_id == item_id)
                .map(|entry| entry.is_quick_item)
                .context("cart item not found")?;
            let available = Self::stock_on_hand(&store, item_id, is_quick_item)
                .context("catalog item not found")?;
            if quantity > available {
                anyhow::bail!("not enough stock on hand");
            }
            let session = store.sessions.get_mut(token).context("invalid session token")?;
            let entry = session
                .cart
                .iter_mut()
                .find(|entry| entry.item_id == item_id)
                .context("cart item not found")?;
            entry.quantity = quantity;
        }
        let session = store.sessions.get(token).context("invalid session token")?;
        Ok(Self::snapshot(session))
    }

    pub async fn checkout_external_card(
        &self,
        token: &str,
        _external_ref: &str,
        discount_cents: i64,
    ) -> anyhow::Result<PosCheckoutReceipt> {
        self.finalize_paid_sale(token, 0, 0, discount_cents).await
    }

    pub async fn checkout_cash(
        &self,
        token: &str,
        tendered_cents: i64,
        donate_change: bool,
        discount_cents: i64,
    ) -> anyhow::Result<PosCheckoutReceipt> {
        let subtotal = {
            let store = self.store.read().await;
            Self::cart_total(store.sessions.get(token).context("invalid session token")?)
        };
        let total = Self::discounted_total(subtotal, discount_cents)?;
        if tendered_cents < total {
            anyhow::bail!("tendered amount is less than cart total");
        }
        let mut change_due = (tendered_cents - total).max(0);
        let mut donation = 0;
        if donate_change {
            donation = change_due;
            change_due = 0;
        }
        self.finalize_paid_sale(token, change_due, donation, discount_cents).await
    }

    pub async fn checkout_iou(
        &self,
        token: &str,
        customer_name: &str,
        discount_cents: i64,
    ) -> anyhow::Result<PosCheckoutReceipt> {
        if customer_name.trim().is_empty() {
            anyhow::bail!("customer name is required");
        }
        let mut store = self.store.write().await;
        let (lines, total) = {
            let session = store.sessions.get(token).context("invalid session token")?;
            (session.cart.clone(), Self::cart_total(session))
        };
        if total <= 0 {
            anyhow::bail!("cart is empty");
        }
        let total = Self::discounted_total(total, discount_cents)?;
        Self::apply_stock_deductions(&mut store, &lines)?;
        let session = store.sessions.get_mut(token).context("invalid session token")?;
        session.cart.clear();
        Ok(PosCheckoutReceipt {
            outcome: PosPaymentOutcome::UnpaidIou,
            total_cents: total,
            change_due_cents: 0,
            donation_cents: 0,
            discount_cents,
        })
    }

    fn add_to_cart(
        session: &mut PosSession,
        item: &PosCatalogItem,
        quantity: i64,
        is_quick_item: bool,
    ) -> anyhow::Result<()> {
        if quantity <= 0 {
            anyhow::bail!("quantity must be positive");
        }
        let next_quantity = session
            .cart
            .iter()
            .find(|entry| entry.item_id == item.item_id)
            .map(|entry| entry.quantity + quantity)
            .unwrap_or(quantity);
        if next_quantity > item.stock_on_hand {
            anyhow::bail!("not enough stock on hand");
        }
        if let Some(existing) = session.cart.iter_mut().find(|entry| entry.item_id == item.item_id)
        {
            existing.quantity += quantity;
            return Ok(());
        }
        session.cart.push(PosCartItem {
            item_id: item.item_id.clone(),
            title: item.title.clone(),
            unit_price_cents: item.price_cents,
            quantity,
            is_quick_item,
        });
        Ok(())
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
        discount_cents: i64,
    ) -> anyhow::Result<PosCheckoutReceipt> {
        let mut store = self.store.write().await;
        let lines = store.sessions.get(token).cloned().context("invalid session token")?.cart;
        let subtotal = lines.iter().map(|line| line.unit_price_cents * line.quantity).sum::<i64>();
        if subtotal <= 0 {
            anyhow::bail!("cart is empty");
        }
        let total = Self::discounted_total(subtotal, discount_cents)?;

        Self::apply_stock_deductions(&mut store, &lines)?;
        if let Some(session) = store.sessions.get_mut(token) {
            session.cart.clear();
        }

        Ok(PosCheckoutReceipt {
            outcome: PosPaymentOutcome::Paid,
            total_cents: total,
            change_due_cents,
            donation_cents,
            discount_cents,
        })
    }

    fn discounted_total(total_cents: i64, discount_cents: i64) -> anyhow::Result<i64> {
        if total_cents <= 0 {
            anyhow::bail!("cart is empty");
        }
        if discount_cents < 0 {
            anyhow::bail!("discount cannot be negative");
        }
        if discount_cents >= total_cents {
            anyhow::bail!("discount must be less than cart total");
        }
        Ok(total_cents - discount_cents)
    }

    fn stock_on_hand(store: &PosStore, item_id: &str, is_quick_item: bool) -> Option<i64> {
        if is_quick_item {
            return store.quick_items.get(item_id).map(|item| item.stock_on_hand);
        }
        store
            .catalog_by_barcode
            .values()
            .find(|item| item.item_id == item_id)
            .map(|item| item.stock_on_hand)
    }

    fn apply_stock_deductions(store: &mut PosStore, lines: &[PosCartItem]) -> anyhow::Result<()> {
        for line in lines {
            let available = Self::stock_on_hand(store, &line.item_id, line.is_quick_item)
                .context("catalog item not found")?;
            if line.quantity > available {
                anyhow::bail!("not enough stock on hand");
            }
        }
        for line in lines {
            if line.is_quick_item {
                let item =
                    store.quick_items.get_mut(&line.item_id).context("catalog item not found")?;
                item.stock_on_hand -= line.quantity;
            } else {
                let item = store
                    .catalog_by_barcode
                    .values_mut()
                    .find(|item| item.item_id == line.item_id)
                    .context("catalog item not found")?;
                item.stock_on_hand -= line.quantity;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    #[tokio::test]
    async fn cash_checkout_rejects_underpayment_without_clearing_cart() {
        let pos = PosService::with_seed();
        let token = pos.login_with_pin("1234").await.expect("login");
        pos.scan_item(&token, "9780060652937").await.expect("scan");

        let error =
            pos.checkout_cash(&token, 1000, false, 0).await.expect_err("underpayment should fail");
        assert!(error.to_string().contains("tendered amount is less than cart total"));

        let snapshot =
            pos.set_cart_quantity(&token, "bk-102", 1).await.expect("cart remains intact");
        assert_eq!(snapshot.total_cents, 1699);
        assert_eq!(snapshot.items.len(), 1);
    }

    #[tokio::test]
    async fn scanned_items_cannot_exceed_available_stock() {
        let pos = PosService::with_seed();
        let token = pos.login_with_pin("1234").await.expect("login");
        for _ in 0..10 {
            pos.scan_item(&token, "9780060652937").await.expect("scan within stock");
        }

        let error =
            pos.scan_item(&token, "9780060652937").await.expect_err("extra scan should fail");
        assert!(error.to_string().contains("not enough stock on hand"));
    }

    #[tokio::test]
    async fn iou_checkout_deducts_stock() {
        let pos = PosService::with_seed();
        let first = pos.login_with_pin("1234").await.expect("login");
        pos.scan_item(&first, "9780060652937").await.expect("scan");

        let receipt = pos.checkout_iou(&first, "John Doe", 0).await.expect("iou checkout");
        assert_eq!(receipt.outcome, PosPaymentOutcome::UnpaidIou);
        assert_eq!(receipt.total_cents, 1699);

        let second = pos.login_with_pin("1234").await.expect("login");
        for _ in 0..9 {
            pos.scan_item(&second, "9780060652937").await.expect("remaining stock");
        }
        let error = pos
            .scan_item(&second, "9780060652937")
            .await
            .expect_err("stock should be reduced after IOU");
        assert!(error.to_string().contains("not enough stock on hand"));
    }

    #[tokio::test]
    async fn external_card_checkout_applies_discount() {
        let pos = PosService::with_seed();
        let token = pos.login_with_pin("1234").await.expect("login");
        pos.scan_item(&token, "9780060652937").await.expect("scan");

        let receipt =
            pos.checkout_external_card(&token, "square-discount", 170).await.expect("checkout");
        assert_eq!(receipt.outcome, PosPaymentOutcome::Paid);
        assert_eq!(receipt.total_cents, 1529);
        assert_eq!(receipt.discount_cents, 170);
    }

    // --- Property-based tests: discounted_total ---

    #[quickcheck]
    fn discounted_total_result_plus_discount_equals_original(total: i64, discount: i64) -> bool {
        if total <= 0 || discount < 0 || discount >= total {
            return PosService::discounted_total(total, discount).is_err();
        }
        let result = PosService::discounted_total(total, discount).unwrap();
        result + discount == total
    }

    #[quickcheck]
    fn discounted_total_result_is_always_positive(total: u32, discount: u32) -> bool {
        let total = total as i64 + 1; // ensure > 0
        let discount = (discount as i64) % total; // ensure < total
        PosService::discounted_total(total, discount).unwrap() > 0
    }

    #[quickcheck]
    fn discounted_total_rejects_non_positive_total(total: i64) -> bool {
        if total <= 0 { PosService::discounted_total(total, 0).is_err() } else { true }
    }

    #[quickcheck]
    fn discounted_total_rejects_negative_discount(discount: i64) -> bool {
        if discount < 0 { PosService::discounted_total(1000, discount).is_err() } else { true }
    }

    #[quickcheck]
    fn discounted_total_rejects_discount_gte_total(total: u16) -> bool {
        let total = (total as i64).max(1);
        PosService::discounted_total(total, total).is_err()
            && PosService::discounted_total(total, total + 1).is_err()
    }

    // --- Property-based tests: checkout session ---

    #[quickcheck]
    fn checkout_session_total_equals_sales_plus_donation(sales: u32, donation: u16) -> bool {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let svc = StorefrontService::new();
            let sales = (sales as i64).max(1);
            let donation = donation as i64;
            match svc
                .create_checkout_session(
                    "t".to_string(),
                    sales,
                    0,
                    0,
                    donation,
                    "e@e.com".to_string(),
                )
                .await
            {
                Ok(session) => session.total_cents == sales + donation,
                Err(_) => true, // validation rejections are fine
            }
        })
    }

    #[quickcheck]
    fn checkout_session_rejects_non_positive_sales(sales: i64) -> bool {
        if sales > 0 {
            return true;
        }
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let svc = StorefrontService::new();
            svc.create_checkout_session("t".to_string(), sales, 0, 0, 0, "e@e.com".to_string())
                .await
                .is_err()
        })
    }

    #[quickcheck]
    fn checkout_session_ids_are_sequential(n: u8) -> bool {
        let n = (n % 20) as usize;
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let svc = StorefrontService::new();
            let mut ids = Vec::new();
            for _ in 0..n {
                let session = svc
                    .create_checkout_session("t".to_string(), 100, 0, 0, 0, "e@e.com".to_string())
                    .await
                    .unwrap();
                ids.push(session.session_id);
            }
            for (i, id) in ids.iter().enumerate() {
                if *id != format!("chk-{i}") {
                    return false;
                }
            }
            true
        })
    }

    #[quickcheck]
    fn webhook_finalize_duplicate_ref_returns_duplicate(ref_a: String) -> bool {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let svc = StorefrontService::new();
            let session = svc
                .create_checkout_session("t".to_string(), 100, 0, 0, 0, "e@e.com".to_string())
                .await
                .unwrap();
            let first = svc.finalize_webhook(&ref_a, &session.session_id).await.unwrap();
            let second = svc.finalize_webhook(&ref_a, &session.session_id).await.unwrap();
            first.status == WebhookFinalizeStatus::Processed
                && second.status == WebhookFinalizeStatus::Duplicate
        })
    }

    // --- Property-based tests: ISBN normalization ---

    #[quickcheck]
    fn isbn_normalization_strips_non_digits(raw: String) -> bool {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let admin = AdminService::with_local_defaults();
            match admin.lookup_isbn(&raw).await {
                Ok(meta) => meta.isbn.chars().all(|c| c.is_ascii_digit()),
                Err(_) => true, // unknown ISBN is fine
            }
        })
    }

    #[quickcheck]
    fn isbn_normalization_is_idempotent(raw: String) -> bool {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let admin = AdminService::with_local_defaults();
            match admin.lookup_isbn(&raw).await {
                Ok(first) => {
                    let second = admin.lookup_isbn(&first.isbn).await.unwrap();
                    first.isbn == second.isbn
                }
                Err(_) => true, // unknown ISBN is fine
            }
        })
    }

    // --- Property-based tests: inventory ---

    #[quickcheck]
    fn receive_inventory_increases_on_hand(qty_a: u8, qty_b: u8) -> bool {
        let qty_a = (qty_a as i64 % 50).max(1);
        let qty_b = (qty_b as i64 % 50).max(1);
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let admin = AdminService::with_local_defaults();
            let r1 = admin.receive_inventory("church-a", "978TEST", qty_a).await.unwrap();
            let r2 = admin.receive_inventory("church-a", "978TEST", qty_b).await.unwrap();
            r1.on_hand == qty_a && r2.on_hand == qty_a + qty_b
        })
    }

    #[quickcheck]
    fn receive_inventory_rejects_non_positive(qty: i64) -> bool {
        if qty > 0 {
            return true;
        }
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let admin = AdminService::with_local_defaults();
            admin.receive_inventory("church-a", "978TEST", qty).await.is_err()
        })
    }

    #[quickcheck]
    fn adjust_inventory_rejects_zero_delta() -> bool {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let admin = AdminService::with_local_defaults();
            admin.receive_inventory("church-a", "978ADJ", 10).await.unwrap();
            admin.adjust_inventory("church-a", "978ADJ", 0, "test").await.is_err()
        })
    }

    // --- Property-based tests: report summary ---

    #[quickcheck]
    fn report_sales_by_payment_has_no_duplicates(n: u8) -> bool {
        let n = (n % 10) as usize;
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let admin = AdminService::with_local_defaults();
            let methods = [PaymentMethod::Cash, PaymentMethod::ExternalCard, PaymentMethod::Iou];
            let date = NaiveDateTime::new(
                chrono::NaiveDate::from_ymd_opt(2026, 3, 12).unwrap(),
                chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            );
            for i in 0..n {
                admin
                    .record_sales_event(SalesEvent {
                        tenant_id: "church-a".to_string(),
                        payment_method: methods[i % methods.len()],
                        sales_cents: 100,
                        donations_cents: 0,
                        cogs_cents: 50,
                        occurred_at: date,
                    })
                    .await;
            }
            let summary = admin.report_summary_range("church-a", None, None).await;
            let keys: Vec<PaymentMethod> =
                summary.sales_by_payment.iter().map(|(k, _)| *k).collect();
            let unique: std::collections::HashSet<PaymentMethod> = keys.iter().copied().collect();
            keys.len() == unique.len()
        })
    }

    #[quickcheck]
    fn report_gross_profit_equals_sales_minus_cogs(sales: u16, cogs: u16) -> bool {
        let sales = sales as i64;
        let cogs = cogs as i64;
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        rt.block_on(async {
            let admin = AdminService::with_local_defaults();
            admin
                .record_sales_event(SalesEvent {
                    tenant_id: "church-a".to_string(),
                    payment_method: PaymentMethod::Cash,
                    sales_cents: sales,
                    donations_cents: 0,
                    cogs_cents: cogs,
                    occurred_at: NaiveDateTime::new(
                        chrono::NaiveDate::from_ymd_opt(2026, 3, 12).unwrap(),
                        chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                    ),
                })
                .await;
            let summary = admin.report_summary_range("church-a", None, None).await;
            summary.gross_profit_cents == sales - cogs
        })
    }
}
