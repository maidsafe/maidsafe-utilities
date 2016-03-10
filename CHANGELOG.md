# MaidSafe Utilities - Change Log

## [0.4.1]
- Use bincode serialisation size limits

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
