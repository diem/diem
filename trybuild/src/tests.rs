use std::path::Path;

macro_rules! test_normalize {
    ($name:ident $original:literal $expected:literal) => {
        #[test]
        fn $name() {
            let context = super::Context {
                krate: "trybuild000",
                source_dir: Path::new("/git/trybuild/test_suite"),
                workspace: Path::new("/git/trybuild"),
            };
            let original = $original.to_owned().into_bytes();
            let variations = super::diagnostics(original, context);
            assert_eq!(variations.preferred(), $expected);
        }
    };
}

test_normalize! {test_basic "
error: `self` parameter is only allowed in associated functions
  --> /git/trybuild/test_suite/ui/error.rs:11:23
   |
11 | async fn bad_endpoint(self) -> Result<HttpResponseOkObject<()>, HttpError> {
   |                       ^^^^ not semantically valid as function parameter

error: aborting due to 2 previous errors

For more information about this error, try `rustc --explain E0401`.
error: could not compile `trybuild-tests`.

To learn more, run the command again with --verbose.
" "
error: `self` parameter is only allowed in associated functions
  --> $DIR/error.rs:11:23
   |
11 | async fn bad_endpoint(self) -> Result<HttpResponseOkObject<()>, HttpError> {
   |                       ^^^^ not semantically valid as function parameter
"}

test_normalize! {test_dir_backslash "
error[E0277]: the trait bound `QueryParams: serde::de::Deserialize<'de>` is not satisfied
   --> \\git\\trybuild\\test_suite\\ui\\error.rs:22:61
" "
error[E0277]: the trait bound `QueryParams: serde::de::Deserialize<'de>` is not satisfied
   --> $DIR/error.rs:22:61
"}

test_normalize! {test_rust_lib "
error[E0599]: no method named `quote_into_iter` found for struct `std::net::Ipv4Addr` in the current scope
  --> /git/trybuild/test_suite/ui/not-repeatable.rs:6:13
   |
6  |     let _ = quote! { #(#ip)* };
   |             ^^^^^^^^^^^^^^^^^^ method not found in `std::net::Ipv4Addr`
   |
  ::: /rustlib/src/rust/src/libstd/net/ip.rs:83:1
  ::: /rustlib/src/rust/library/std/src/net/ip.rs:83:1
   |
83 | pub struct Ipv4Addr {
   | -------------------
   | |
   | doesn't satisfy `std::net::Ipv4Addr: quote::to_tokens::ToTokens`
" "
error[E0599]: no method named `quote_into_iter` found for struct `std::net::Ipv4Addr` in the current scope
  --> $DIR/not-repeatable.rs:6:13
   |
6  |     let _ = quote! { #(#ip)* };
   |             ^^^^^^^^^^^^^^^^^^ method not found in `std::net::Ipv4Addr`
   |
  ::: $RUST/src/libstd/net/ip.rs
  ::: $RUST/std/src/net/ip.rs
   |
   | pub struct Ipv4Addr {
   | -------------------
   | |
   | doesn't satisfy `std::net::Ipv4Addr: quote::to_tokens::ToTokens`
"}

test_normalize! {test_type_dir_backslash "
error[E0277]: `*mut _` cannot be shared between threads safely
   --> /git/trybuild/test_suite/ui/compile-fail-3.rs:7:5
    |
7   |     thread::spawn(|| {
    |     ^^^^^^^^^^^^^ `*mut _` cannot be shared between threads safely
    |
    = help: the trait `std::marker::Sync` is not implemented for `*mut _`
    = note: required because of the requirements on the impl of `std::marker::Send` for `&*mut _`
    = note: required because it appears within the type `[closure@/git/trybuild/test_suite/ui/compile-fail-3.rs:7:19: 9:6 x:&*mut _]`
" "
error[E0277]: `*mut _` cannot be shared between threads safely
   --> $DIR/compile-fail-3.rs:7:5
    |
7   |     thread::spawn(|| {
    |     ^^^^^^^^^^^^^ `*mut _` cannot be shared between threads safely
    |
    = help: the trait `std::marker::Sync` is not implemented for `*mut _`
    = note: required because of the requirements on the impl of `std::marker::Send` for `&*mut _`
    = note: required because it appears within the type `[closure@$DIR/ui/compile-fail-3.rs:7:19: 9:6 x:&*mut _]`
"}
