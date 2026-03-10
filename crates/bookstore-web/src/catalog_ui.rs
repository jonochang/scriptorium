use crate::ui::html_escape;

pub fn stock_hint(book_id: &str) -> (&'static str, &'static str) {
    match book_id {
        "bk-104" => ("Only 2 left", "stock-badge stock-badge--warning"),
        "bk-108" => ("Only 3 left", "stock-badge stock-badge--warning"),
        "bk-105" => ("Out of stock", "stock-badge stock-badge--danger"),
        _ => ("In stock", "stock-badge stock-badge--success"),
    }
}

pub fn book_blurb(book_id: &str) -> &'static str {
    match book_id {
        "bk-100" => {
            "A practical invitation to reorder ordinary life around prayer, service, and long obedience."
        }
        "bk-101" => {
            "A theology shelf staple for readers who want doctrine with warmth, confidence, and pastoral clarity."
        }
        "bk-102" => {
            "A steady guide to spiritual disciplines that serves parish reading groups, gifts, and personal devotion alike."
        }
        "bk-103" => {
            "Chesterton's vivid defense of Christian belief, ideal for curious browsers and after-liturgy discussion circles."
        }
        "bk-104" => {
            "A tactile devotional gift that sits well in prayer corners, chrismation baskets, and feast-day giving."
        }
        "bk-105" => {
            "A gentle stationery gift for feast days, hospital visits, and hand-written parish encouragement."
        }
        "bk-106" => {
            "A keepsake icon suited to blessing gifts, patronal feasts, and home prayer spaces."
        }
        "bk-107" => {
            "A travel-sized icon for commuters, students, and anyone building a portable rule of prayer."
        }
        "bk-108" => {
            "A compact icon that brings courage and intercession into gloveboxes, work desks, and prayer corners."
        }
        "bk-109" => {
            "A warm beeswax candle for evening prayers, vigil tables, and quiet household observance."
        }
        "bk-110" => {
            "A fragrant starter set for home blessings, memorial prayers, and gift-table recommendations."
        }
        "bk-900" => {
            "A compact prayer companion for weekday offices, feast preparation, and gift-table recommendations."
        }
        _ => "Selected for parish browsing, gifting, and easy recommendation after services.",
    }
}

pub fn book_publisher(book_id: &str) -> &'static str {
    match book_id {
        "bk-100" => "Zondervan",
        "bk-101" => "IVP",
        "bk-102" => "HarperOne",
        "bk-103" => "Ignatius Press",
        "bk-104" => "Parish Workshop",
        "bk-105" => "Scriptorium Press",
        "bk-106" => "Monastery Press",
        "bk-107" => "Icon Studio",
        "bk-108" => "Pilgrim Workshop",
        "bk-109" => "Church Supplier",
        "bk-110" => "Cathedral Supply",
        "bk-900" => "Parish House",
        _ => "Parish House",
    }
}

pub fn book_binding(book_id: &str) -> &'static str {
    match book_id {
        "bk-104" | "bk-105" | "bk-106" | "bk-107" | "bk-108" | "bk-109" | "bk-110" => "Gift item",
        "bk-900" => "Flexibound",
        _ => "Softcover",
    }
}

pub fn book_pages(book_id: &str) -> &'static str {
    match book_id {
        "bk-100" => "336 pages",
        "bk-101" => "304 pages",
        "bk-102" => "256 pages",
        "bk-103" => "320 pages",
        "bk-104" => "Hand-knotted",
        "bk-105" => "12 cards",
        "bk-106" => "8 x 10 in.",
        "bk-107" => "4 x 6 in.",
        "bk-108" => "3 x 4 in.",
        "bk-109" => "Single taper",
        "bk-110" => "Starter bundle",
        "bk-900" => "192 pages",
        _ => "Parish shelf edition",
    }
}

pub fn book_isbn(book_id: &str) -> &'static str {
    match book_id {
        "bk-100" => "9780310337508",
        "bk-101" => "9780830816507",
        "bk-102" => "9780060628390",
        "bk-103" => "9780898704440",
        "bk-104" => "9781920000104",
        "bk-105" => "9781920000105",
        "bk-106" => "9781920000106",
        "bk-107" => "9781920000107",
        "bk-108" => "9781920000108",
        "bk-109" => "9781920000109",
        "bk-110" => "9781920000110",
        "bk-900" => "9781920000900",
        _ => "9781920000000",
    }
}

pub fn book_cover_symbol(book_id: &str) -> &'static str {
    match book_id {
        "bk-104" | "bk-105" => "🎁",
        "bk-106" | "bk-107" | "bk-108" => "🖼️",
        "bk-109" | "bk-110" | "bk-900" => "🕯️",
        _ => "📚",
    }
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

pub fn render_catalog_cards(books: Vec<bookstore_domain::Book>) -> String {
    if books.is_empty() {
        return "<div class=\"catalog-empty\">No books matched that search.</div>".to_string();
    }
    let items = books
        .into_iter()
        .map(|book| {
            let (stock_label, stock_class) = stock_hint(&book.id);
            format!(
                r#"<article class="catalog-card">
  <a class="catalog-card__link" href="/catalog/items/{book_id}" aria-label="View {title}"></a>
  <div class="catalog-cover"><span class="{stock_class} stock-badge--overlay">{stock_label}</span><span class="catalog-cover__symbol">{cover_symbol}</span></div>
  <div class="catalog-kicker"><span>{category}</span></div>
  <h2 class="catalog-title">{title}</h2>
  <p class="catalog-meta">{author}</p>
  <p class="catalog-note">{blurb}</p>
  <div class="button-row">
    <span class="catalog-price">{price}</span>
    <button class="primary-button primary-button--sm" type="button" data-add-book-id="{book_id}" data-add-book-title="{title_attr}" data-add-book-author="{author_attr}" data-add-book-price-cents="{price_cents}" data-feedback-target="catalog-feedback">Add</button>
    <a class="ghost-link ghost-link--ink" href="/catalog/items/{book_id}">View details</a>
  </div>
</article>"#,
                title = html_escape(&book.title),
                author = html_escape(&book.author),
                category = html_escape(&book.category),
                price = format_money(i64::from(book.price_cents)),
                book_id = html_escape(&book.id),
                title_attr = html_escape(&book.title),
                author_attr = html_escape(&book.author),
                price_cents = i64::from(book.price_cents),
                stock_label = stock_label,
                stock_class = stock_class,
                cover_symbol = book_cover_symbol(&book.id),
                blurb = html_escape(book_blurb(&book.id)),
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
