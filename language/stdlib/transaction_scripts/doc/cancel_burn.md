
<a name="SCRIPT"></a>

# Script `cancel_burn.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(account: &signer, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Token&gt;(account: &signer, preburn_address: address) {
    <a href="../../modules/doc/LibraAccount.md#0x0_LibraAccount_cancel_burn">LibraAccount::cancel_burn</a>&lt;Token&gt;(account, preburn_address)
}
</code></pre>



</details>
