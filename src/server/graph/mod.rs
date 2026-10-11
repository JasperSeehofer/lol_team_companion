//! The Draft Graph's server side (M4): the trait table, the rule set, the
//! team's match evidence and the fit ranking.
//!
//! - [`traits`]: defaults file, database copy, in-memory table.
//! - [`rules`]: trait rules and the per-pair relations computed once.
//! - [`evidence`]: pair and matchup records from the team's stored games.
//! - [`fit`]: ranks candidates for the current slot ([`fit::rank`]).

pub mod evidence;
pub mod fit;
pub mod rules;
pub mod traits;
