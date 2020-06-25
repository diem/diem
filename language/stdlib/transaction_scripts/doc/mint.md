
<a name="SCRIPT"></a>

# Script `mint.move`

### Table of Contents

-  [Function `mint`](#SCRIPT_mint)



<a name="SCRIPT_mint"></a>

## Function `mint`

Create
<code>amount</code> coins for
<code>payee</code>.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_mint">mint</a>&lt;Token&gt;(account: &signer, payee: address, amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_mint">mint</a>&lt;Token&gt;(account: &signer, payee: address, amount: u64) {
  <b>assert</b>(<a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_exists_at">LibraAccount::exists_at</a>(payee), 8000971);
  <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_mint_to_address">LibraAccount::mint_to_address</a>&lt;Token&gt;(account, payee, amount);
}
</code></pre>



</details>
