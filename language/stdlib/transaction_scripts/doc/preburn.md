
<a name="SCRIPT"></a>

# Script `preburn.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`

Preburn
<code>amount</code>
<code>Token</code>s from
<code>account</code>.
This will only succeed if
<code>account</code> already has a published
<code>Preburn&lt;Token&gt;</code> resource.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(account: &signer, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Token&gt;(account: &signer, amount: u64) {
    <a href="../../modules/doc/libra.md#0x0_Libra_preburn_to">Libra::preburn_to</a>&lt;Token&gt;(account, <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_withdraw_from_sender">LibraAccount::withdraw_from_sender</a>(amount))
}
</code></pre>



</details>
