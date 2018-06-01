// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

use bincode::{
    deserialize, deserialize_from, serialize, serialize_into, serialized_size,
    serialized_size_bounded, Bounded, ErrorKind, Infinite,
};
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

        /// Bincode error during deserialisation (decoding).
        Deserialise(err: ErrorKind) {
            description("Deserialise error")
            display("Deserialise error: {}", err)
            cause(err)
        }

        /// Not all input bytes were consumed when deserialising (decoding).
        DeserialiseExtraBytes {
            description("DeserialiseExtraBytes error")
            display("Deserialise error: Not all bytes of slice consumed")
        }
    }
}

/// Serialise an `Serialize` type with no limit on the size of the serialised data.
pub fn serialise<T>(data: &T) -> Result<Vec<u8>, SerialisationError>
where
    T: Serialize,
{
    serialize(data, Infinite).map_err(|e| SerialisationError::Serialise(*e))
}

/// Serialise an `Serialize` type with max limit specified.
pub fn serialise_with_limit<T>(data: &T, size_limit: Bounded) -> Result<Vec<u8>, SerialisationError>
where
    T: Serialize,
{
    serialize(data, size_limit).map_err(|e| SerialisationError::Serialise(*e))
}

/// Deserialise a `Deserialize` type with no limit on the size of the serialised data.
pub fn deserialise<T>(data: &[u8]) -> Result<T, SerialisationError>
where
    T: Serialize + DeserializeOwned,
{
    let value = deserialize(data).map_err(|e| SerialisationError::Deserialise(*e))?;
    if serialized_size(&value) != data.len() as u64 {
        return Err(SerialisationError::DeserialiseExtraBytes);
    }
    Ok(value)
}

/// Deserialise a `Deserialize` type with max size limit specified.
pub fn deserialise_with_limit<T>(data: &[u8], size_limit: Bounded) -> Result<T, SerialisationError>
where
    T: DeserializeOwned,
{
    let mut cursor = Cursor::new(data);
    let value =
        deserialize_from(&mut cursor, size_limit).map_err(|e| SerialisationError::Deserialise(*e))?;
    if cursor.position() != data.len() as u64 {
        return Err(SerialisationError::DeserialiseExtraBytes);
    }
    Ok(value)
}

/// Serialise an `Serialize` type directly into a `Write` with no limit on the size of the
/// serialised data.
pub fn serialise_into<T: Serialize, W: Write>(
    data: &T,
    write: &mut W,
) -> Result<(), SerialisationError> {
    serialize_into(write, data, Infinite).map_err(|e| SerialisationError::Serialise(*e))
}

/// Serialise an `Serialize` type directly into a `Write` with max size limit specified.
pub fn serialise_into_with_limit<T: Serialize, W: Write>(
    data: &T,
    write: &mut W,
    size_limit: Bounded,
) -> Result<(), SerialisationError> {
    serialize_into(write, data, size_limit).map_err(|e| SerialisationError::Serialise(*e))
}

/// Deserialise a `Deserialize` type directly from a `Read` with no limit on the size of the
/// serialised data.
pub fn deserialise_from<R: Read, T: DeserializeOwned>(
    read: &mut R,
) -> Result<T, SerialisationError> {
    deserialize_from(read, Infinite).map_err(|e| SerialisationError::Deserialise(*e))
}

