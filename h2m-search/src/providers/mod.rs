//! Built-in search provider implementations.
//!
//! Each provider is gated behind a Cargo feature so downstream binaries can
//! opt into only the backends they need. All providers share the same public
//! contract: `async fn search(&self, &SearchQuery) -> Result<SearchResponse,
//! SearchError>`.
//!
//! Provider types are re-exported at the crate root; consumers should depend
//! on those re-exports rather than reaching through the module path.

pub(crate) mod common;

#[cfg(feature = "brave")]
mod brave;
#[cfg(feature = "duckduckgo")]
mod duckduckgo;
#[cfg(feature = "searxng")]
mod searxng;
#[cfg(feature = "tavily")]
mod tavily;
#[cfg(feature = "wikipedia")]
mod wikipedia;

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
