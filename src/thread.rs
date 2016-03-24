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

use std::thread::JoinHandle;

/// A RAII style thread joiner. The destruction of an instance of this type will block until
/// the thread it is managing has joined.
pub struct RaiiThreadJoiner {
    joiner: Option<JoinHandle<()>>,
}

impl RaiiThreadJoiner {
    /// Create a new instance of self-managing thread joiner
    pub fn new(joiner: JoinHandle<()>) -> RaiiThreadJoiner {
        RaiiThreadJoiner { joiner: Some(joiner) }
    }
}

impl Drop for RaiiThreadJoiner {
    fn drop(&mut self) {
        let joiner = unwrap_option!(self.joiner.take(),
                                    "Programming error: please report this as a bug.");
        unwrap_result!(joiner.join());
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
/// # fn main() {
/// let _ = thread!("DaemonThread", move || {
///     std::thread::sleep(std::time::Duration::from_millis(10));
/// });
///
/// let sleep_duration_ms = 500;
/// let _raii_joiner = maidsafe_utilities::thread
///                                      ::RaiiThreadJoiner::new(thread!("ManagedThread", move || {
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

#[cfg(test)]
mod test {
    extern crate time;

    use super::*;
    use std::thread;
    use std::time::Duration;
    use self::time::SteadyTime;

    #[test]
    fn raii_thread_joiner() {
        const SLEEP_DURATION_DAEMON: u64 = 150;
        // Factor of 3 should ensure that all threads gracefully exit when the test finishes
        const SLEEP_DURATION_MANAGED: u64 = SLEEP_DURATION_DAEMON * 3;

        {
            let time_before = SteadyTime::now();
            {
                let _ = thread!("JoinerTestDaemon", move || {
                    thread::sleep(Duration::from_millis(SLEEP_DURATION_DAEMON));
                });
            }
            let time_after = SteadyTime::now();

            let diff = time_after - time_before;

            assert!(diff < time::Duration::milliseconds(SLEEP_DURATION_DAEMON as i64));
        }

        {
            let time_before = SteadyTime::now();
            {
                let _raii_joiner = RaiiThreadJoiner::new(thread!("JoinerTestManaged", move || {
                    thread::sleep(Duration::from_millis(SLEEP_DURATION_MANAGED));
                }));
            }
            let time_after = SteadyTime::now();

            let diff = time_after - time_before;

            assert!(diff >= time::Duration::milliseconds(SLEEP_DURATION_MANAGED as i64));
        }
    }
}
