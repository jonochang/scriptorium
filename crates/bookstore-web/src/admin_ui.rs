pub fn admin_dashboard_script() -> &'static str {
    r#"<script>
const adminSession = window.SCRIPTORIUM_ADMIN_SESSION || {};
let adminToken = adminSession.token || '';
let adminTenant = adminSession.tenantId || '';
let adminOrders = [];
let adminSnapshot = { summary: null, products: [], categories: [], vendors: [], orders: [], journal: [] };
let orderFilter = 'All';

const money = (cents) => `$${(Number(cents || 0) / 100).toFixed(2)}`;
const byId = (id) => document.getElementById(id);
const setText = (id, value) => {
  const node = byId(id);
  if (node) node.textContent = value;
};

function setStatus(message, tone = '') {
  const panel = byId('admin-status');
  if (!panel) return;
  panel.textContent = message;
  panel.className = `notice-panel${tone ? ` notice-panel--${tone}` : ''}`;
}

function renderList(containerId, items, emptyMessage, renderer) {
  const node = byId(containerId);
  if (!node) return;
  if (!items.length) {
    node.innerHTML = `<div class="empty-inline">${emptyMessage}</div>`;
    return;
  }
  node.innerHTML = items.map(renderer).join('');
}

function reportQuery() {
  const from = byId('report-from')?.value || '';
  const to = byId('report-to')?.value || '';
  const params = new URLSearchParams({ tenant_id: adminTenant });
  if (from) params.set('from', from);
  if (to) params.set('to', to);
  return params.toString();
}

function normalizeChannel(order) {
  return order.channel === 'Online' ? 'Online' : 'POS';
}

async function fetchJson(url, options = {}) {
  const headers = { ...(options.headers || {}), Authorization: `Bearer ${adminToken}` };
  const res = await fetch(url, { ...options, headers });
  const json = await res.json().catch(() => ({}));
  if (!res.ok) {
    throw new Error(json.message || json.error || `Request failed for ${url}`);
  }
  return json;
}

function orderStatusBadge(order) {
  return order.status === 'Paid'
    ? '<span class="status-badge status-badge--paid">Paid</span>'
    : '<span class="status-badge status-badge--iou">IOU</span>';
}

function renderPaymentBreakdown(summary) {
  const rows = Object.entries(summary?.sales_by_payment || {});
  renderList('admin-payment-breakdown', rows, 'Payment method totals will appear here.', ([method, cents]) => {
    const total = Math.max(1, Number(summary.sales_cents || 0));
    const width = Math.max(8, Math.round((Number(cents || 0) / total) * 100));
    return `<div class="stack-list"><div class="list-row list-row--soft"><div><div class="list-title">${method.replaceAll('_', ' ')}</div><div class="list-meta">Share of report window</div></div><strong>${money(cents)}</strong></div><div class="bar-track"><div class="bar-fill" style="width:${width}%"></div></div></div>`;
  });
  const trend = byId('admin-trend-note');
  if (trend) {
    const paid = Number(summary?.sales_cents || 0) - Number(summary?.donations_cents || 0);
    trend.textContent = paid > 0
      ? `Paid sales are ${money(paid)} for the selected window, with donations contributing ${money(summary?.donations_cents || 0)} on top.`
      : 'No paid sales were recorded in the selected window.';
    trend.className = 'notice-panel notice-panel--success';
  }
}

function renderOrderActions(order) {
  const actions = [
    `<button class="ghost-link ghost-link--ink ghost-link--mini" type="button" onclick="viewOrder('${order.order_id}')">View</button>`,
    `<button class="ghost-link ghost-link--ink ghost-link--mini" type="button" onclick="resendReceipt('${order.order_id}')">Resend</button>`,
  ];
  if (order.status === 'UnpaidIou') {
    actions.push(`<button class="primary-button primary-button--sm" type="button" onclick="markOrderPaid('${order.order_id}')">Mark Paid</button>`);
  } else {
    actions.push('<span class="helper-copy helper-copy--flush">Cleared</span>');
  }
  return `<div class="button-row button-row--compact">${actions.join('')}</div>`;
}

