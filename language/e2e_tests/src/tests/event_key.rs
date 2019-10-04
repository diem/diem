use crate::{account::Account, executor::test_all_genesis};
use types::account_config::AccountEvent;

#[test]
fn stable_event_key() {
    // create a FakeExecutor with a genesis from file
    test_all_genesis(|executor| {
        let genesis_account = Account::new_association();
        let genesis = executor
            .read_account_resource(&genesis_account)
            .expect("sender must exist");
        assert_eq!(
            genesis.sent_events().key(),
            &AccountEvent::sent_event_key(genesis_account.address())
        );
        assert_eq!(
            genesis.received_events().key(),
            &AccountEvent::received_event_key(genesis_account.address())
        );
    });
}
