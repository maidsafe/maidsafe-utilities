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

use maidsafe_utilities::{log, thread};
use std::sync::mpsc::{self, Sender};
use std::thread::sleep;
use std::time::Duration;
use ws::{Handler, Message, Request, Response};

#[test]
fn web_socket_logging() {
    const MSG_COUNT: usize = 3;
    const MAGIC_VALUE: &str = "magic-value";

    let (tx, rx) = mpsc::channel();

    // Start Log Message Server
    let _thread = thread::named("LogMessageWebServer", move || {
        struct Server {
            tx: Sender<()>,
            ws_tx: ws::Sender,
            count: usize,
        }

        impl Handler for Server {
            fn on_request(&mut self, req: &Request) -> ws::Result<Response> {
                log::validate_web_socket_request(req, Some(MAGIC_VALUE))
            }

            fn on_message(&mut self, msg: Message) -> ws::Result<()> {
                let text = unwrap!(msg.as_text());
                assert!(text.contains(&format!("This is message {}", self.count)[..]));
                self.count += 1;
                if self.count == MSG_COUNT {
                    unwrap!(self.tx.send(()));
                    unwrap!(self.ws_tx.shutdown());
                }

                Ok(())
            }
        }

        unwrap!(ws::listen("127.0.0.1:44444", |ws_tx| Server {
            tx: tx.clone(),
            ws_tx,
            count: 0,
        }));
    });

    // Allow some time for server to start listening.
    sleep(Duration::from_millis(100));

    unwrap!(log::init_to_web_socket(
        "ws://127.0.0.1:44444",
        Some(MAGIC_VALUE.into()),
        false,
        false,
    ));

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

    unwrap!(rx.recv_timeout(Duration::from_secs(10)));
}
