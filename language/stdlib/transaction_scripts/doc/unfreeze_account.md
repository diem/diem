
<a name="SCRIPT"></a>

# Script `unfreeze_account.move`

### Table of Contents

-  [Function `unfreeze_account`](#SCRIPT_unfreeze_account)



<a name="SCRIPT_unfreeze_account"></a>

## Function `unfreeze_account`

Script for un-freezing account by authorized initiator
sliding_nonce is a unique nonce for operation, see sliding_nonce.move for details


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_unfreeze_account">unfreeze_account</a>(account: &signer, sliding_nonce: u64, to_unfreeze_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_unfreeze_account">unfreeze_account</a>(account: &signer, sliding_nonce: u64, to_unfreeze_account: address) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_unfreeze_account">LibraAccount::unfreeze_account</a>(account,  to_unfreeze_account);
}
</code></pre>



</details>
