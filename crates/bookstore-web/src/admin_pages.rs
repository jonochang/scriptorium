use askama::Template;
use bookstore_app::AdminAuthSession;

use crate::ui::orders_table_placeholder;
use crate::views::admin::{
    AdminDashboardTemplate, AdminIntakeTemplate, AdminLoginTemplate, AdminOrdersTemplate,
};

pub fn admin_login_shell_html(next: &str, message: Option<&str>) -> String {
    AdminLoginTemplate::new(next, message).render().expect("admin login template should render")
}

pub fn admin_dashboard_shell_html(session: &AdminAuthSession) -> String {
    AdminDashboardTemplate::new(session, orders_table_placeholder("No orders loaded yet."))
        .render()
        .expect("admin dashboard template should render")
}

pub fn admin_orders_shell_html(session: &AdminAuthSession) -> String {
    AdminOrdersTemplate::new(session, orders_table_placeholder("No orders loaded yet."))
        .render()
        .expect("admin orders template should render")
}

pub fn admin_intake_shell_html(session: &AdminAuthSession, intake_script: &'static str) -> String {
    AdminIntakeTemplate::new(session, intake_script)
        .render()
        .expect("admin intake template should render")
}
