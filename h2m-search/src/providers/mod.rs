//! Built-in search provider implementations.
//!
//! Each provider is gated behind a Cargo feature so downstream binaries can
//! opt into only the backends they need. All providers share the same public
//! contract: `async fn search(&self, &SearchQuery) -> Result<SearchResponse,
//! SearchError>`.

mod common;

#[cfg(feature = "brave")]
pub mod brave;
#[cfg(feature = "duckduckgo")]
pub mod duckduckgo;
#[cfg(feature = "searxng")]
pub mod searxng;
#[cfg(feature = "tavily")]
pub mod tavily;
#[cfg(feature = "wikipedia")]
pub mod wikipedia;

#[cfg(feature = "brave")]
pub use brave::Brave;
#[cfg(feature = "duckduckgo")]
pub use duckduckgo::DuckDuckGo;
#[cfg(feature = "searxng")]
pub use searxng::SearXNG;
#[cfg(feature = "tavily")]
pub use tavily::Tavily;
#[cfg(feature = "wikipedia")]
pub use wikipedia::Wikipedia;
