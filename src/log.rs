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

//! These functions can initialise logging for output to stdout only, or to a file and
//! stdout. For more fine-grained control, create file called `log.toml` in the root
//! directory of the project, or in the same directory where the executable is.
//! See http://sfackler.github.io/log4rs/doc/v0.3.3/log4rs/index.html for details
//! about format and structure of this file.
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

use log4rs;
use log4rs::appender::{ConsoleAppender, FileAppender};
use log4rs::config::{Appender, Config, Root};
use log4rs::pattern::PatternLayout;
use log4rs::toml::Creator;

use std::path::Path;
use std::sync::{Once, ONCE_INIT};

use async_log::{AsyncConsoleAppenderCreator, AsyncFileAppenderCreator};
use logger::LogLevelFilter;

static INITIALISE_LOGGER: Once = ONCE_INIT;
static CONFIG_FILE: &'static str = "log.toml";
static DEFAULT_LOG_LEVEL_FILTER: LogLevelFilter = LogLevelFilter::Warn;

/// Initialises the env_logger for output to stdout.
///
/// For further details, see the [module docs](index.html).
pub fn init(show_thread_name: bool) {
    INITIALISE_LOGGER.call_once(|| {
        let config_path = Path::new(CONFIG_FILE);

        if config_path.is_file() {
            let mut creator = Creator::default();
            creator.add_appender("async_console", Box::new(AsyncConsoleAppenderCreator));
            creator.add_appender("async_file", Box::new(AsyncFileAppenderCreator));

            log4rs::init_file(config_path, creator)
        } else {
            let pattern = make_pattern(show_thread_name);

            let appender = ConsoleAppender::builder().pattern(pattern).build();
            let appender = Appender::builder("console".to_owned(), Box::new(appender)).build();

            let root = Root::builder(DEFAULT_LOG_LEVEL_FILTER).appender("console".to_owned()).build();
            let config = Config::builder(root).appender(appender).build().unwrap();

            // TODO: add loggers by parsing RUST_LOG env variable.

            log4rs::init_config(config)
        }
        .expect("failed to initialize logging");
    });
}

/// Initialises the env_logger for output to a file and to stdout.
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
///         "Logger already initialised".to_owned());
///
///     // E 22:38:05.499016 <main> [example:main.rs:7] An error!
/// }
/// ```
pub fn init_to_file<P: AsRef<Path>>(show_thread_name: bool, file_path: P) -> Result<(), String> {
    let mut result = Err("Logger already initialised".to_owned());

    INITIALISE_LOGGER.call_once(|| {
        let file_appender = FileAppender::builder(file_path)
                                .pattern(make_pattern(show_thread_name))
                                .append(false)
                                .build();
        let file_appender = match file_appender {
            Ok(appender) => appender,
            Err(error) => {
                result = Err(format!("{}", error));
                return;
            }
        };
        let file_appender = Appender::builder("file".to_owned(), Box::new(file_appender)).build();

        let console_appender = ConsoleAppender::builder()
                                   .pattern(make_pattern(show_thread_name))
                                   .build();
        let console_appender = Appender::builder("console".to_owned(), Box::new(console_appender))
                                   .build();

        let root = Root::builder(DEFAULT_LOG_LEVEL_FILTER)
                       .appender("console".to_owned())
                       .appender("file".to_owned())
                       .build();

        let config = Config::builder(root)
                         .appender(console_appender)
                         .appender(file_appender)
                         .build()
                         .unwrap();

        // TODO: add loggers by parsing RUST_LOG env variable.

        result = log4rs::init_config(config).map_err(|e| format!("{}", e))
    });

    result
}


fn make_pattern(show_thread_name: bool) -> PatternLayout {
    let pattern = if show_thread_name {
        "%l %T [%M:%f:%L] %m"
    } else {
        "%l [%M:%f:%L] %m"
    };

    PatternLayout::new(pattern).unwrap()
}
