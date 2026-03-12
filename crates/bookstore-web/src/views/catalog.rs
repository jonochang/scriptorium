use askama::Template;
use bookstore_domain::Book;

use crate::storefront_ui::{storefront_cart_script, storefront_checkout_script};
use crate::ui::{google_fonts_link, page_header, shared_styles, site_footer, site_nav};

#[derive(Template)]
#[template(path = "catalog/catalog.html")]
pub struct CatalogIndexTemplate {
    pub shared_styles: &'static str,
    pub nav_html: String,
    pub header_html: String,
    pub category_value: String,
    pub search_value: String,
    pub category_chips_html: String,
    pub active_category: String,
    pub item_count: usize,
    pub items_html: String,
    pub pagination_html: String,
    pub footer_html: &'static str,
    pub cart_script: &'static str,
}

impl CatalogIndexTemplate {
    pub fn new(
        category_value: String,
        search_value: String,
        category_chips_html: String,
        active_category: String,
        item_count: usize,
        items_html: String,
        pagination_html: String,
    ) -> Self {
        Self {
            shared_styles: shared_styles(),
            nav_html: site_nav("catalog"),
            header_html: page_header(
                "Storefront",
                "Feed your soul.",
                "Find books for parish reading, gifting, and liturgical practice.",
                &["Parish bookshop", "Curated titles", "Warm, accessible checkout"],
                r#"<a class="ghost-link ghost-link--ink" href="/cart">Cart</a><a class="ghost-link ghost-link--ink" href="/checkout">Checkout</a>"#,
            ),
            category_value,
            search_value,
            category_chips_html,
            active_category,
            item_count,
            items_html,
            pagination_html,
            footer_html: site_footer(),
            cart_script: storefront_cart_script(),
        }
    }
}

#[derive(Template)]
#[template(path = "catalog/product_not_found.html")]
pub struct ProductNotFoundTemplate {
    pub shared_styles: &'static str,
    pub nav_html: String,
    pub header_html: String,
    pub footer_html: &'static str,
}

impl ProductNotFoundTemplate {
    pub fn new() -> Self {
        Self {
            shared_styles: shared_styles(),
            nav_html: site_nav("catalog"),
            header_html: page_header(
                "Product Detail",
                "Title not found",
                "That catalog item is not available in this parish shelf view. Return to browsing and choose another selection.",
                &["404", "Friendly fallback"],
                r#"<a class="ghost-link ghost-link--ink" href="/catalog">Back to catalog</a><a class="ghost-link ghost-link--ink" href="/cart">Open cart</a>"#,
            ),
            footer_html: site_footer(),
        }
    }
}

#[derive(Template)]
#[template(path = "catalog/product_detail.html")]
pub struct ProductDetailTemplate {
    pub shared_styles: &'static str,
    pub nav_html: String,
    pub header_html: String,
    pub footer_html: &'static str,
    pub cart_script: &'static str,
    pub book_title: String,
    pub book_author: String,
    pub book_category: String,
    pub price: String,
    pub stock_label: String,
    pub stock_class: String,
    pub blurb_html: String,
    pub publisher: String,
    pub isbn: String,
    pub binding: String,
    pub pages: String,
    pub book_id: String,
    pub related_books_html: String,
    pub price_cents: i64,
}

impl ProductDetailTemplate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        book: &Book,
        price: String,
        stock_label: String,
        stock_class: String,
        blurb_html: String,
        publisher: String,
        isbn: String,
        binding: String,
        pages: String,
        related_books_html: String,
    ) -> Self {
        Self {
            shared_styles: shared_styles(),
            nav_html: site_nav("catalog"),
            header_html: page_header(
                "Product Detail",
                &book.title,
                &format!("by {}", book.author),
                &["Reader favorite", "Shelf-ready gift"],
                r#"<a class="ghost-link ghost-link--ink" href="/catalog">Back to catalog</a><a class="ghost-link ghost-link--ink" href="/cart">Cart</a>"#,
            ),
            footer_html: site_footer(),
            cart_script: storefront_cart_script(),
            book_title: book.title.clone(),
            book_author: book.author.clone(),
            book_category: book.category.clone(),
            price,
            stock_label,
            stock_class,
            blurb_html,
            publisher,
            isbn,
            binding,
            pages,
            book_id: book.id.clone(),
            related_books_html,
            price_cents: i64::from(book.price_cents),
        }
    }
}

#[derive(Template)]
#[template(path = "catalog/cart.html")]
pub struct CartTemplate {
    pub shared_styles: &'static str,
    pub nav_html: String,
    pub header_html: String,
    pub footer_html: &'static str,
    pub cart_script: &'static str,
    pub recommendations_html: String,
}

impl CartTemplate {
    pub fn new(recommendations_html: String) -> Self {
        Self {
            shared_styles: shared_styles(),
            nav_html: site_nav("cart"),
            header_html: page_header(
                "Cart",
                "Review your basket",
                "Confirm quantities, keep gifting simple, and move smoothly into checkout.",
                &["Gentle checkout", "Parish-friendly copy"],
                r#"<a class="ghost-link ghost-link--ink" href="/catalog">Keep browsing</a><a class="ghost-link ghost-link--ink" href="/checkout">Checkout</a>"#,
            ),
            footer_html: site_footer(),
            cart_script: storefront_cart_script(),
            recommendations_html,
        }
    }
}

#[derive(Template)]
#[template(path = "catalog/checkout.html")]
pub struct CheckoutTemplate {
    pub shared_styles: &'static str,
    pub nav_html: String,
    pub header_html: String,
    pub footer_html: &'static str,
    pub checkout_script: &'static str,
}

impl CheckoutTemplate {
    pub fn new() -> Self {
        Self {
            shared_styles: shared_styles(),
            nav_html: site_nav("checkout"),
            header_html: page_header(
                "Checkout",
                "Finish your order",
                "Confirm your contact details, choose any extra parish support, and place the order with confidence.",
                &["Secure handoff", "Receipt-ready", "Confirmation state"],
                r#"<a class="ghost-link ghost-link--ink" href="/cart">Back to cart</a><a class="ghost-link ghost-link--ink" href="/catalog">Continue shopping</a>"#,
            ),
            footer_html: site_footer(),
            checkout_script: storefront_checkout_script(),
        }
    }
}

pub fn google_fonts() -> &'static str {
    google_fonts_link()
}
