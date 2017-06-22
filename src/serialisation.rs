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

use bincode::{Bounded, Error, ErrorKind, Infinite, deserialize_from, serialize, serialize_into,
              serialized_size, serialized_size_bounded};
use serde::de::DeserializeOwned;
use serde::ser::Serialize;
use std::io::{Cursor, Read, Write};

quick_error! {
    /// Serialisation error.
    #[derive(Debug)]
    pub enum SerialisationError {
        /// Error during serialisation (encoding).
        Serialise(err: ErrorKind) {
            description("Serialise error")
            display("Serialise error: {}", err)
            cause(err)
        }

        /// Error during deserialisation (decoding).
        Deserialise(err: ErrorKind) {
            description("Deserialise error")
            display("Deserialise error: {}", err)
            cause(err)
        }
    }
}

/// Serialise an `Serialize` type with no limit on the size of the serialised data.
pub fn serialise<T>(data: &T) -> Result<Vec<u8>, SerialisationError>
    where T: Serialize
{
    serialize(data, Infinite).map_err(|e| SerialisationError::Serialise(*e))
}

/// Serialise an `Serialize` type with max limit specified.
pub fn serialise_with_limit<T>(data: &T, size_limit: Bounded) -> Result<Vec<u8>, SerialisationError>
    where T: Serialize
{
    serialize(data, size_limit).map_err(|e| SerialisationError::Serialise(*e))
}

/// Deserialise a `Deserialize` type with no limit on the size of the serialised data.
pub fn deserialise<T>(data: &[u8]) -> Result<T, SerialisationError>
    where T: Serialize + DeserializeOwned
{
    let mut cursor = Cursor::new(data);
    deserialize_from(&mut cursor, Infinite)
        .and_then(|parsed| check_deserialised_size(data, parsed))
        .map_err(|e| SerialisationError::Deserialise(*e))
}

/// Deserialise a `Deserialize` type with max size limit specified.
pub fn deserialise_with_limit<T>(data: &[u8], size_limit: Bounded) -> Result<T, SerialisationError>
    where T: Serialize + DeserializeOwned
{
    let mut cursor = Cursor::new(data);
    deserialize_from(&mut cursor, size_limit)
        .and_then(|parsed| check_deserialised_size(data, parsed))
        .map_err(|e| SerialisationError::Deserialise(*e))
}

/// Serialise an `Serialize` type directly into a `Write` with no limit on the size of the
/// serialised data.
pub fn serialise_into<T: Serialize, W: Write>(data: &T,
                                              write: &mut W)
                                              -> Result<(), SerialisationError> {
    serialize_into(write, data, Infinite).map_err(|e| SerialisationError::Serialise(*e))
}

/// Serialise an `Serialize` type directly into a `Write` with max size limit specified.
pub fn serialise_into_with_limit<T: Serialize, W: Write>(data: &T,
                                                         write: &mut W,
                                                         size_limit: Bounded)
                                                         -> Result<(), SerialisationError> {
    serialize_into(write, data, size_limit).map_err(|e| SerialisationError::Serialise(*e))
}

/// Deserialise a `Deserialize` type directly from a `Read` with no limit on the size of the
/// serialised data.
pub fn deserialise_from<R: Read, T: DeserializeOwned>(read: &mut R)
                                                      -> Result<T, SerialisationError> {
    deserialize_from(read, Infinite).map_err(|e| SerialisationError::Deserialise(*e))
}

/// Deserialise a `Deserialize` type directly from a `Read` with max size limit specified.
pub fn deserialise_from_with_limit<R: Read, T: DeserializeOwned>
    (read: &mut R,
     size_limit: Bounded)
     -> Result<T, SerialisationError> {
    deserialize_from(read, size_limit).map_err(|e| SerialisationError::Deserialise(*e))
}

/// Returns the size that an object would be if serialised using [`serialise()`](fn.serialise.html).
pub fn serialised_size<T: Serialize>(data: &T) -> u64 {
    serialized_size(data)
}

/// Given a maximum size limit, check how large an object would be if it were to be serialised.
///
/// If it can be encoded in `max` or fewer bytes, that number will be returned inside `Some`.  If it
/// goes over bounds, then `None` is returned.
pub fn serialised_size_with_limit<T: Serialize>(data: &T, max: u64) -> Option<u64> {
    serialized_size_bounded(data, max)
}

