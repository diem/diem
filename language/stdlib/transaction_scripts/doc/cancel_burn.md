
<a name="SCRIPT"></a>

# Script `cancel_burn.move`

### Table of Contents

-  [Function `cancel_burn`](#SCRIPT_cancel_burn)



<a name="SCRIPT_cancel_burn"></a>

## Function `cancel_burn`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_cancel_burn">cancel_burn</a>&lt;Token&gt;(account: &signer, preburn_address: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_cancel_burn">cancel_burn</a>&lt;Token&gt;(account: &signer, preburn_address: address) {
    <a href="../../modules/doc/LibraAccount.md#0x1_LibraAccount_cancel_burn">LibraAccount::cancel_burn</a>&lt;Token&gt;(account, preburn_address)
}
</code></pre>



</details>
