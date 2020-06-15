
<a name="SCRIPT"></a>

# Script `some_script.move`

### Table of Contents

-  [Function `some`](#SCRIPT_some)



<a name="SCRIPT_some"></a>

## Function `some`

This script does really nothing but just aborts.


<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_some">some</a>&lt;T&gt;(_account: &signer)
</code></pre>



##### Implementation


<pre><code><b>fun</b> <a href="#SCRIPT_some">some</a>&lt;T&gt;(_account: &signer) {
    <b>abort</b> 1
}
</code></pre>