function renderOrders() {
  const filtered = adminOrders.filter((order) => orderFilter === 'All' || (orderFilter === 'IOU' ? order.status === 'UnpaidIou' : normalizeChannel(order) === orderFilter));
  const iouCount = adminOrders.filter((order) => order.status === 'UnpaidIou').length;
  const iouButton = byId('order-filter-iou-label');
  if (iouButton) iouButton.textContent = `IOU (${iouCount})`;
  const node = byId('admin-orders');
  if (!node) return;
  if (!filtered.length) {
    node.innerHTML = '<div class="empty-inline">No orders found for this filter.</div>';
    return;
  }
  node.innerHTML = `<table class="orders-table"><thead><tr><th>Order ID</th><th>Date</th><th>Channel</th><th>Customer</th><th>Status</th><th>Method</th><th>Total</th><th>Actions</th></tr></thead><tbody>${filtered.map((order) => `<tr><td>${order.order_id}</td><td>${order.created_on}</td><td>${order.channel}</td><td>${order.customer_name}</td><td>${orderStatusBadge(order)}</td><td>${order.payment_method}</td><td><strong>${money(order.total_cents)}</strong></td><td>${renderOrderActions(order)}</td></tr>`).join('')}</tbody></table>`;
}

function bindOrderFilters() {
  document.querySelectorAll('[data-order-filter]').forEach((button) => {
    button.onclick = () => {
      orderFilter = button.dataset.orderFilter || 'All';
      document.querySelectorAll('[data-order-filter]').forEach((chip) => chip.classList.remove('filter-chip--active'));
      button.classList.add('filter-chip--active');
      renderOrders();
    };
  });
}

function viewOrder(orderId) {
  const order = adminOrders.find((candidate) => candidate.order_id === orderId);
  if (!order) {
    setStatus(`Order ${orderId} is no longer available.`, 'danger');
    return;
  }
  setStatus(`Viewing ${orderId}: ${order.customer_name} via ${order.payment_method} for ${money(order.total_cents)}.`, 'success');
}

function resendReceipt(orderId) {
  setStatus(`Receipt resend queued for ${orderId}.`, 'success');
}

async function markOrderPaid(orderId) {
  if (!adminToken) {
    setStatus('Sign in first to manage orders.', 'danger');
    return;
  }
  try {
    await fetchJson(`/api/admin/orders/${orderId}/mark-paid?tenant_id=${adminTenant}`, {
      method: 'POST',
      headers: { Origin: window.location.origin },
    });
    setStatus(`Marked ${orderId} paid.`, 'success');
    await refreshAdminData();
  } catch (error) {
    setStatus(error.message, 'danger');
  }
}

