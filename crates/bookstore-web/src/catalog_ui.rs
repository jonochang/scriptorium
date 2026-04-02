use bookstore_data::runtime::RuntimeProduct;

use crate::ui::html_escape;

pub fn stock_hint(quantity_on_hand: i64) -> (String, &'static str) {
    match quantity_on_hand {
        i64::MIN..=-1 => ("Out of stock".to_string(), "stock-badge stock-badge--danger"),
        0 => ("Out of stock".to_string(), "stock-badge stock-badge--danger"),
        1..=3 => (
            format!("Only {quantity_on_hand} left"),
            "stock-badge stock-badge--warning",
        ),
        _ => ("In stock".to_string(), "stock-badge stock-badge--success"),
    }
}

pub fn display_title(book: &RuntimeProduct) -> String {
    if book.public_title.trim().is_empty() {
        book.title.clone()
    } else {
        book.public_title.clone()
    }
}

pub fn display_author(book: &RuntimeProduct) -> String {
    if book.public_author.trim().is_empty() {
        book.author.clone()
    } else {
        book.public_author.clone()
    }
}

pub fn book_blurb(book: &RuntimeProduct) -> String {
    if !book.description.trim().is_empty() {
        return book.description.clone();
    }
    if !book.public_description.trim().is_empty() {
        return book.public_description.clone();
    }
    "Selected for parish browsing, gifting, and easy recommendation after services.".to_string()
}

pub fn book_publisher(book: &RuntimeProduct) -> String {
    if !book.publisher.trim().is_empty() {
        return book.publisher.clone();
    }
    if !book.public_publisher.trim().is_empty() {
        return book.public_publisher.clone();
    }
    "Parish House".to_string()
}

pub fn book_binding(book: &RuntimeProduct) -> String {
    if book.binding.trim().is_empty() { "Softcover".to_string() } else { book.binding.clone() }
}

pub fn book_pages(book: &RuntimeProduct) -> String {
    if book.pages.trim().is_empty() { "Parish shelf edition".to_string() } else { book.pages.clone() }
}

pub fn book_isbn(book: &RuntimeProduct) -> String {
    if book.isbn.trim().is_empty() { "9781920000000".to_string() } else { book.isbn.clone() }
}

pub fn format_money(cents: i64) -> String {
    format!("${}.{:02}", cents / 100, (cents % 100).abs())
}

pub fn filter_books(
    books: Vec<RuntimeProduct>,
    query: Option<&str>,
    category: Option<&str>,
) -> Vec<RuntimeProduct> {
    let query = query.unwrap_or("").trim().to_ascii_lowercase();
    let category = category.unwrap_or("").trim().to_ascii_lowercase();
    if query.is_empty() {
        if category.is_empty() || category == "all" {
            return books;
        }
        return books
            .into_iter()
            .filter(|book| book.category.to_ascii_lowercase() == category)
            .collect();
    }
    books
        .into_iter()
        .filter(|book| {
            let matches_query = display_title(book).to_ascii_lowercase().contains(&query)
                || display_author(book).to_ascii_lowercase().contains(&query)
                || book.isbn.to_ascii_lowercase().contains(&query);
            let matches_category = category.is_empty()
                || category == "all"
                || book.category.to_ascii_lowercase() == category;
            matches_query && matches_category
        })
        .collect()
}

pub fn catalog_categories(books: &[RuntimeProduct]) -> Vec<String> {
    let mut categories = books.iter().map(|book| book.category.clone()).collect::<Vec<_>>();
    categories.sort();
    categories.dedup();
    categories
}

