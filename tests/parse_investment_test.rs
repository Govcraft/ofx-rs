use ofx_rs::parse;

#[test]
fn parse_investment_statement_from_fixture() {
    let input = include_str!("fixtures/invstmtrs.ofx");
    let doc = parse(input).expect("should parse investment statement");

    // Signon should be present and successful
    assert!(doc.signon().status().is_success());

    // Banking should NOT be present (this file only has investment data)
    assert!(doc.banking().is_none());

    // Investment message set should be present
    let investment = doc.investment().expect("investment should be present");
    let responses = investment.statement_responses();
    assert_eq!(responses.len(), 1);

    // Transaction wrapper
    let wrapper = &responses[0];
    assert_eq!(wrapper.transaction_uid(), "1001");
    assert!(wrapper.status().is_success());

    // Investment statement response
    let stmt = wrapper.response().expect("response should be present");
    assert_eq!(stmt.currency_default().as_str(), "USD");

    // Investment account
    let acct = stmt.investment_account();
    assert_eq!(acct.broker_id(), "121099999");
    assert_eq!(acct.account_id().as_str(), "999988");

    // Transaction list -- only INVBANKTRAN/STMTTRN should be parsed
    let txn_list = stmt
        .transaction_list()
        .expect("transaction list should be present");
    assert_eq!(
        txn_list.len(),
        1,
        "only the INVBANKTRAN/STMTTRN should be extracted; BUYSTOCK should be skipped"
    );

    // Verify the single banking transaction
    let txn = &txn_list.transactions()[0];
    assert_eq!(txn.fit_id().as_str(), "12345");
    assert_eq!(
        txn.amount().as_decimal(),
        rust_decimal::Decimal::new(100000, 2)
    );
    assert_eq!(
        txn.transaction_type(),
        ofx_rs::types::TransactionType::Credit
    );
    assert_eq!(txn.name(), Some("Customer deposit"));
    assert_eq!(txn.memo(), Some("Your check #1034"));
}
