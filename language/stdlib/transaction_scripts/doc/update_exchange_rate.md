
<a name="SCRIPT"></a>

# Script `update_exchange_rate.move`

### Table of Contents

-  [Function `main`](#SCRIPT_main)



<a name="SCRIPT_main"></a>

## Function `main`



<pre><code><b>public</b> <b>fun</b> <a href="#SCRIPT_main">main</a>&lt;unknown#0&gt;(new_exchange_rate_denominator: u64, new_exchange_rate_numerator: u64)
</code></pre>



<details>
<summary>Implementation</summary>


<pre><code><b>fun</b> <a href="#SCRIPT_main">main</a>&lt;Currency&gt;(new_exchange_rate_denominator: u64, new_exchange_rate_numerator: u64) {
    <b>let</b> rate = <a href="../../modules/doc/fixedpoint32.md#0x0_FixedPoint32_create_from_rational">FixedPoint32::create_from_rational</a>(
        new_exchange_rate_denominator,
        new_exchange_rate_numerator,
    );
    <a href="../../modules/doc/libra.md#0x0_Libra_update_lbr_exchange_rate">Libra::update_lbr_exchange_rate</a>&lt;Currency&gt;(rate);
}
</code></pre>



</details>
