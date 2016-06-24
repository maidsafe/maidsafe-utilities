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

use std;
use std::thread::JoinHandle;

/// A RAII style thread joiner. The destruction of an instance of this type will block until
/// the thread it is managing has joined.
pub struct Joiner {
    joiner: Option<JoinHandle<()>>,
}

/// Deprecated name for `Joiner`
pub type RaiiThreadJoiner = Joiner;

impl Joiner {
    /// Create a new instance of self-managing thread joiner
    pub fn new(joiner: JoinHandle<()>) -> Joiner {
        Joiner { joiner: Some(joiner) }
    }

    /// Releases the `Joiner` by detaching the thread.
    pub fn detach(mut self) {
        let _ = unwrap_option!(self.joiner.take(),
                               "Programming error: please report this as a bug.");
    }
}

impl Drop for Joiner {
    fn drop(&mut self) {
        if let Some(joiner) = self.joiner.take() {
            unwrap_result!(joiner.join());
        }
    }
}

/// This macro is intended to be used in all cases where we want to spawn a new thread of execution
/// and if that is not possible then panic out.
///
/// #Examples
///
/// ```
/// # #[macro_use]
/// # extern crate maidsafe_utilities;
/// # #[macro_use]
/// # extern crate unwrap;
/// # fn main() {
/// let _ = thread!("DaemonThread", move || {
///     std::thread::sleep(std::time::Duration::from_millis(10));
/// });
///
/// let sleep_duration_ms = 500;
/// let _raii_joiner = maidsafe_utilities::thread
///                                      ::Joiner::new(thread!("ManagedThread", move || {
///     std::thread::sleep(std::time::Duration::from_millis(sleep_duration_ms));
/// }));
/// # }
/// ```
#[macro_export]
macro_rules! thread {
    ($thread_name:expr, $entry_point:expr) => {
        unwrap_result!(::std::thread::Builder::new().name($thread_name.to_owned())
                                             .spawn($entry_point))
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
    where S: Into<String>,
          F: FnOnce() + Send + 'static
{
    let thread_name: String = thread_name.into();
    let join_handle_res = std::thread::Builder::new()
        .name(thread_name)
        .spawn(func);
    Joiner::new(unwrap!(join_handle_res))
}

#[cfg(test)]
mod test {
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
