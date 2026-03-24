use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsbnLookupRecord {
    pub isbn: String,
    pub title: String,
    pub author: String,
    pub publisher: String,
    pub description: String,
    pub cover_image_url: Option<String>,
}

#[derive(Clone, Debug)]
pub struct IsbnLookupClient {
    client: reqwest::Client,
    base_url: String,
}

impl IsbnLookupClient {
    pub fn open_library() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: std::env::var("SCRIPTORIUM_ISBN_LOOKUP_BASE_URL")
                .unwrap_or_else(|_| "https://openlibrary.org/api/books".to_string()),
        }
    }

    pub fn with_base_url(base_url: impl Into<String>) -> Self {
        Self { client: reqwest::Client::new(), base_url: base_url.into() }
    }

    pub async fn lookup(&self, isbn: &str) -> anyhow::Result<Option<IsbnLookupRecord>> {
        let normalized = isbn.chars().filter(|ch| ch.is_ascii_digit()).collect::<String>();
        if normalized.is_empty() {
            return Ok(None);
        }
        if let Some(record) = self.lookup_google_books(&normalized).await? {
            return Ok(Some(record));
        }
        self.lookup_open_library(&normalized).await
    }

    async fn lookup_open_library(
        &self,
        normalized: &str,
    ) -> anyhow::Result<Option<IsbnLookupRecord>> {
        let response = self
            .client
            .get(&self.base_url)
            .query(&[
                ("bibkeys", format!("ISBN:{normalized}")),
                ("format", "json".to_string()),
                ("jscmd", "data".to_string()),
            ])
            .send()
            .await?;
        if !response.status().is_success() {
            return Ok(None);
        }
        let body = response.json::<std::collections::HashMap<String, OpenLibraryBook>>().await?;
        let book = match body.get(&format!("ISBN:{normalized}")) {
            Some(book) => book,
            None => return Ok(None),
        };
        let title = book.title.clone().filter(|value| !value.trim().is_empty());
        let author = book
            .authors
            .as_ref()
            .and_then(|authors| authors.first())
            .map(|author| author.name.clone())
            .filter(|value| !value.trim().is_empty());
        let publisher = book
            .publishers
            .as_ref()
            .and_then(|publishers| publishers.first())
            .map(|publisher| publisher.name.clone())
            .unwrap_or_default();
        let description = book
            .description
            .as_ref()
            .and_then(OpenLibraryDescription::as_text)
            .or_else(|| book.subtitle.clone())
            .unwrap_or_default();
        let cover_image_url = book.cover.as_ref().and_then(|cover| {
            cover.large.clone().or_else(|| cover.medium.clone()).or_else(|| cover.small.clone())
        });
        Ok(match (title, author) {
            (Some(title), Some(author)) => Some(IsbnLookupRecord {
                isbn: normalized.to_string(),
                title,
                author,
                publisher,
                description,
                cover_image_url,
            }),
            _ => None,
        })
    }

    async fn lookup_google_books(
        &self,
        normalized: &str,
    ) -> anyhow::Result<Option<IsbnLookupRecord>> {
        let response = self
            .client
            .get("https://www.googleapis.com/books/v1/volumes")
            .query(&[("q", format!("isbn:{normalized}"))])
            .send()
            .await?;
        if !response.status().is_success() {
            return Ok(None);
        }

        let body = response.json::<GoogleBooksResponse>().await?;
        let volume = match body.items.and_then(|mut items| items.drain(..).next()) {
            Some(volume) => volume.volume_info,
            None => return Ok(None),
        };

        let title = volume.title.filter(|value| !value.trim().is_empty());
        let author = volume
            .authors
            .and_then(|authors| authors.into_iter().next())
            .filter(|value| !value.trim().is_empty());
        let publisher = volume.publisher.unwrap_or_default();
        let description = volume
            .description
            .or(volume.subtitle)
            .unwrap_or_default();
        let cover_image_url = volume
            .image_links
            .and_then(|links| links.large.or(links.medium).or(links.small).or(links.thumbnail));

        Ok(match (title, author) {
            (Some(title), Some(author)) => Some(IsbnLookupRecord {
                isbn: normalized.to_string(),
                title,
                author,
                publisher,
                description,
                cover_image_url,
            }),
            _ => None,
        })
    }
}

#[derive(Debug, Deserialize)]
struct OpenLibraryBook {
    title: Option<String>,
    subtitle: Option<String>,
    authors: Option<Vec<OpenLibraryAuthor>>,
    publishers: Option<Vec<OpenLibraryPublisher>>,
    description: Option<OpenLibraryDescription>,
    cover: Option<OpenLibraryCover>,
}

#[derive(Debug, Deserialize)]
struct OpenLibraryAuthor {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OpenLibraryPublisher {
    name: String,
}

#[derive(Debug, Deserialize)]
struct OpenLibraryCover {
    small: Option<String>,
    medium: Option<String>,
    large: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OpenLibraryDescription {
    Text(String),
    Object { value: String },
}

impl OpenLibraryDescription {
    fn as_text(&self) -> Option<String> {
        match self {
            Self::Text(value) | Self::Object { value } if !value.trim().is_empty() => {
                Some(value.clone())
            }
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GoogleBooksResponse {
    items: Option<Vec<GoogleBookItem>>,
}

#[derive(Debug, Deserialize)]
struct GoogleBookItem {
    #[serde(rename = "volumeInfo")]
    volume_info: GoogleVolumeInfo,
}

#[derive(Debug, Deserialize)]
struct GoogleVolumeInfo {
    title: Option<String>,
    subtitle: Option<String>,
    authors: Option<Vec<String>>,
    publisher: Option<String>,
    description: Option<String>,
    #[serde(rename = "imageLinks")]
    image_links: Option<GoogleImageLinks>,
}

#[derive(Debug, Deserialize)]
struct GoogleImageLinks {
    small: Option<String>,
    medium: Option<String>,
    large: Option<String>,
    thumbnail: Option<String>,
}
