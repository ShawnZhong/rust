// verus-explorer: in-process override for proc-macro crate contents.
//
// wasm32-unknown-unknown has no dlopen, so rustc's normal proc-macro loader
// (which dlsyms `_rustc_proc_macro_decls_*` from a host .so) can't work. Hosts
// running rustc-in-wasm register a `&'static [ProcMacro]` slice under a crate
// name; two paths consume the registry:
//
// 1. `register_crate` in `creader.rs` — when the rmeta header says the crate
//    is a proc-macro crate, it reads the slice from here instead of calling
//    `dlsym_proc_macros`.
//
// 2. `rustc_resolve::build_reduced_graph::get_macro_by_def_id` — when the
//    compiled macro belongs to a registered crate, it swaps the compiled
//    `SyntaxExtensionKind` for one wrapping the matching client. This path
//    handles crates whose rmeta is *not* a proc-macro crate (host rustc can't
//    emit `--crate-type=proc-macro` for wasm32, so we ship a regular-library
//    shim rmeta with empty `pub macro` stubs); the registered kind is what
//    actually gets expanded at use sites.

use std::sync::OnceLock;
use std::sync::RwLock;

use rustc_data_structures::fx::FxHashMap;
use rustc_expand::base::SyntaxExtensionKind;
use rustc_expand::proc_macro::{AttrProcMacro, BangProcMacro, DeriveProcMacro};
use rustc_proc_macro::bridge::client::ProcMacro;
use std::sync::Arc;

static REGISTRY: OnceLock<RwLock<FxHashMap<&'static str, &'static [ProcMacro]>>> = OnceLock::new();

fn registry() -> &'static RwLock<FxHashMap<&'static str, &'static [ProcMacro]>> {
    REGISTRY.get_or_init(|| RwLock::new(FxHashMap::default()))
}

/// Register a proc-macro slice for a crate so that rustc's creader resolves
/// it to these in-process descriptors instead of dlopen-ing a host dylib.
/// `crate_name` must match the emitted crate name (i.e. what user code writes
/// in `extern crate <name>;`).
pub fn register(crate_name: &'static str, macros: &'static [ProcMacro]) {
    registry().write().unwrap().insert(crate_name, macros);
}

pub(crate) fn lookup(crate_name: &str) -> Option<&'static [ProcMacro]> {
    registry().read().unwrap().get(crate_name).copied()
}

/// Build a `SyntaxExtensionKind` for the registered proc-macro with name
/// `macro_name` in crate `crate_name`, or `None` if no such entry is
/// registered. Used by `rustc_resolve` to replace the empty-body stub kind
/// from the shim rmeta with a real Bang/Attr/Derive wrapper that runs the
/// registered client through the normal proc-macro bridge.
pub fn lookup_syntax_extension_kind(
    crate_name: &str,
    macro_name: &str,
) -> Option<SyntaxExtensionKind> {
    let slice = lookup(crate_name)?;
    let pm = slice.iter().find(|p| p.name() == macro_name)?;
    Some(match *pm {
        ProcMacro::Bang { client, .. } => {
            SyntaxExtensionKind::Bang(Arc::new(BangProcMacro { client }))
        }
        ProcMacro::Attr { client, .. } => {
            SyntaxExtensionKind::Attr(Arc::new(AttrProcMacro { client }))
        }
        ProcMacro::CustomDerive { client, .. } => {
            SyntaxExtensionKind::Derive(Arc::new(DeriveProcMacro { client }))
        }
    })
}
