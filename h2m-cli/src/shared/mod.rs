//! Reusable argument groups shared across subcommands.
//!
//! Each group is a `#[derive(Args)]` struct that gets `#[command(flatten)]`-ed
//! into a subcommand's top-level arg struct. Centralising them here keeps
//! flag definitions single-sourced so `convert` and `search --scrape` stay
//! in lockstep.

pub(crate) mod content;
pub(crate) mod format;
pub(crate) mod http;

pub(crate) use content::ContentArgs;
pub(crate) use format::{FormatArgs, build_options};
pub(crate) use http::HttpArgs;
