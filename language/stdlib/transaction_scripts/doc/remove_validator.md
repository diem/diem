
<a name="SCRIPT"></a>

# Script `remove_validator.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>(new_validator: address)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>(new_validator: address) {
  <a href="../../modules/doc/libra_system.md#0x0_LibraSystem_remove_validator">LibraSystem::remove_validator</a>(new_validator)
}
</code></pre>



</details>
