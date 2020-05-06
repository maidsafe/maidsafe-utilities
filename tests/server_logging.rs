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
    variant_size_differences
)]

#[macro_use]
extern crate log as logger;
#[macro_use]
extern crate unwrap;

use maidsafe_utilities::log::{self, MSG_TERMINATOR};
use maidsafe_utilities::thread;
use std::net::TcpListener;
use std::str;
use std::sync::mpsc;
use std::thread::sleep;
use std::time::Duration;

#[test]
fn server_logging() {
    const MSG_COUNT: usize = 3;

    let (tx, rx) = mpsc::channel();

    // Start Log Message Server
    let _raii_joiner = thread::named("LogMessageServer", move || {
        use std::io::Read;

        let listener = unwrap!(TcpListener::bind("127.0.0.1:55555"));
        unwrap!(tx.send(()));
        let (mut stream, _) = unwrap!(listener.accept());

        let mut log_msgs = Vec::with_capacity(MSG_COUNT);

        let mut read_buf = Vec::with_capacity(1024);
        let mut scratch_buf = [0u8; 1024];
        let mut search_frm_index = 0;

        while log_msgs.len() < MSG_COUNT {
            let bytes_rxd = unwrap!(stream.read(&mut scratch_buf));
            if bytes_rxd == 0 {
                unreachable!("Should not have encountered shutdown yet");
            }

            read_buf.extend_from_slice(&scratch_buf[..bytes_rxd]);

            while read_buf.len() - search_frm_index >= MSG_TERMINATOR.len() {
                if read_buf[search_frm_index..].starts_with(&MSG_TERMINATOR) {
                    log_msgs
                        .push(unwrap!(str::from_utf8(&read_buf[..search_frm_index])).to_owned());
                    read_buf = read_buf.split_off(search_frm_index + MSG_TERMINATOR.len());
                    search_frm_index = 0;
                } else {
                    search_frm_index += 1;
                }
            }
        }

        for it in log_msgs.iter().enumerate() {
            assert!(
                it.1.contains(&format!("This is message {}", it.0)[..]),
                "{} -- {}",
                it.0,
                it.1
            );
            assert!(!it.1.contains('#'));
        }
    });

    unwrap!(rx.recv());

    unwrap!(log::init_to_server("127.0.0.1:55555", true, false));

    info!("This message should not be found by default log level");

    warn!("This is message 0");
    trace!("This message should not be found by default log level");
    warn!("This is message 1");

    // Some interval before the 3rd message to test if server logic above works fine with separate
    // arrival of messages. Without sleep it will usually receive all 3 messages in a single read
    // cycle.
    sleep(Duration::from_millis(500));

    debug!("This message should not be found by default log level");
    error!("This is message 2");
}
