// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

use std;
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
        Joiner { joiner: Some(joiner) }
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
                }).detach();
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
