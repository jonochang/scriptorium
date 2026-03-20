use bookstore_app::seed::SeedData;
use crate::ui::html_escape;

pub fn stock_hint(seed: &SeedData, book_id: &str) -> (String, &'static str) {
    let hint = seed.catalog.find_book(book_id).map(|b| b.stock_hint.as_str()).unwrap_or("in_stock");
    match hint {
        "low_2" => ("Only 2 left".to_string(), "stock-badge stock-badge--warning"),
        "low_3" => ("Only 3 left".to_string(), "stock-badge stock-badge--warning"),
        "out_of_stock" => ("Out of stock".to_string(), "stock-badge stock-badge--danger"),
        _ => ("In stock".to_string(), "stock-badge stock-badge--success"),
    }
}

pub fn book_blurb(seed: &SeedData, book_id: &str) -> String {
    seed.catalog.find_book(book_id)
        .map(|b| b.blurb.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Selected for parish browsing, gifting, and easy recommendation after services.")
        .to_string()
}

pub fn book_publisher(seed: &SeedData, book_id: &str) -> String {
    seed.catalog.find_book(book_id)
        .map(|b| b.publisher.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Parish House")
        .to_string()
}

pub fn book_binding(seed: &SeedData, book_id: &str) -> String {
    seed.catalog.find_book(book_id)
        .map(|b| b.binding.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Softcover")
        .to_string()
}

pub fn book_pages(seed: &SeedData, book_id: &str) -> String {
    seed.catalog.find_book(book_id)
        .map(|b| b.pages.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("Parish shelf edition")
        .to_string()
}

pub fn book_isbn(seed: &SeedData, book_id: &str) -> String {
    seed.catalog.find_book(book_id)
        .map(|b| b.isbn.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("9781920000000")
        .to_string()
}

pub fn book_cover_symbol(seed: &SeedData, book_id: &str) -> String {
    seed.catalog.find_book(book_id)
        .map(|b| b.cover_symbol.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("📚")
        .to_string()
}

pub fn format_money(cents: i64) -> String {
    format!("${}.{:02}", cents / 100, (cents % 100).abs())
}

pub fn filter_books(
    books: Vec<bookstore_domain::Book>,
    query: Option<&str>,
    category: Option<&str>,
) -> Vec<bookstore_domain::Book> {
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
            let matches_query = book.title.to_ascii_lowercase().contains(&query)
                || book.author.to_ascii_lowercase().contains(&query);
            let matches_category = category.is_empty()
                || category == "all"
                || book.category.to_ascii_lowercase() == category;
            matches_query && matches_category
        })
        .collect()
}

pub fn catalog_categories(books: &[bookstore_domain::Book]) -> Vec<String> {
    let mut categories = books.iter().map(|book| book.category.clone()).collect::<Vec<_>>();
    categories.sort();
    categories.dedup();
    categories
}

pub fn render_catalog_category_chips(
    categories: &[String],
    query: Option<&str>,
    active_category: Option<&str>,
    filtered_books: &[bookstore_domain::Book],
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

pub fn render_catalog_cards(seed: &SeedData, books: Vec<bookstore_domain::Book>) -> String {
    if books.is_empty() {
        return "<div class=\"catalog-empty\">No books matched that search.</div>".to_string();
    }
    let items = books
        .into_iter()
        .map(|book| {
            let (stock_label, stock_class) = stock_hint(seed, &book.id);
            let is_out_of_stock = stock_label == "Out of stock";
            let add_button = if is_out_of_stock {
                String::new()
            } else {
                format!(
                    r#"<button class="primary-button primary-button--sm" type="button" data-add-book-id="{}" data-add-book-title="{}" data-add-book-author="{}" data-add-book-price-cents="{}" data-feedback-target="catalog-feedback">Add</button>"#,
                    html_escape(&book.id),
                    html_escape(&book.title),
                    html_escape(&book.author),
                    book.price_cents,
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
                title = html_escape(&book.title),
                author = html_escape(&book.author),
                category = html_escape(&book.category),
                price = format_money(book.price_cents),
                book_id = html_escape(&book.id),
                stock_label = stock_label,
                stock_class = stock_class,
                blurb = html_escape(&book_blurb(seed, &book.id)),
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
