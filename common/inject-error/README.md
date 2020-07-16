# inject-error

This crate provides a proc macro to inject non-deterministic anyhow::Errors into functions to help test the robustness of the system.

It's enabled with feature flag `enable-inject-error` and !cfg(test) environment, see `bin/test.rs` for example.

Note `enable-inject-error` should be a feature in the caller crate instead of depending on this crate.
