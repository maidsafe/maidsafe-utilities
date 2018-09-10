// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// https://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

/// This macro will panic with the given message if the "testing" feature is enabled and the calling
/// thread is not already panicking, otherwise it will simply log an error message.
///
/// # Example
/// ```no_run
/// #[macro_use]
/// extern crate log;
/// #[macro_use]
/// extern crate maidsafe_utilities;
///
/// fn main() {
///     log_or_panic!("Bad value: {}", 1746);
/// }
/// ```
#[macro_export]
macro_rules! log_or_panic {
    ($($arg:tt)*) => {
        if cfg!(any(test, feature = "testing")) && !::std::thread::panicking() {
            panic!($($arg)*);
        } else {
            error!($($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    struct Helper;

    impl Drop for Helper {
        fn drop(&mut self) {
            log_or_panic!("Dropping helper");
        }
    }

    #[test]
    #[should_panic(expected = "Bad value: 1746")]
    fn log_or_panic() {
        // Use the helper to check that we can handle calling `log_or_panic!` while panicking.
        let _helper = Helper;
        log_or_panic!("Bad value: {}", 1746);
    }
}
