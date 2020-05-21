
<a name="SCRIPT"></a>

# Script `preburn.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(amount: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Token&gt;(amount: u64) {
    <a href="../../modules/doc/libra.md#0x0_Libra_preburn_to_sender">Libra::preburn_to_sender</a>&lt;Token&gt;(<a href="../../modules/doc/libra_account.md#0x0_LibraAccount_withdraw_from_sender">LibraAccount::withdraw_from_sender</a>(amount))
}
</code></pre>



</details>
