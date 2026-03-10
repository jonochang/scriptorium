const TRANSLATIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "en-AU",
        &[
            ("checkout.complete", "Sale Complete"),
            ("admin.intake.title", "Admin Inventory Intake"),
            ("storefront.checkout.title", "Checkout"),
            ("storefront.catalog.title", "Feed your soul."),
            ("admin.dashboard.title", "Good morning, Father Michael"),
        ],
    ),
    (
        "el-GR",
        &[
            ("checkout.complete", "Η πώληση ολοκληρώθηκε"),
            ("admin.intake.title", "Παραλαβή αποθέματος διαχειριστή"),
            ("storefront.checkout.title", "Ταμείο"),
            ("storefront.catalog.title", "Θρέψε την ψυχή σου."),
            ("admin.dashboard.title", "Καλημέρα, πάτερ Μιχαήλ"),
        ],
    ),
];

pub fn lookup<'a>(locale: &str, key: &'a str) -> Cow<'a, str> {
    lookup_exact(locale, key)
        .or_else(|| lookup_exact("en-AU", key))
        .map(Cow::Borrowed)
        .unwrap_or_else(|| Cow::Borrowed(key))
}

fn lookup_exact(locale: &str, key: &str) -> Option<&'static str> {
    TRANSLATIONS.iter().find(|(candidate_locale, _)| *candidate_locale == locale).and_then(
        |(_, entries)| {
            entries.iter().find(|(candidate_key, _)| *candidate_key == key).map(|(_, value)| *value)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::lookup;

    #[test]
    fn falls_back_to_english_for_known_key() {
        assert_eq!(lookup("fr-FR", "checkout.complete").as_ref(), "Sale Complete");
    }

    #[test]
    fn returns_key_for_unknown_translation() {
        assert_eq!(lookup("en-AU", "unknown.key").as_ref(), "unknown.key");
    }
}
use std::borrow::Cow;
