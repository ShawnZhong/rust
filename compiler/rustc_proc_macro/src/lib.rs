// verus-explorer shim. Upstream `rustc_proc_macro` is a second build of
// `library/proc_macro/src/lib.rs` so the rustc binary has its own copy
// independent of whatever proc-macro crates dlopen. We don't dlopen — we
// register clients in-process — so we want a single `TokenStream` type shared
// between the bridge server (rustc) and the bridge client
// (`verus_builtin_macros`). Re-exporting the sysroot `proc_macro` gets us
// exactly that via type-identity preservation through `pub use`.

#![allow(internal_features)]
#![feature(proc_macro_internals)]

extern crate proc_macro;

pub use proc_macro::*;

// `pub use proc_macro::*;` should re-export the `bridge` submodule (it's
// `pub mod bridge;` in the source even though `#[doc(hidden)]`), but be
// explicit so `rustc_proc_macro::bridge::...` paths used elsewhere in the
// compiler tree resolve unambiguously.
pub use proc_macro::bridge;
