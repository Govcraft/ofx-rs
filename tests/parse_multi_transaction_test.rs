use ofx_rs::parse;
use ofx_rs::types::{BalanceType, TransactionType};

#[test]
fn parse_multi_transaction_statement() {
    let input = include_str!("fixtures/multi_transaction_statement.ofx");
    let doc = parse(input).unwrap();

    // Signon
    assert!(doc.signon().status().is_success());
    assert_eq!(doc.signon().fi_org(), Some("TestBank"));
    assert_eq!(doc.signon().fi_id(), Some("5678"));

    // Banking message set
    let banking = doc.banking().expect("banking should be present");
    assert_eq!(banking.statement_responses().len(), 1);

    let wrapper = &banking.statement_responses()[0];
    assert!(wrapper.status().is_success());
    assert_eq!(wrapper.transaction_uid(), "3001");

    let stmt = wrapper.response().expect("response should be present");
    assert_eq!(stmt.currency_default().as_str(), "USD");
    assert_eq!(stmt.bank_account().bank_id().as_str(), "111222333");
    assert_eq!(stmt.bank_account().account_id().as_str(), "444555666");

    // Transaction list with 5 transactions
    let txn_list = stmt
        .transaction_list()
        .expect("transaction list should be present");
    assert_eq!(txn_list.len(), 5);

    // Transaction 1: payroll credit
    let txn1 = &txn_list.transactions()[0];
    assert_eq!(txn1.transaction_type(), TransactionType::Credit);
    assert_eq!(txn1.fit_id().as_str(), "MTX001");
    assert_eq!(txn1.name(), Some("PAYROLL DEPOSIT"));
    assert_eq!(txn1.memo(), Some("Bi-weekly payroll"));
    assert_eq!(
        txn1.amount().as_decimal(),
        rust_decimal::Decimal::new(500000, 2)
    );

    // Transaction 2: rent with check number
    let txn2 = &txn_list.transactions()[1];
    assert_eq!(txn2.transaction_type(), TransactionType::Debit);
    assert_eq!(txn2.fit_id().as_str(), "MTX002");
    assert_eq!(txn2.check_number().unwrap().as_str(), "2001");
    assert!(txn2.amount().is_negative());

    // Transaction 3: gas station with SIC code
    let txn3 = &txn_list.transactions()[2];
    assert_eq!(txn3.fit_id().as_str(), "MTX003");
    assert_eq!(txn3.sic(), Some(5541));
    assert_eq!(
        txn3.amount().as_decimal(),
        rust_decimal::Decimal::new(-4567, 2)
    );

    // Transaction 4: online store with DTUSER, REFNUM
    let txn4 = &txn_list.transactions()[3];
    assert_eq!(txn4.fit_id().as_str(), "MTX004");
    assert!(txn4.date_user().is_some());
    assert_eq!(txn4.reference_number(), Some("ORD-12345"));
    assert_eq!(txn4.memo(), Some("Electronics purchase"));

    // Transaction 5: correction transaction
    let txn5 = &txn_list.transactions()[4];
    assert_eq!(txn5.transaction_type(), TransactionType::Credit);
    assert_eq!(txn5.fit_id().as_str(), "MTX005");
    assert_eq!(txn5.correction_id().unwrap().as_str(), "MTX003");
    assert_eq!(
        txn5.correction_action().unwrap(),
        ofx_rs::types::CorrectionAction::Replace
    );

    // Ledger balance
    let ledger = stmt.ledger_balance().expect("ledger balance should be present");
    assert_eq!(
        ledger.amount().as_decimal(),
        rust_decimal::Decimal::new(330433, 2)
    );

    // Available balance
    let avail = stmt
        .available_balance()
        .expect("available balance should be present");
    assert_eq!(
        avail.amount().as_decimal(),
        rust_decimal::Decimal::new(320433, 2)
    );

    // Balance list
    let bal_list = stmt.balance_list();
    assert_eq!(bal_list.len(), 2);

    assert_eq!(bal_list[0].name(), "MinBal");
    assert_eq!(bal_list[0].description(), "Minimum balance this period");
    assert_eq!(bal_list[0].kind(), BalanceType::Dollar);
    assert!(bal_list[0].as_of().is_some());

    assert_eq!(bal_list[1].name(), "RewardPts");
    assert_eq!(bal_list[1].kind(), BalanceType::Number);
    assert!(bal_list[1].as_of().is_none());
}
