// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

//! These functions can initialise the env_logger for output to stderr only, or to a file and
//! stderr.
//!
//! An example of a log message is:
//!
//! ```
//! # fn main() { /*
//! W 19:33:49.245434 <main> [example:src/main.rs:50] Warning level message.
//! ^        ^          ^        ^          ^                    ^
//! |    timestamp      | top-level module  |                 message
//! |                   |                   |
//! |              thread name       file and line no.
//! |
//! level (E, W, I, D, or T for error, warn, info, debug or trace respectively)
//! # */}
//! ```
//!
//! Logging of the thread name is enabled or disabled via the `show_thread_name` parameter.  If
//! enabled, and the thread executing the log statement is unnamed, the thread name is shown as
//! `???`.
//!
//! The functions can safely be called multiple times concurrently.
//!
//! #Examples
//!
//! ```
//! #[macro_use]
//! extern crate log;
//! #[macro_use]
//! extern crate maidsafe_utilities;
//! use std::thread;
//! use maidsafe_utilities::thread::RaiiThreadJoiner;
//!
//! fn main() {
//!     maidsafe_utilities::log::init(true);
//!
//!     warn!("A warning");
//!
//!     let unnamed = thread::spawn(move || info!("Message in unnamed thread"));
//!     let _ = unnamed.join();
//!
//!     let _named = RaiiThreadJoiner::new(thread!("Worker",
//!                      move || error!("Message in named thread")));
//!
//!     // W 12:24:07.064746 <main> [example:src/main.rs:11] A warning
//!     // I 12:24:07.065746 ??? [example:src/main.rs:13] Message in unnamed thread
//!     // E 12:24:07.065746 Worker [example:src/main.rs:16] Message in named thread
//! }
//! ```

#![allow(unused)]
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;

use env_logger::LogBuilder;
use logger::{LogLevel, LogRecord};
use time;

static INITIALISE_LOGGER: ::std::sync::Once = ::std::sync::ONCE_INIT;

/// Initialises the env_logger for output to stderr.
///
/// For further details, see the [module docs](index.html).
pub fn init(show_thread_name: bool) {
    INITIALISE_LOGGER.call_once(|| {
        build(move |record: &LogRecord| format_record(show_thread_name, record));
    });
}

/// Initialises the env_logger for output to a file and to stderr.
///
/// This function will create the logfile at `file_path` if it does not exist, and will truncate it
/// if it does.  For further details, see the [module docs](index.html).
///
/// #Examples
///
/// ```
/// #[macro_use]
/// extern crate log;
/// extern crate maidsafe_utilities;
///
/// fn main() {
///     assert!(maidsafe_utilities::log::init_to_file(true, "target/test.log").is_ok());
///     error!("An error!");
///     assert_eq!(maidsafe_utilities::log::init_to_file(true, "target/test.log").unwrap_err(),
///         "Logger already initialised.".to_owned());
///
///     // E 22:38:05.499016 <main> [example:main.rs:7] An error!
/// }
/// ```
pub fn init_to_file<P: AsRef<Path>>(show_thread_name: bool, file_path: P) -> Result<(), String> {
    let mut result = Err("Logger already initialised.".to_owned());
    let filepath: PathBuf = file_path.as_ref().to_owned();
    INITIALISE_LOGGER.call_once(|| {
        // Check the file can be created in the initialisation phase rather than waiting until the
        // first call to a logging macro.  If the file can't be created, fall back to stderr logging
        // only and return an `Err`.
        let _ = fs::remove_file(&filepath);
        match File::create(&filepath) {
            Ok(_) => {
                result = Ok(());
                let format = move |record: &LogRecord| {
                    let mut log_message = format_record(show_thread_name, record);
                    log_message.push('\n');
                    let mut logfile = unwrap_result!(OpenOptions::new()
                                                         .write(true)
                                                         .create(true)
                                                         .append(true)
                                                         .open(&filepath));
                    unwrap_result!(logfile.write_all(&log_message.clone().into_bytes()[..]));
                    let _ = log_message.pop();
                    log_message
                };
                build(format);
            }
            Err(error) => {
                result = Err(format!("Failed to create logfile at {} - {}",
                                     filepath.display(),
                                     error));
                build(move |record: &LogRecord| format_record(show_thread_name, record));
            }
        };
    });
    result
}

fn format_record(show_thread_name: bool, record: &LogRecord) -> String {
    let now = time::now();
    let mut thread_name = "".to_owned();
    if show_thread_name {
        thread_name = thread::current().name().unwrap_or("???").to_owned();
        thread_name.push_str(" ");
    }

    let file_name = if let Some(file_name_slice) = Path::new(record.location().file())
                                                       .file_name()
                                                       .and_then(|elt| elt.to_str()) {
        file_name_slice
    } else {
        "???"
    };

    format!("{} {}.{:06} {}[{} {}:{}] {}",
            match record.level() {
                LogLevel::Error => 'E',
                LogLevel::Warn => 'W',
                LogLevel::Info => 'I',
                LogLevel::Debug => 'D',
                LogLevel::Trace => 'T',
            },
            if let Ok(time_txt) = ::time::strftime("%T", &now) {
                time_txt
            } else {
                "".to_owned()
            },
            now.tm_nsec / 1000,
            thread_name,
            record.location().module_path(),
            file_name,
            record.location().line(),
            record.args())
}

fn build<F: 'static>(format: F)
    where F: Fn(&LogRecord) -> String + Sync + Send
{
    let mut builder = LogBuilder::new();
    let _ = builder.format(format);

    if let Ok(rust_log) = env::var("RUST_LOG") {
        let _ = builder.parse(&rust_log);
    }

    builder.init().unwrap_or_else(|error| println!("Error initialising logger: {}", error));
}
