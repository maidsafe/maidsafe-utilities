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

/// A replacement for calling `unwrap()` on a `Result`.
///
/// This macro is intended to be used in all cases where we `unwrap` a `Result` to deliberately
/// panic in case of error, e.g. in test-cases.  Such `unwrap`s don't give a precise point of
/// failure in our code and instead indicate some line number in the Rust core library.  This macro
/// provides a precise point of failure and decorates the failure for easy viewing.
///
/// # Examples
///
/// ```
/// # #[macro_use]
/// # extern crate maidsafe_utilities;
/// # fn main() {
/// let some_result: Result<String, std::io::Error> = Ok("Hello".to_string());
/// let string_length = unwrap_result!(some_result).len();
/// assert_eq!(string_length, 5);
/// # }
/// ```
#[macro_export]
macro_rules! unwrap_result {
    ($result:expr) => {
        $result.unwrap_or_else(|error| {
            let mut message =
                "Result evaluated to Err: ".to_owned();
            message.push_str(&format!("{:?}", error)[..]);
            let decorator_length = ::std::cmp::min(message.len() + 2, 100);
            let decorator = (0..decorator_length).map(|_| "=").collect::<String>();
            panic!("\n\n {}\n| {} |\n {}\n\n", decorator, message, decorator)
        })
    }
}

/// A replacement for calling `unwrap()` on an `Option`.
///
/// This macro is intended to be used in all cases where we `unwrap` an `Option` to deliberately
/// panic in case of error, e.g. in test-cases.  Such `unwrap`s don't give a precise point of
/// failure in our code and instead indicate some line number in the Rust core library.  This macro
/// provides a precise point of failure and decorates the failure for easy viewing.
///
/// # Examples
///
/// ```
/// # #[macro_use]
/// # extern crate maidsafe_utilities;
/// # fn main() {
/// let some_option = Some("Hello".to_string());
/// let string_length = unwrap_option!(some_option, "This is a user-supplied text.").len();
/// assert_eq!(string_length, 5);
/// # }
/// ```
#[macro_export]
macro_rules! unwrap_option {
    ($option:expr, $user_string:expr) => {
        $option.unwrap_or_else(|| {
            let mut error = "Option evaluated to None".to_owned();
            if !$user_string.is_empty() {
                error.push_str(": \"");
                error.push_str($user_string);
                error.push_str("\"");
            }
            let decorator_length = ::std::cmp::min(error.len() + 2, 100);
            let decorator = (0..decorator_length).map(|_| "=").collect::<String>();
            panic!("\n\n {}\n| {} |\n {}\n\n", decorator, error, decorator)
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn unwrap_good_result() {
        let result: Result<u8, ()> = Ok(1);
        assert_eq!(unwrap_result!(result), 1);
    }

    #[test]
    fn unwrap_good_option() {
        let option = Some(1);
        assert_eq!(unwrap_option!(option, "Error text."), 1);
    }

    #[test]
    #[should_panic(expected = "Message on failure.")]
    fn unwrap_bad_result() {
        let result: Result<(), String> = Err("Message on failure.".to_string());
        unwrap_result!(result);
    }

    #[test]
    #[should_panic(expected = "Message on failure.")]
    fn unwrap_bad_option() {
        let option: Option<()> = None;
        unwrap_option!(option, "Message on failure.");
    }
}
