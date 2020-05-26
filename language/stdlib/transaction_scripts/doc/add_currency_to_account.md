
<a name="SCRIPT"></a>

# Script `add_currency_to_account.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Currency&gt;(account: &signer) {
    <a href="../../modules/doc/libra_account.md#0x0_LibraAccount_add_currency">LibraAccount::add_currency</a>&lt;Currency&gt;(account);
}
</code></pre>



</details>
