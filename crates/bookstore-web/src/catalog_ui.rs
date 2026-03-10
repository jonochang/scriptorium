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
