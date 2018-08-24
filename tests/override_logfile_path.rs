// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
    html_favicon_url = "https://maidsafe.net/img/favicon.ico",
    test(attr(forbid(warnings)))
)]
// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(
    exceeding_bitshifts,
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
    plugin_as_library,
    private_no_mangle_fns,
    private_no_mangle_statics,
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
    variant_size_differences
)]

extern crate config_file_handler;
#[macro_use]
extern crate log as logger;
extern crate maidsafe_utilities;
#[macro_use]
extern crate unwrap;

use maidsafe_utilities::log;
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::thread;
use std::time::Duration;

#[test]
fn override_logfile_path() {
    const LOG_FILE: &str = "secret-log-file-name.log";

    let mut current_dir = unwrap!(env::current_dir());
    let mut current_bin_dir = unwrap!(config_file_handler::current_bin_dir());

    if current_dir.as_path() != current_bin_dir.as_path() {
        // Try to copy log.toml from the current dir to bin dir so that the config_file_handler
        // can find it
        current_dir.push("sample_log_file/log.toml");
        current_bin_dir.push("log.toml");

        let _ = unwrap!(fs::copy(current_dir, current_bin_dir));
    }

    unwrap!(log::init_with_output_file(false, LOG_FILE));

    error!("SECRET-MESSAGE");

    // Wait for async file writer
    thread::sleep(Duration::from_millis(500));

    let mut log_file_path = unwrap!(config_file_handler::current_bin_dir());
    log_file_path.push(LOG_FILE);

    let mut file = unwrap!(File::open(log_file_path));
    let mut contents = String::new();
    let _ = unwrap!(file.read_to_string(&mut contents));

    assert!(contents.contains("SECRET-MESSAGE"));
}
