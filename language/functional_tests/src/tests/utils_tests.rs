use crate::utils::parse_input;

#[test]
fn parse_input_no_transactions() {
    parse_input("").unwrap_err();
}

#[test]
fn parse_input_no_transactions_with_config() {
    parse_input("//! no-verify").unwrap_err();
}

#[rustfmt::skip]
#[test]
fn parse_input_nothing_before_first_empty_transaction() {
    parse_input(r"
        //! new-transaction
        main() {}
    ").unwrap();
}

#[rustfmt::skip]
#[test]
fn parse_input_config_before_first_empty_transaction() {
    parse_input(r"
        //! no-execute
        //! new-transaction
        main() {}
    ").unwrap_err();
}

#[rustfmt::skip]
#[test]
fn parse_input_empty_transaction() {
    parse_input(r"
        main() {}
        
        //! new-transaction

        //! new-transaction
        main() {}
    ").unwrap_err();
}

#[rustfmt::skip]
#[test]
fn parse_input_empty_transaction_with_config() {
    parse_input(r"
        main() {}

        //! new-transaction
        //! sender: default

        //! new-transaction
        main() {}
    ").unwrap_err();
}
