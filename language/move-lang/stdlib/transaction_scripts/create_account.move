fun main(fresh_address: address, initial_amount: u64) {
    0x0::LibraAccount::create_new_account(fresh_address, initial_amount)
}
