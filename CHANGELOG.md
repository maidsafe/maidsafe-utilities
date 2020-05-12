# MaidSafe Utilities - Change Log

## [0.19.0]
- Update to Rust 1.43 stable
- Migrate CI/CD to GitHub actions
- Remove logging to a websocket

## [0.18.0]
- Update `url` dependency from `~1.5.1` to `~1.7.2`

## [0.17.0]
- Change log tests to integration ones
- Add log_or_panic macro

## [0.16.0]
- Update to dual license (MIT/BSD)
- Upgrade unwrap version to 1.2.0
- Use rust 1.28.0 stable / 2018-07-07 nightly
- rustfmt 0.99.2 and clippy-0.0.212

## [0.15.0]
- Use rust 1.22.1 stable / 2017-11-23 nightly
- rustfmt 0.9.0 and clippy-0.0.174

## [0.14.1]
- Fix issue when deserialising corrupt data
- Reconnect web socket logger if it disconnects

## [0.14.0]
- Use rust 1.19 stable / 2017-07-20 nightly
- rustfmt 0.9.0 and clippy-0.0.144
- Replace -Zno-trans with cargo check
- Make appveyor script using fixed version of stable

## [0.13.0]
- Change `serialise` to return error if not all input bytes are consumed.
- Refactor the log module.
- Add an option to override a log output file name.

## [0.12.1]
- Fix to make log4rs use the toml format for our log.toml file.

## [0.12.0]
- Update to Rust 1.17 stable
- Update serde serialisation
- Update CI to run cargo_install from QA

## [0.11.4]
- Fix seeded_rng to avoid generating identical rngs.

## [0.11.3]
- Make `SeededRng` deterministic even if there are several test threads.

## [0.11.2]
- Update `config_file_handler` and `ws` to latest - v0.6.0 and v0.7.0 respectively.

## [0.11.1]
- Fix seeded-rng bug in which it was not getting printed for failing tests if there as a passing test that ran before
- The construction of SeededRng was faulty as it lead to printout of the inner most SeededRng of a stack frame instead of the global one from which all others were derived. This is fixed too.

## [0.11.0]
- Use serde instead of rustc-serialize
- rustfmt 0.8.1
- remove big-endian-sip-hasher
- deterministically seeded thread local version of seeded-rng
- make shuffle consistent across architectures

## [0.10.2]
- Fix the bug which created an empty log file when timestampping was set to true in log.toml
- Update and conform to rustc 1.16.0 stable, 2017-03-16 nightly and Clippy 0.0.120

## [0.10.1]
- Update CI scripts and remove the requirement of clippy in Cargo.toml.
- Add timestamping to file-names if specified so in log.toml.

## [0.10.0]
- Removed deprecated type and macros.
- Removed upper limit from serialisation helpers.
- Fixed log formatting regression.

## [0.9.0]
- Use config_file_handler crate to derive file locations.
- Integration with current log4rs (0.4.8) leading to changes in the log.toml specification.

## [0.8.0]
- Revert use of `unwrap!` inside `thread!`.

## [0.7.1]
- Replaced `thread!` macro with `named()` function.
- Renamed `RaiiThreadJoiner` to `Joiner`.
- Modified `SeededRng::from_seed()` to panic rather than return an error.

## [0.7.0]
- Fixed Clippy warnings.
- Added `SeededRng` type.

## [0.6.0]
- Added endian-agnostic Sip hash function.
- Added test for log.toml file.
- Replaced usage of `time` crate with `std::time`.

## [0.5.4]
- Read the config file from the binary directory instead of the current one.
- Websocket logging to the web-server writes the complete and verbose JSON when
  no pattern is specified in `log.toml` for async_web_socket.

## [0.5.3]
- Logging of date and time to web-server is now an ISO format with time-zone offset
- Unique id in log messages is string instead of u64 in JSON as 64 bit numbers are not supported out of the box in NodeJS web-servers.

## [0.5.2]
- Added ability to serialise and deserialise without being limited by the default size limit
- Websocket logging now logs unique ids as well

## [0.5.1]
- Fixed serialisation issue.
- Fixed bug in logging.
- Updated logging docs.
- Updated Contributor Agreement to version 1.1.

## [0.5.0]
- Async Logging introduced - uses log4rs.

## [0.4.1]
- Use bincode serialisation size limits.

## [0.4.0]
- Changed from using CBOR to Bincode.
- Disabled Clippy warning.

## [0.3.0]
- Added new function to allow logging to file.

## [0.2.1]
- Used quick-error for SerialisationError.

## [0.2.0]
- Clippy fixes including renaming some public enum variants.
- Formatting and style fixes.
- Limited length of filename in log output.

## [0.1.5]
- to_string() -> to_owned()
- str + str -> str.push(str)

## [0.1.4]
- Added serialisation module to encode and decode types using Cbor.

## [0.1.3]
- Added MaidSafeObserver to facilitate Routing to work with multiple event-subsets in a single thread.

## [0.1.2]
- Added env_logger initialiser.

## [0.1.1]
- Remove wildcard dependencies.

## [0.1.0]
- Thread spawning
- Thread joining via RAII
- Unwrap helpers for `Option` and `Result`
- `EventSender` for event sub-setting
