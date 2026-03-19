use ofx_rs::parse;

#[test]
fn parse_mercury_checking_fixture() {
    let input = include_str!("fixtures/mercury-checking.ofx");
    let doc = parse(input).unwrap();

    // Header: SGML v1.51
    assert_eq!(doc.header().version().major(), 1);
    assert_eq!(doc.header().version().minor(), 5);
    assert_eq!(doc.header().version().patch(), 1);

    // Signon
    assert!(doc.signon().status().is_success());

    // Banking
    let banking = doc.banking().expect("banking message set should be present");
    let wrapper = &banking.statement_responses()[0];
    assert!(wrapper.status().is_success());

    let stmt = wrapper.response().expect("statement response should be present");
    assert_eq!(stmt.currency_default().as_str(), "USD");
    assert_eq!(stmt.bank_account().account_id().as_str(), "202497494038");

    // Should have 674 transactions
    let txn_list = stmt.transaction_list().expect("transaction list should be present");
    assert_eq!(txn_list.len(), 674);

    // Ledger balance
    let ledger = stmt.ledger_balance().expect("ledger balance should be present");
    assert_eq!(
        ledger.amount().as_decimal(),
        rust_decimal::Decimal::new(114986, 2)
    );
}

#[test]
fn parse_ofxtools_stmtrs_fixture() {
    let input = include_str!("fixtures/stmtrs.ofx");
    let doc = parse(input).unwrap();

    // OFX 2.0.0 XML
    assert_eq!(doc.header().version().major(), 2);

    let banking = doc.banking().expect("banking should be present");
    let stmt = banking.statement_responses()[0].response().unwrap();
    assert_eq!(stmt.currency_default().as_str(), "USD");
    assert_eq!(stmt.bank_account().account_id().as_str(), "999988");

    let txn_list = stmt.transaction_list().unwrap();
    assert_eq!(txn_list.len(), 2);
}

#[test]
fn parse_ofxtools_stmtrs_euro_fixture() {
    // stmtrs_euro uses USD as CURDEF but comma decimal separators (European style amounts)
    let input = include_str!("fixtures/stmtrs_euro.ofx");
    let doc = parse(input).unwrap();

    let banking = doc.banking().expect("banking should be present");
    let stmt = banking.statement_responses()[0].response().unwrap();
    assert_eq!(stmt.currency_default().as_str(), "USD");

    let txn_list = stmt.transaction_list().unwrap();
    assert_eq!(txn_list.len(), 2);
}
