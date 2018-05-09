// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

//! # MaidSafe Utilities
//!
//! Rust utility functions provided by `MaidSafe`.

#![doc(html_logo_url =
           "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
       html_favicon_url = "https://maidsafe.net/img/favicon.ico",
       html_root_url = "https://docs.rs/maidsafe_utilities")]

// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(exceeding_bitshifts, mutable_transmutes, no_mangle_const_items, unknown_crate_types,
          warnings)]
#![deny(bad_style, deprecated, improper_ctypes, missing_docs, non_shorthand_field_patterns,
        overflowing_literals, plugin_as_library, private_no_mangle_fns, private_no_mangle_statics,
        stable_features, unconditional_recursion, unknown_lints, unsafe_code, unused,
        unused_allocation, unused_attributes, unused_comparisons, unused_features, unused_parens,
        while_true)]
#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results)]
#![allow(box_pointers, missing_copy_implementations,
         missing_debug_implementations, variant_size_differences)]

// TODO: Allow `panic_params` until https://github.com/Manishearth/rust-clippy/issues/768
//       is resolved.
#![cfg_attr(all(feature="cargo-clippy", test), allow(panic_params))]

extern crate bincode;
extern crate config_file_handler;
#[macro_use]
extern crate lazy_static;
#[cfg_attr(test, macro_use)]
extern crate log as logger;
extern crate log4rs;
#[macro_use]
extern crate quick_error;
extern crate rand;
extern crate regex;
extern crate serde;
extern crate serde_value;
#[macro_use]
extern crate unwrap;
extern crate url;
extern crate ws;

/// Utilities related to threading.
pub mod thread;
/// Utilities related to event-subsetting.
pub mod event_sender;
/// Allows initialising the `env_logger` with a standard message format.
pub mod log;
mod seeded_rng;
/// Functions for serialisation and deserialisation
pub mod serialisation;

pub use seeded_rng::SeededRng;
