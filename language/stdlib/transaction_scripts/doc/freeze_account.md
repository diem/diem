
<a name="SCRIPT"></a>

# Script `freeze_account.move`

### Table of Contents

-  [Function `freeze_account`](#SCRIPT_freeze_account)



<a name="SCRIPT_freeze_account"></a>

## Function `freeze_account`

Freeze account
<code>address</code>. Initiator must be authorized.
<code>sliding_nonce</code> is a unique nonce for operation, see sliding_nonce.move for details.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_freeze_account">freeze_account</a>(account: &signer, sliding_nonce: u64, to_freeze_account: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_freeze_account">freeze_account</a>(account: &signer, sliding_nonce: u64, to_freeze_account: address) {
    <a href="../../modules/doc/SlidingNonce.md#0x1_SlidingNonce_record_nonce_or_abort">SlidingNonce::record_nonce_or_abort</a>(account, sliding_nonce);
    <a href="../../modules/doc/AccountFreezing.md#0x1_AccountFreezing_freeze_account">AccountFreezing::freeze_account</a>(account, to_freeze_account);
}
</code></pre>



</details>
