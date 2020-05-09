
<a name="SCRIPT"></a>

# Script `update_libra_version.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, major: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(account: &signer, major: u64) {
    <a href="../../modules/doc/libra_version.md#0x0_LibraVersion_set">LibraVersion::set</a>(major, account)
}
</code></pre>



</details>
