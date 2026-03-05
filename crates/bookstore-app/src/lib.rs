use std::sync::Arc;

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