fn check_deserialised_size<T>(serialised: &[u8], deserialised: T) -> Result<T, Error>
    where T: Serialize + DeserializeOwned
{
    if serialized_size(&deserialised) == serialised.len() as u64 {
        Ok(deserialised)
    } else {
        Err(Box::new(ErrorKind::Custom("Not all bytes of slice consumed.".to_string())))
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use bincode::{Bounded, ErrorKind};
    use std::io::Cursor;

    #[test]
    fn serialise_deserialise() {
        let original_data = (vec![0u8, 1, 3, 9], vec![-1i64, 888, -8765], "SomeString".to_string());

        let serialised_data = unwrap!(serialise(&original_data));
        let deserialised_data: (Vec<u8>, Vec<i64>, String) = unwrap!(deserialise(&serialised_data));
        assert_eq!(original_data, deserialised_data);

        // Try to parse a `String` into a `u64` to check the unused bytes triggers an error.
        let serialised_string = unwrap!(serialise(&"Another string".to_string()));
        if let Err(SerialisationError::Deserialise(ErrorKind::Custom(string))) =
            deserialise::<u64>(&serialised_string) {
            assert_eq!(&string, "Not all bytes of slice consumed.");
        } else {
            panic!("Failed to return the right error type.");
        }
    }

    #[test]
    fn serialise_into_deserialise_from() {
        let original_data = (vec![0u8, 1, 3, 9], vec![-1i64, 888, -8765], "SomeString".to_string());
        let mut serialised_data = vec![];
        unwrap!(serialise_into(&original_data, &mut serialised_data));

        let mut serialised = Cursor::new(serialised_data);
        let deserialised_data: (Vec<u8>, Vec<i64>, String) =
            unwrap!(deserialise_from(&mut serialised));
        assert_eq!(original_data, deserialised_data);
    }

    #[test]
    fn upper_limit() {
        let upper_limit = Bounded(64);
        // Test with data which is at limit
        let mut original_data = (1u64..8).collect::<Vec<_>>();
        let mut serialised_data = unwrap!(serialise_with_limit(&original_data, upper_limit));
        let mut deserialised_data: Vec<u64> = unwrap!(deserialise(&serialised_data));
        assert_eq!(original_data, deserialised_data);

        serialised_data.clear();
        unwrap!(serialise_into_with_limit(&original_data, &mut serialised_data, upper_limit));
        let mut serialised = Cursor::new(serialised_data);
        deserialised_data = unwrap!(deserialise_from(&mut serialised));
        assert_eq!(original_data, deserialised_data);

        // Try to serialise data above limit
        original_data.push(0);
        if let Err(SerialisationError::Serialise(ErrorKind::SizeLimit)) =
            serialise_with_limit(&original_data, upper_limit) {
        } else {
            panic!("Expected size limit error.");
        }
        let mut buffer = vec![];
        if let Err(SerialisationError::Serialise(ErrorKind::SizeLimit)) =
            serialise_into_with_limit(&original_data, &mut buffer, upper_limit) {
        } else {
            panic!("Expected size limit error.");
        }

        // Try to deserialise data above limit
        let excessive = unwrap!(serialise(&original_data));
        if let Err(SerialisationError::Deserialise(ErrorKind::SizeLimit)) =
            deserialise_with_limit::<Vec<u64>>(&excessive, upper_limit) {
        } else {
            panic!("Expected size limit error.");
        }
        serialised = Cursor::new(excessive);
        if let Err(SerialisationError::Deserialise(ErrorKind::SizeLimit)) =
            deserialise_from_with_limit::<Cursor<_>, Vec<u64>>(&mut serialised, upper_limit) {
        } else {
            panic!("Expected size limit error.");
        }
    }

    #[test]
    fn sizes() {
        let data = (1u64..8).collect::<Vec<_>>();
        assert_eq!(serialised_size(&data), 64);
        assert_eq!(serialised_size_with_limit(&data, 100), Some(64));
        assert_eq!(serialised_size_with_limit(&data, 64), Some(64));
        assert!(serialised_size_with_limit(&data, 63).is_none());
    }
}
