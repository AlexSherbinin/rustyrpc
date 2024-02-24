#![feature(
    trivial_bounds,
    const_type_name,
    async_closure,
    impl_trait_in_fn_trait_return,
    auto_traits,
    negative_impls,
    if_let_guard,
    let_chains
)]
#![deny(
    warnings,
    clippy::correctness,
    clippy::suspicious,
    clippy::complexity,
    clippy::perf,
    clippy::style,
    clippy::pedantic,
    clippy::restriction,
    clippy::cargo
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::blanket_clippy_restriction_lints,
    clippy::missing_inline_in_public_items,
    clippy::single_char_lifetime_names,
    clippy::implicit_return,
    clippy::pattern_type_mismatch,
    clippy::question_mark_used,
    clippy::shadow_reuse,
    clippy::shadow_same,
    clippy::pub_with_shorthand,
    clippy::absolute_paths,
    clippy::exhaustive_enums,
    clippy::exhaustive_structs,
    clippy::multiple_crate_versions,
    clippy::missing_docs_in_private_items,
    clippy::pub_use,
    clippy::infinite_loop, // Allowed because of bug: https://github.com/rust-lang/rust-clippy/issues/12338
    clippy::unseparated_literal_suffix,
    clippy::self_named_module_files,
    clippy::big_endian_bytes
)]
#![forbid(unreachable_pub, missing_docs)]
#![doc = include_str!("../../README.md")]

extern crate alloc;

/// Provides functionality for client side of RPC.
pub mod client;
/// Errors that may occur while usage of RPC.
pub mod error;
/// Provides abstraction layer against encoding format.
pub mod format;
/// Provides core primitives for RPC protocol.
pub mod protocol;
/// Provides functionality for server side of RPC.
pub mod server;
/// Provides service trait and others.
pub mod service;
/// Provides abstraction layer against transport.
pub mod transport;
