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

static INITIALISE_LOGGER: ::std::sync::Once = ::std::sync::ONCE_INIT;

/// This function initialises the env_logger.
///
/// An example of a log message is:
///
/// ```
/// # fn main() { /*
/// W 19:33:49.245434 <main> [example:src/main.rs:50] Warning level message.
/// ^        ^          ^        ^          ^                    ^
/// |    timestamp      | top-level module  |                 message
/// |                   |                   |
/// |              thread name       file and line no.
/// |
/// level (E, W, I, D, or T for error, warn, info, debug or trace respectively)
/// # */}
/// ```
///
/// Logging of the thread name is enabled or disabled via the `show_thread_name` parameter.  If
/// enabled, and the thread executing the log statement is unnamed, the thread name is shown as
/// `???`.
///
/// The function can safely be called multiple times concurrently.
///
/// #Examples
///
/// ```
/// #[macro_use]
/// extern crate log;
/// #[macro_use]
/// extern crate maidsafe_utilities;
///
/// fn main() {
///     maidsafe_utilities::log::init(true);
///
///     warn!("A warning");
///
///     let unnamed = ::std::thread::spawn(move || info!("Message in unnamed thread"));
///     let _ = unnamed.join();
///
///     let _named = ::maidsafe_utilities::thread::RaiiThreadJoiner::new(thread!("Worker",
///                      move || error!("Message in named thread")));
///
///     // W 12:24:07.064746 <main> [example:src/main.rs:9] A warning
///     // I 12:24:07.065746 ??? [example:src/main.rs:11] Message in unnamed thread
///     // E 12:24:07.065746 Worker [example:src/main.rs:14] Message in named thread
/// }
/// ```
pub fn init(show_thread_name: bool) {
    INITIALISE_LOGGER.call_once(|| {
        let format = move |record: &::logger::LogRecord| {
            let now = ::time::now();
            let mut thread_name = "".to_owned();
            if show_thread_name {
                thread_name = ::std::thread::current().name().unwrap_or("???").to_owned();
                thread_name.push_str(" ");
            }
            let filename_length = record.location().file().len();
            let file = if filename_length > 40 {
                let mut file = "...".to_owned();
                file.push_str(&record.location().file()[(filename_length - 40)..filename_length]);
                file
            } else {
                record.location().file().to_owned()
            };

            format!("{} {}.{:06} {}[{}:{}:{}] {}",
                    match record.level() {
                        ::logger::LogLevel::Error => 'E',
                        ::logger::LogLevel::Warn => 'W',
                        ::logger::LogLevel::Info => 'I',
                        ::logger::LogLevel::Debug => 'D',
                        ::logger::LogLevel::Trace => 'T',
                    },
                    if let Ok(time_txt) = ::time::strftime("%T", &now) {
                        time_txt
                    } else {
                        "".to_owned()
                    },
                    now.tm_nsec / 1000,
                    thread_name,
                    record.location().module_path().splitn(2, "::").next().unwrap_or(""),
                    file,
                    record.location().line(),
                    record.args())
        };

        let mut builder = ::env_logger::LogBuilder::new();
        let _ = builder.format(format);

        if let Ok(rust_log) = ::std::env::var("RUST_LOG") {
            let _ = builder.parse(&rust_log);
        }

        builder.init().unwrap_or_else(|error| println!("Error initialising logger: {}", error));
    });
}
