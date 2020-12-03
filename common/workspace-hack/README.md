# diem-workspace-hack

This crate is hack to unify 3rd party dependency crate features in order to
have better compilation caching. Each time cargo is invoked, it computes
features, so across multi invocations it may resolve differently and therefore
find things are cached.

The downside is that all code is compiled with the features needed for any
invocation.

See the
[rustc-workspace-hack](https://github.com/rust-lang/rust/tree/master/src/tools/rustc-workspace-hack)
for further details.
