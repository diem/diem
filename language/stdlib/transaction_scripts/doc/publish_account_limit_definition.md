
<a name="SCRIPT"></a>

# Script `publish_account_limit_definition.move`

### Table of Contents

-  [Function `publish_account_limit_definition`](#SCRIPT_publish_account_limit_definition)



<a name="SCRIPT_publish_account_limit_definition"></a>

## Function `publish_account_limit_definition`

Publishes an unrestricted
<code>LimitsDefintion&lt;CoinType&gt;</code> under
<code>account</code>.
Will abort if a resource with the same type already exists under
<code>account</code>.
No windows will point to this limit at the time it is published.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_publish_account_limit_definition">publish_account_limit_definition</a>&lt;CoinType&gt;(account: &signer)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_publish_account_limit_definition">publish_account_limit_definition</a>&lt;CoinType&gt;(account: &signer) {
    <a href="../../modules/doc/AccountLimits.md#0x1_AccountLimits_publish_unrestricted_limits">AccountLimits::publish_unrestricted_limits</a>&lt;CoinType&gt;(account);
}
</code></pre>



</details>
