//! OFX aggregate types -- composite structures built from primitive types.

pub mod available_balance;
pub mod balance;
pub mod bank_account;
pub mod cc_statement_response;
pub mod credit_card_account;
pub mod currency;
pub mod error;
pub mod ledger_balance;
pub mod payee;
pub mod statement_response;
pub mod statement_transaction;
pub mod transaction_list;
pub mod transaction_wrapper;

pub use available_balance::AvailableBalance;
pub use balance::Balance;
pub use bank_account::BankAccount;
pub use cc_statement_response::CcStatementResponse;
pub use credit_card_account::CreditCardAccount;
pub use currency::CurrencyInfo;
pub use error::AggregateError;
pub use ledger_balance::LedgerBalance;
pub use payee::Payee;
pub use statement_response::StatementResponse;
pub use statement_transaction::{StatementTransaction, StatementTransactionBuilder};
pub use status::Status;
pub use transaction_list::TransactionList;
pub use transaction_wrapper::TransactionWrapper;

mod status;
