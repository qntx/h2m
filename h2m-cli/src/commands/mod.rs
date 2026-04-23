//! Subcommand implementations.
//!
//! Each subcommand lives in its own module and owns both its `Args` struct
//! and its `async fn run(&self)` method. The [`Command`] enum defined in
//! [`crate::cli`] wraps them and dispatches via [`Command::run`].

pub(crate) mod convert;
#[cfg(feature = "search")]
pub(crate) mod search;

pub(crate) use convert::ConvertArgs;
#[cfg(feature = "search")]
pub(crate) use search::SearchArgs;

use crate::cli::Command;
use crate::error::CliError;
use crate::output;

impl Command {
    /// Executes the selected subcommand.
    ///
    /// # Errors
    ///
    /// Propagates any error returned by the inner subcommand.
    pub(crate) async fn run(&self) -> Result<(), CliError> {
        match self {
            Self::Convert(args) => args.run().await,
            #[cfg(feature = "search")]
            Self::Search(args) => args.run().await,
        }
    }

    /// Reports an error with a format consistent with the command's output
    /// mode (JSON object vs. plain stderr).
    pub(crate) fn report_error(&self, err: &CliError) {
        if self.wants_json_error() {
            #[cfg(feature = "search")]
            if let CliError::Search(search_err) = err {
                output::emit_search_json_error(search_err);
                return;
            }
            output::emit_json_error(&err.to_string(), err.url());
        } else {
            eprintln!("error: {err}");
        }
    }

    /// Returns `true` if the command wants machine-readable JSON errors.
    const fn wants_json_error(&self) -> bool {
        match self {
            Self::Convert(args) => args.json,
            #[cfg(feature = "search")]
            Self::Search(args) => args.json || args.scrape,
        }
    }
}