function exportSnapshot() {
  if (!adminToken || !adminSnapshot.summary) {
    setStatus('Load dashboard data before exporting.', 'danger');
    return;
  }
  const blob = new Blob([JSON.stringify(adminSnapshot, null, 2)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = `scriptorium-${adminTenant}-dashboard.json`;
  link.click();
  URL.revokeObjectURL(url);
  setStatus(`Exported dashboard snapshot for ${adminTenant}.`, 'success');
}

function reorderTitle(title) {
  setStatus(`Open intake to reorder ${title}.`, 'success');
}

async function refreshAdminData() {
  if (!adminToken) {
    setStatus('Sign in first to load dashboard data.', 'danger');
    return;
  }
  setStatus('Loading dashboard data...');
  try {
    const [summary, products, categories, vendors, orders, journal] = await Promise.all([
      fetchJson(`/api/admin/reports/summary?${reportQuery()}`),
      fetchJson(`/api/admin/products?tenant_id=${adminTenant}`),
      fetchJson(`/api/admin/categories?tenant_id=${adminTenant}`),
      fetchJson(`/api/admin/vendors?tenant_id=${adminTenant}`),
      fetchJson(`/api/admin/orders?tenant_id=${adminTenant}`),
      fetchJson(`/api/admin/inventory/journal?tenant_id=${adminTenant}`),
    ]);
    adminSnapshot = { summary, products, categories: categories.values || [], vendors: vendors.values || [], orders, journal };
    adminOrders = orders;
    const paidPos = orders.filter((order) => normalizeChannel(order) === 'POS' && order.status === 'Paid').reduce((sum, order) => sum + Number(order.total_cents || 0), 0);
    const paidOnline = orders.filter((order) => normalizeChannel(order) === 'Online' && order.status === 'Paid').reduce((sum, order) => sum + Number(order.total_cents || 0), 0);
    const openIous = orders.filter((order) => order.status === 'UnpaidIou');
    setText('metric-today-sales', money(summary.sales_cents));
    setText('metric-pos-revenue', money(paidPos));
    setText('metric-online-revenue', money(paidOnline));
    setText('metric-open-ious', `${openIous.length} open`);
    setText('report-caption', `Showing ${byId('report-from')?.value || 'the start'} to ${byId('report-to')?.value || 'today'}.`);
    renderPaymentBreakdown(summary);
    renderList('admin-products', products, 'No products found for this tenant.', (product) => `<div class="list-row list-row--soft"><div><div class="list-title">${product.title}</div><div class="list-meta">${product.category} · ${product.vendor}</div></div><strong>${money(product.retail_cents)}</strong></div>`);
    renderList('admin-categories', categories.values || [], 'No categories found.', (value) => `<span class="chip">${value}</span>`);
    renderList('admin-vendors', vendors.values || [], 'No vendors found.', (value) => `<span class="chip">${value}</span>`);
    renderOrders();
    renderList('admin-ious', openIous, 'No open IOUs.', (order) => `<div class="list-row list-row--soft"><div><div class="list-title">${order.customer_name}</div><div class="list-meta">${order.order_id} · ${order.created_on}</div></div><div class="button-row button-row--compact"><strong>${money(order.total_cents)}</strong><button class="primary-button primary-button--sm" type="button" onclick="markOrderPaid('${order.order_id}')">Mark Paid</button></div></div>`);
    const lowStock = (products || []).filter((product) => Number(product.quantity_on_hand || 0) <= 3);
    renderList('admin-low-stock', lowStock, 'No low-stock titles right now.', (product) => {
      const onHand = Number(product.quantity_on_hand || 0);
      const badge = onHand <= 0 ? '<span class="status-badge status-badge--iou">Out of stock</span>' : `<span class="status-badge status-badge--iou">${onHand} left</span>`;
      return `<div class="list-row list-row--soft"><div><div class="list-title">${product.title}</div><div class="list-meta">${product.category} · On hand ${onHand}</div></div><div class="button-row button-row--compact">${badge}<button class="ghost-link ghost-link--ink ghost-link--mini" type="button" onclick="reorderTitle('${product.title.replaceAll("'", "&#39;")}')">Prep</button></div></div>`;
    });
    renderList('admin-journal', journal, 'No inventory movement recorded yet.', (entry) => `<div class="list-row list-row--soft"><div><div class="list-title">${entry.isbn}</div><div class="list-meta">${entry.reason}</div></div><strong>${entry.delta > 0 ? `+${entry.delta}` : entry.delta}</strong></div>`);
    setStatus(`Dashboard refreshed for ${adminTenant}.`, 'success');
  } catch (error) {
    setStatus(error.message, 'danger');
  }
}

const refreshButton = byId('admin-refresh');
if (refreshButton) refreshButton.addEventListener('click', refreshAdminData);
const exportButton = byId('admin-export');
if (exportButton) exportButton.addEventListener('click', exportSnapshot);
const exportInlineButton = byId('admin-export-inline');
if (exportInlineButton) exportInlineButton.addEventListener('click', exportSnapshot);
const reportFrom = byId('report-from');
if (reportFrom) reportFrom.addEventListener('change', () => { if (adminToken) refreshAdminData(); });
const reportTo = byId('report-to');
if (reportTo) reportTo.addEventListener('change', () => { if (adminToken) refreshAdminData(); });

window.markOrderPaid = markOrderPaid;
window.reorderTitle = reorderTitle;
window.viewOrder = viewOrder;
window.resendReceipt = resendReceipt;

bindOrderFilters();
if (adminToken && adminTenant) {
  refreshAdminData();
} else {
  setStatus('Admin session missing. Sign in again.', 'danger');
}
</script>"#
}
