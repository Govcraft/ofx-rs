use ofx_rs::parse;
use ofx_rs::types::{AccountType, TransactionType};

#[test]
fn parse_bank_statement_from_fixture() {
    let input = include_str!("fixtures/simple_bank_statement.ofx");
    let doc = parse(input).unwrap();

    // Header
    assert_eq!(doc.header().version().major(), 2);
    assert_eq!(doc.header().version().minor(), 2);

    // Signon
    assert!(doc.signon().status().is_success());
    assert_eq!(doc.signon().language(), "ENG");
    assert_eq!(doc.signon().fi_org(), Some("MyBank"));
    assert_eq!(doc.signon().fi_id(), Some("1234"));

    // Banking
    let banking = doc.banking().expect("banking message set should be present");
    assert_eq!(banking.statement_responses().len(), 1);

    let wrapper = &banking.statement_responses()[0];
    assert!(wrapper.status().is_success());
    assert_eq!(wrapper.transaction_uid(), "1001");

    let stmt = wrapper.response().expect("statement response should be present");
    assert_eq!(stmt.currency_default().as_str(), "USD");
    assert_eq!(stmt.bank_account().bank_id().as_str(), "123456789");
    assert_eq!(stmt.bank_account().account_id().as_str(), "987654321");
    assert_eq!(stmt.bank_account().account_type(), AccountType::Checking);

    // Transaction list
    let txn_list = stmt.transaction_list().expect("transaction list should be present");
    assert_eq!(txn_list.len(), 1);

    let txn = &txn_list.transactions()[0];
    assert_eq!(txn.transaction_type(), TransactionType::Debit);
    assert_eq!(txn.fit_id().as_str(), "20230115001");
    assert_eq!(txn.name(), Some("GROCERY STORE"));
    assert_eq!(txn.memo(), Some("Weekly groceries"));
    assert!(txn.amount().is_negative());

    // Ledger balance
    let ledger = stmt.ledger_balance().expect("ledger balance should be present");
    assert_eq!(
        ledger.amount().as_decimal(),
        rust_decimal::Decimal::new(150000, 2)
    );

    // No credit card section
    assert!(doc.credit_card().is_none());
}

#[test]
fn parse_credit_card_statement_from_fixture() {
    let input = include_str!("fixtures/credit_card_statement.ofx");
    let doc = parse(input).unwrap();

    // Signon
    assert!(doc.signon().status().is_success());
    assert_eq!(doc.signon().fi_org(), Some("MyCCIssuer"));

    // No banking section
    assert!(doc.banking().is_none());

    // Credit card section
    let cc = doc.credit_card().expect("credit card message set should be present");
    assert_eq!(cc.statement_responses().len(), 1);

    let wrapper = &cc.statement_responses()[0];
    assert!(wrapper.status().is_success());
    assert_eq!(wrapper.transaction_uid(), "2001");

    let stmt = wrapper.response().expect("cc statement response should be present");
    assert_eq!(stmt.currency_default().as_str(), "USD");
    assert_eq!(stmt.credit_card_account().account_id().as_str(), "4111111111111111");

    let txn_list = stmt.transaction_list().expect("transaction list should be present");
    assert_eq!(txn_list.len(), 1);

    let txn = &txn_list.transactions()[0];
    assert_eq!(txn.transaction_type(), TransactionType::Debit);
    assert_eq!(txn.name(), Some("STREAMING SERVICE"));
    assert_eq!(txn.memo(), Some("Monthly subscription"));

    // Ledger balance should be negative (credit card)
    let ledger = stmt.ledger_balance().expect("ledger balance should be present");
    assert!(ledger.amount().is_negative());
}

#[test]
fn parse_sgml_header_statement_from_fixture() {
    let input = include_str!("fixtures/sgml_header_statement.ofx");
    let doc = parse(input).unwrap();

    // Header should show OFX 1.0.2
    assert_eq!(doc.header().version().major(), 1);
    assert_eq!(doc.header().version().minor(), 0);
    assert_eq!(doc.header().version().patch(), 2);

    // Banking
    let banking = doc.banking().expect("banking should be present");
    let wrapper = &banking.statement_responses()[0];
    let stmt = wrapper.response().unwrap();

    assert_eq!(stmt.bank_account().account_type(), AccountType::Savings);
    assert_eq!(stmt.bank_account().bank_id().as_str(), "111222333");

    let txn = &stmt.transaction_list().unwrap().transactions()[0];
    assert_eq!(txn.transaction_type(), TransactionType::Deposit);
    assert_eq!(txn.name(), Some("PAYROLL DEPOSIT"));
    assert!(!txn.amount().is_negative());
}
