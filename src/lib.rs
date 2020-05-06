// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! # MaidSafe Utilities
//!
//! Rust utility functions provided by `MaidSafe`.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
    html_favicon_url = "https://maidsafe.net/img/favicon.ico",
    test(attr(forbid(warnings)))
)]
// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(
    arithmetic_overflow,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types,
    warnings
)]
#![deny(
    bad_style,
    deprecated,
    improper_ctypes,
    missing_docs,
    non_shorthand_field_patterns,
    overflowing_literals,
    stable_features,
    unconditional_recursion,
    unknown_lints,
    unsafe_code,
    unused,
    unused_allocation,
    unused_attributes,
    unused_comparisons,
    unused_features,
    unused_parens,
    while_true
)]
#![warn(
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
#![allow(
    box_pointers,
    missing_copy_implementations,
    missing_debug_implementations,
    variant_size_differences,
    // Note: allowing unused_imports because rust 2018 seems confused about the use
    // of `#[cfg_attr(test, macro_use)]` instead of plain `#[macro_use]` to restrict
    // the use of macros to test profiles.
    // See `extern crate log as loggers`.
    unused_imports,
    // TODO: we need this because of rust-typemap.
    // Stop allowing this warning when this PR gets accepted upstream:
    // https://github.com/reem/rust-typemap/pull/44
    where_clauses_object_safety,
    clippy::needless_doctest_main
)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log as logger;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate unwrap;

/// Utilities related to event-subsetting.
pub mod event_sender;
/// Allows initialising the `env_logger` with a standard message format.
pub mod log;
mod log_or_panic;
mod seeded_rng;
/// Functions for serialisation and deserialisation
pub mod serialisation;
/// Utilities related to threading.
pub mod thread;

pub use crate::seeded_rng::SeededRng;
