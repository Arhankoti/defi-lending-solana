//! Instruction handlers and their associated Anchor account structs.

pub mod cancel_loan_request;
pub mod close_loan;
pub mod create_loan_request;
pub mod fund_loan;
pub mod initialize_market;
pub mod liquidate_loan;
pub mod partial_repay;
pub mod repay_loan;
pub mod update_market;

// Glob re-exports are required so that Anchor-generated client-account modules
// (e.g. `__client_accounts_fund_loan`) are visible at the crate root via `use instructions::*`.
// Handler functions are invoked through their fully-qualified module paths in lib.rs,
// so the ambiguous `handler` names from each module are never used unqualified.
pub use cancel_loan_request::*;
pub use close_loan::*;
pub use create_loan_request::*;
pub use fund_loan::*;
pub use initialize_market::*;
pub use liquidate_loan::*;
pub use partial_repay::*;
pub use repay_loan::*;
pub use update_market::*;
