
<a name="SCRIPT"></a>

# Script `preburn.move`

### Table of Contents

-  [Function `preburn`](#SCRIPT_preburn)



<a name="SCRIPT_preburn"></a>

## Function `preburn`

Preburn
<code>amount</code>
<code>Token</code>s from
<code>account</code>.
This will only succeed if
<code>account</code> already has a published
<code>Preburn&lt;Token&gt;</code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_preburn">preburn</a>&lt;Token&gt;(account: &signer, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_preburn">preburn</a>&lt;Token&gt;(account: &signer, amount: u64) {
    <b>let</b> withdraw_cap = <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_extract_withdraw_capability">LibraAccount::extract_withdraw_capability</a>(account);
    <a href="../../modules/doc/Libra.md#0x1_Libra_preburn_to">Libra::preburn_to</a>&lt;Token&gt;(account, <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_withdraw_from">LibraAccount::withdraw_from</a>(&withdraw_cap, amount));
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_restore_withdraw_capability">LibraAccount::restore_withdraw_capability</a>(withdraw_cap);
}
</code></pre>



</details>
