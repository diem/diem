
<a name="SCRIPT"></a>

# Script `allow_currency_for_vasp.move`

### Table of Contents

-  [Function `allow_currency_for_vasp`](#SCRIPT_allow_currency_for_vasp)



<a name="SCRIPT_allow_currency_for_vasp"></a>

## Function `allow_currency_for_vasp`

Adds limits and an accounting window for
<code>CoinType</code> currency to the parent VASP
<code>account</code>.
This transaction will fail if sent from a child account.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_allow_currency_for_vasp">allow_currency_for_vasp</a>&lt;CoinType&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_allow_currency_for_vasp">allow_currency_for_vasp</a>&lt;CoinType&gt;(account: &signer) {
    <b>let</b> _ = <a href="../../modules/doc/VASP.md#0x1_VASP_try_allow_currency">VASP::try_allow_currency</a>&lt;CoinType&gt;(account)
}
</code></pre>



</details>
