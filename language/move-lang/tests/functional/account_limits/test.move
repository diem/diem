//! account: validatorvivian, 10000000, 0, validator
//! account: root, 1000000000, 0, true
//! account: child1, 1000000000, 0, true
//! account: normal, 100000000, 0

//! new-transaction
//! sender: root
//! gas-price: 0
use 0x0::VASP;
use 0x0::LCS;
use 0x0::AccountLimits;
fun main() {
    VASP::apply_for_vasp_root_credential(
        LCS::to_bytes<address>(&0xAAA),
        LCS::to_bytes<address>(&0xBBB),
        LCS::to_bytes<address>(&0xCCC),
    );
    AccountLimits::publish_limits_definition(1000, 1000, 2000, 2000);
}
// check: EXECUTED

//! new-transaction
//! sender: child1
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// Try to use the uncertified limits definition under the VASP root for
// child1
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(10);
    LibraAccount::deposit({{normal}}, lbr);
}
// check: ABORTED
// check: 2

//! new-transaction
//! sender: association
use 0x0::VASP;
use 0x0::Transaction;
use 0x0::Association;
use 0x0::AccountLimits;
fun main() {
    Association::apply_for_privilege<VASP::CreationPrivilege>();
    Association::grant_privilege<VASP::CreationPrivilege>({{association}});
    VASP::grant_vasp({{root}});
    AccountLimits::certify_limits_definition({{root}});
    Transaction::assert(VASP::is_root_vasp({{root}}), 1);
}
// check: EXECUTED

//! new-transaction
//! sender: root
//! gas-price: 0
use 0x0::VASP;
fun main() {
    VASP::allow_child_accounts();
}
// check: EXECUTED

//! new-transaction
//! sender: child1
//! gas-price: 0
use 0x0::VASP;
fun main() {
    VASP::apply_for_child_vasp_credential({{root}})
}
// check: EXECUTED

//! new-transaction
//! sender: root
//! gas-price: 0
use 0x0::VASP;
use 0x0::Transaction;
fun main() {
    VASP::grant_child_account({{child1}});
    Transaction::assert(VASP::is_child_vasp({{child1}}), 0);
}
// check: EXECUTED

///////////////////////////////////////////////////////////////////////////
// tests
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: child1
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// This should now work since it's a valid account.
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(10);
    LibraAccount::deposit({{normal}}, lbr);
}
// check: EXECUTED

//! new-transaction
//! sender: child1
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// Can withdraw from the account
fun main() {
    LibraAccount::pay_from_sender<LBR::T>({{normal}}, 10);
}
// check: EXECUTED

//! new-transaction
//! sender: child1
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// Withdrawal at the edge of limit. But it's all good!
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(980);
    LibraAccount::deposit({{normal}}, lbr);
}
// check: EXECUTED

//! new-transaction
//! sender: child1
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// Over the withdrawal limit so fails.
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(1);
    LibraAccount::deposit({{normal}}, lbr);
}
// check: ABORTED
// check: 11

//! new-transaction
//! sender: child1
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// Over the withdrawal limit so fails.  Make sure it also holds for the
// root account.
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(1);
    LibraAccount::deposit({{normal}}, lbr);
}
// check: ABORTED
// check: 11

//! new-transaction
//! sender: normal
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// VASP-normal interaction: Inflow limit OK, so succeeeds.
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(1000);
    LibraAccount::deposit({{root}}, lbr);
}
// check: EXECUTED


//! new-transaction
//! sender: normal
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// VASP-normal interaction: Inflow limit now exceeded.
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(1000);
    LibraAccount::deposit({{root}}, lbr);
}
// check: ABORTED
// check: 9

//! new-transaction
//! sender: normal
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// VASP-normal interaction: Inflow limit now exceeded. Make sure it also
// holds for the child account.
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(1000);
    LibraAccount::deposit({{root}}, lbr);
}
// check: ABORTED
// check: 9

//! block-prologue
//! proposer: validatorvivian
//! block-time: 2000

//! new-transaction
//! sender: normal
//! expiration-time: 300
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// VASP-normal interaction: Inflow limit now exceeded.
// Time hasn't reset yet.
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(10);
    LibraAccount::deposit({{root}}, lbr);
}
// check: ABORTED
// check: 9

//! block-prologue
//! proposer: validatorvivian
//! block-time: 2001

//! new-transaction
//! sender: normal
//! expiration-time: 300
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// VASP-normal interaction: time has reset, so inflow not exceeded now.
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(10);
    LibraAccount::deposit({{root}}, lbr);
}
// check: EXECUTED

//! new-transaction
//! sender: normal
//! expiration-time: 300
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// max-out inflow
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(990);
    LibraAccount::deposit({{root}}, lbr);
}
// check: EXECUTED

//! block-prologue
//! proposer: validatorvivian
//! block-time: 6003

//! new-transaction
//! sender: normal
//! expiration-time: 700
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// max-out balance
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(1);
    LibraAccount::deposit({{root}}, lbr);
}
// check: ABORTED
// check: 9

//! new-transaction
//! sender: child1
//! expiration-time: 700
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// remove some of the balance
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(200);
    LibraAccount::deposit({{normal}}, lbr);
}
// check: EXECUTED

//! new-transaction
//! sender: normal
//! expiration-time: 700
//! gas-price: 0
use 0x0::LibraAccount;
use 0x0::LBR;
// Now make sure we can deposit. But abort so we don't change state.
fun main() {
    let lbr = LibraAccount::withdraw_from_sender<LBR::T>(100);
    LibraAccount::deposit({{root}}, lbr);
    abort 1099
}
// check: ABORTED
// check: 1099