pub fn render_catalog_category_chips(
    categories: &[String],
    query: Option<&str>,
    active_category: Option<&str>,
    filtered_books: &[RuntimeProduct],
) -> String {
    let active = active_category.unwrap_or("All");
    let query = query.unwrap_or("").trim();
    std::iter::once("All".to_string())
        .chain(categories.iter().cloned())
        .map(|category| {
            let href = if query.is_empty() {
                format!("/catalog?category={}", urlencoding::encode(&category))
            } else {
                format!(
                    "/catalog?q={}&category={}",
                    urlencoding::encode(query),
                    urlencoding::encode(&category)
                )
            };
            let is_active = category.eq_ignore_ascii_case(active);
            let count = if category == "All" {
                filtered_books.len()
            } else {
                filtered_books
                    .iter()
                    .filter(|book| book.category.eq_ignore_ascii_case(&category))
                    .count()
            };
            format!(
                "<a class=\"category-chip{}\" href=\"{}\">{} <span>{}</span></a>",
                if is_active { " category-chip--active" } else { "" },
                href,
                html_escape(&category),
                count
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

pub fn render_catalog_cards(books: Vec<RuntimeProduct>) -> String {
    if books.is_empty() {
        return "<div class=\"catalog-empty\">No books matched that search.</div>".to_string();
    }
    let items = books
        .into_iter()
        .map(|book| {
            let (stock_label, stock_class) = stock_hint(book.quantity_on_hand);
            let is_out_of_stock = book.quantity_on_hand <= 0;
            let add_button = if is_out_of_stock {
                String::new()
            } else {
                format!(
                    r#"<button class="primary-button primary-button--sm" type="button" data-add-book-id="{}" data-add-book-title="{}" data-add-book-author="{}" data-add-book-price-cents="{}" data-feedback-target="catalog-feedback">Add</button>"#,
                    html_escape(&book.product_id),
                    html_escape(&display_title(&book)),
                    html_escape(&display_author(&book)),
                    book.retail_cents,
                )
            };
            format!(
                r#"<article class="catalog-card">
  <a class="catalog-card__link" href="/catalog/items/{book_id}" aria-label="View {title}">
    <div class="catalog-cover"><span class="{stock_class}">{stock_label}</span></div>
  </a>
  <div class="catalog-card__body">
    <div class="catalog-kicker">{category}</div>
    <a href="/catalog/items/{book_id}" style="text-decoration:none"><h2 class="catalog-title" style="font-size:16px;line-height:1.3;margin-bottom:4px;cursor:pointer">{title}</h2></a>
    <p class="catalog-meta" style="font-size:13px;margin-bottom:8px">{author}</p>
    <p class="catalog-note">{blurb}</p>
    <div style="display:flex;align-items:center;gap:8px">
      <span class="catalog-price">{price}</span>
      {add_button}
    </div>
    <div style="margin-top:10px"><a class="ghost-link ghost-link--ink ghost-link--mini" href="/catalog/items/{book_id}">View details</a></div>
  </div>
</article>"#,
                title = html_escape(&display_title(&book)),
                author = html_escape(&display_author(&book)),
                category = html_escape(&book.category),
                price = format_money(book.retail_cents),
                book_id = html_escape(&book.product_id),
                stock_label = stock_label,
                stock_class = stock_class,
                blurb = html_escape(&book_blurb(&book)),
                add_button = add_button,
            )
        })
        .collect::<Vec<_>>()
        .join("");
    format!(r#"<div class="catalog-grid">{items}</div>"#)
}

pub fn render_catalog_pagination(
    current_page: usize,
    total_pages: usize,
    query: Option<&str>,
    category: Option<&str>,
) -> String {
    if total_pages <= 1 {
        return String::new();
    }
    let mut items = Vec::new();
    for page in 1..=total_pages {
        let mut params = vec![format!("page={page}")];
        if let Some(q) = query.filter(|value| !value.trim().is_empty()) {
            params.push(format!("q={}", urlencoding::encode(q)));
        }
        if let Some(category) = category.filter(|value| !value.trim().is_empty()) {
            params.push(format!("category={}", urlencoding::encode(category)));
        }
        items.push(format!(
            "<a class=\"pagination-link{}\" href=\"/catalog?{}\">{}</a>",
            if page == current_page { " pagination-link--active" } else { "" },
            params.join("&"),
            page
        ));
    }
    format!(
        "<div class=\"pagination\"><span class=\"helper-copy helper-copy--flush\">Page {} of {}</span><div class=\"pagination-links\">{}</div></div>",
        current_page,
        total_pages,
        items.join("")
    )
}
