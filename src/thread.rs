// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use std::fmt;
use std::thread::JoinHandle;

/// A RAII style thread joiner. The destruction of an instance of this type will block until
/// the thread it is managing has joined.
pub struct Joiner {
    joiner: Option<JoinHandle<()>>,
}

impl fmt::Debug for Joiner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.joiner.is_some() {
            write!(f, "Joiner {{ joiner: Some(...) }}")
        } else {
            write!(f, "Joiner {{ joiner: None }}")
        }
    }
}

impl Joiner {
    /// Create a new instance of self-managing thread joiner
    pub fn new(joiner: JoinHandle<()>) -> Joiner {
        Joiner {
            joiner: Some(joiner),
        }
    }

    /// Releases the `Joiner` by detaching the thread.
    pub fn detach(mut self) {
        let _ = unwrap!(self.joiner.take());
    }
}

impl Drop for Joiner {
    fn drop(&mut self) {
        if let Some(joiner) = self.joiner.take() {
            unwrap!(joiner.join());
        }
    }
}

/// This function is intended to be used in all cases where we want to spawn a new thread with a
/// given name and panic if we fail to create the thread.
///
/// #Examples
///
/// ```
/// let _ = maidsafe_utilities::thread::named("DaemonThread", move || {
///     std::thread::sleep(std::time::Duration::from_millis(10));
/// });
///
/// let sleep_duration_ms = 500;
/// let _raii_joiner = maidsafe_utilities::thread::named("ManagedThread", move || {
///     std::thread::sleep(std::time::Duration::from_millis(sleep_duration_ms));
/// });
/// ```
pub fn named<S, F>(thread_name: S, func: F) -> Joiner
where
    S: Into<String>,
    F: FnOnce() + Send + 'static,
{
    let thread_name: String = thread_name.into();
    let join_handle_res = std::thread::Builder::new().name(thread_name).spawn(func);
    Joiner::new(unwrap!(join_handle_res))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn raii_thread_joiner() {
        const SLEEP_DURATION_DAEMON: u64 = 150;
        // Factor of 3 should ensure that all threads gracefully exit when the test finishes
        const SLEEP_DURATION_MANAGED: u64 = SLEEP_DURATION_DAEMON * 3;

        {
            let time_before = Instant::now();
            {
                named("JoinerTestDaemon", move || {
                    thread::sleep(Duration::from_millis(SLEEP_DURATION_DAEMON));
                })
                .detach();
            }
            let diff = time_before.elapsed();

            assert!(diff < Duration::from_millis(SLEEP_DURATION_DAEMON));
        }

        {
            let time_before = Instant::now();
            {
                let _joiner = named("JoinerTestManaged", move || {
                    thread::sleep(Duration::from_millis(SLEEP_DURATION_MANAGED));
                });
            }
            let diff = time_before.elapsed();

            assert!(diff >= Duration::from_millis(SLEEP_DURATION_MANAGED));
        }
    }
}