/// Deserialise a `Deserialize` type directly from a `Read` with max size limit specified.
pub fn deserialise_from_with_limit<R: Read, T: DeserializeOwned>(
    read: &mut R,
    size_limit: Bounded,
) -> Result<T, SerialisationError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use bincode::{Bounded, ErrorKind};
    use serde::de::{self, Visitor};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::fmt;
    use std::io::Cursor;

    #[test]
    fn serialise_deserialise() {
        let original_data = (
            vec![0u8, 1, 3, 9],
            vec![-1i64, 888, -8765],
            "SomeString".to_string(),
        );

        let serialised_data = unwrap!(serialise(&original_data));
        let deserialised_data: (Vec<u8>, Vec<i64>, String) = unwrap!(deserialise(&serialised_data));
        assert_eq!(original_data, deserialised_data);

        // Try to parse a `String` into a `u64` to check the unused bytes triggers an error.
        let serialised_string = unwrap!(serialise(&"Another string".to_string()));
        match deserialise::<u64>(&serialised_string).unwrap_err() {
            SerialisationError::DeserialiseExtraBytes => (),
            err => panic!("{:?}", err),
        }
    }

    #[test]
    fn serialise_into_deserialise_from() {
        let original_data = (
            vec![0u8, 1, 3, 9],
            vec![-1i64, 888, -8765],
            "SomeString".to_string(),
        );
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
        unwrap!(serialise_into_with_limit(
            &original_data,
            &mut serialised_data,
            upper_limit,
        ));
        let mut serialised = Cursor::new(serialised_data);
        deserialised_data = unwrap!(deserialise_from(&mut serialised));
        assert_eq!(original_data, deserialised_data);

        // Try to serialise data above limit
        original_data.push(0);
        if let Err(SerialisationError::Serialise(ErrorKind::SizeLimit)) =
            serialise_with_limit(&original_data, upper_limit)
        {
        } else {
            panic!("Expected size limit error.");
        }
        let mut buffer = vec![];
        if let Err(SerialisationError::Serialise(ErrorKind::SizeLimit)) =
            serialise_into_with_limit(&original_data, &mut buffer, upper_limit)
        {
        } else {
            panic!("Expected size limit error.");
        }

        // Try to deserialise data above limit
        let excessive = unwrap!(serialise(&original_data));
        if let Err(SerialisationError::Deserialise(ErrorKind::SizeLimit)) =
            deserialise_with_limit::<Vec<u64>>(&excessive, upper_limit)
        {
        } else {
            panic!("Expected size limit error.");
        }
        serialised = Cursor::new(excessive);
        if let Err(SerialisationError::Deserialise(ErrorKind::SizeLimit)) =
            deserialise_from_with_limit::<Cursor<_>, Vec<u64>>(&mut serialised, upper_limit)
        {
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

    #[derive(PartialEq, Eq, Debug)]
    struct Wrapper([u8; 1]);

    impl Serialize for Wrapper {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            serializer.serialize_bytes(&self.0[..])
        }
    }

    impl<'de> Deserialize<'de> for Wrapper {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Wrapper, D::Error> {
            struct WrapperVisitor;
            impl<'de> Visitor<'de> for WrapperVisitor {
                type Value = Wrapper;
                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "Wrapper")
                }
                fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Self::Value, E> {
                    if value.len() != 1 {
                        return Err(de::Error::invalid_length(value.len(), &self));
                    }
                    Ok(Wrapper([value[0]]))
                }
            }
            deserializer.deserialize_bytes(WrapperVisitor)
        }
    }

    #[test]
    // The bincode implementation of `serialize_bytes` puts the number of bytes of raw data as the
    // first 8 bytes of the encoded data.  The corresponding `deserialize_bytes` uses these first 8
    // bytes to deduce the size of the buffer into which the raw bytes should then be copied.  If we
    // use bincode's `deserialize_from(.., Infinite)` to try and parse such data, size-checking is
    // disabled when allocating the buffer, and corrupted serialised data could cause an OOM crash.
    fn deserialize_bytes() {
        let wrapper = Wrapper([255]);
        let serialised_wrapper = unwrap!(serialise(&wrapper));
        // If the following assertion fails, revisit how we're encoding data via `serialize_bytes`
        // to check that the following `tampered` array below is still trying to trigger an OOM
        // error.
        assert_eq!(serialised_wrapper, [1, 0, 0, 0, 0, 0, 0, 0, 255]);
        let deserialised_wrapper: Wrapper = unwrap!(deserialise(&serialised_wrapper));
        assert_eq!(wrapper, deserialised_wrapper);

        // Try to trigger an OOM crash.
        let tampered = [255u8; 9];
        match deserialise::<Wrapper>(&tampered).unwrap_err() {
            SerialisationError::Deserialise(_) => (),
            err => panic!("{:?}", err),
        }

        match deserialise::<Wrapper>(&[1, 0, 0, 0, 0, 0, 0, 0, 255, 255]).unwrap_err() {
            SerialisationError::DeserialiseExtraBytes => (),
            err => panic!("{:?}", err),
        }
    }
}
