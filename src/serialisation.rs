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

use bincode::SizeLimit;
use bincode::rustc_serialize::{DecodingError, EncodingError, decode_from, encode, encode_into,
                               encoded_size, encoded_size_bounded};
use rustc_serialize::{Decodable, Encodable};
use std::io::{Cursor, Read, Write};

quick_error! {
    /// Serialisation error.
    #[derive(Debug)]
    pub enum SerialisationError {
        /// Error during serialisation (encoding).
        Serialise(err: EncodingError) {
            description("Serialise error")
            display("Serialise error: {}", err)
            cause(err)
            from()
        }

        /// Error during deserialisation (decoding).
        Deserialise(err: DecodingError) {
            description("Deserialise error")
            display("Deserialise error: {}", err)
            cause(err)
            from()
        }
    }
}

/// Serialise an `Encodable` type with no limit on the size of the serialised data.
pub fn serialise<T>(data: &T) -> Result<Vec<u8>, SerialisationError>
    where T: Encodable
{
    encode(data, SizeLimit::Infinite).map_err(From::from)
}

/// Serialise an `Encodable` type with max limit specified.
pub fn serialise_with_limit<T>(data: &T,
                               size_limit: SizeLimit)
                               -> Result<Vec<u8>, SerialisationError>
    where T: Encodable
{
    encode(data, size_limit).map_err(From::from)
}

/// Deserialise a `Decodable` type with no limit on the size of the serialised data.
pub fn deserialise<T>(data: &[u8]) -> Result<T, SerialisationError>
    where T: Decodable
{
    let mut data = Cursor::new(data);
    deserialise_from(&mut data)
}

/// Deserialise a `Decodable` type with max size limit specified.
pub fn deserialise_with_limit<T>(data: &[u8],
                                 size_limit: SizeLimit)
                                 -> Result<T, SerialisationError>
    where T: Decodable
{
    let mut data = Cursor::new(data);
    deserialise_from_with_limit(&mut data, size_limit)
}

/// Serialise an `Encodable` type directly into a `Write` with no limit on the size of the
/// serialised data.
pub fn serialise_into<T: Encodable, W: Write>(data: &T,
                                              write: &mut W)
                                              -> Result<(), SerialisationError> {
    encode_into(data, write, SizeLimit::Infinite).map_err(From::from)
}

/// Serialise an `Encodable` type directly into a `Write` with max size limit specified.
pub fn serialise_into_with_limit<T: Encodable, W: Write>(data: &T,
                                                         write: &mut W,
                                                         size_limit: SizeLimit)
                                                         -> Result<(), SerialisationError> {
    encode_into(data, write, size_limit).map_err(From::from)
}

/// Deserialise a `Decodable` type directly from a `Read` with no limit on the size of the
/// serialised data.
pub fn deserialise_from<R: Read, T: Decodable>(read: &mut R) -> Result<T, SerialisationError> {
    decode_from(read, SizeLimit::Infinite).map_err(From::from)
}

/// Deserialise a `Decodable` type directly from a `Read` with max size limit specified.
pub fn deserialise_from_with_limit<R: Read, T: Decodable>(read: &mut R,
                                                          size_limit: SizeLimit)
                                                          -> Result<T, SerialisationError> {
    decode_from(read, size_limit).map_err(From::from)
}

/// Returns the size that an object would be if serialised using [`serialise()`](fn.serialise.html).
pub fn serialised_size<T: Encodable>(data: &T) -> u64 {
    encoded_size(data)
}

/// Given a maximum size limit, check how large an object would be if it were to be serialised.
///
/// If it can be encoded in `max` or fewer bytes, that number will be returned inside `Some`.  If it
/// goes over bounds, then `None` is returned.
pub fn serialised_size_with_limit<T: Encodable>(data: &T, max: u64) -> Option<u64> {
    encoded_size_bounded(data, max)
}



#[cfg(test)]
mod tests {
    use super::*;
    use bincode::SizeLimit;
    use bincode::rustc_serialize::{DecodingError, EncodingError};
    use std::io::Cursor;

    #[test]
    fn serialise_deserialise() {
        let original_data = (vec![0u8, 1, 3, 9], vec![-1i64, 888, -8765], "Some-String".to_owned());

        let serialised_data = unwrap!(serialise(&original_data));
        let deserialised_data: (Vec<u8>, Vec<i64>, String) = unwrap!(deserialise(&serialised_data));
        assert_eq!(original_data, deserialised_data);
    }

    #[test]
    fn serialise_into_deserialise_from() {
        let original_data = (vec![0u8, 1, 3, 9], vec![-1i64, 888, -8765], "Some-String".to_owned());
        let mut serialised_data = vec![];
        unwrap!(serialise_into(&original_data, &mut serialised_data));

        let mut serialised = Cursor::new(serialised_data);
        let deserialised_data: (Vec<u8>, Vec<i64>, String) =
            unwrap!(deserialise_from(&mut serialised));
        assert_eq!(original_data, deserialised_data);
    }

    #[test]
    fn upper_limit() {
        let upper_limit = SizeLimit::Bounded(64);
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
        if let Err(SerialisationError::Serialise(EncodingError::SizeLimit)) =
            serialise_with_limit(&original_data, upper_limit) {
        } else {
            panic!("Expected size limit error.");
        }
        let mut buffer = vec![];
        if let Err(SerialisationError::Serialise(EncodingError::SizeLimit)) =
            serialise_into_with_limit(&original_data, &mut buffer, upper_limit) {
        } else {
            panic!("Expected size limit error.");
        }

        // Try to deserialise data above limit
        let excessive = unwrap!(serialise(&original_data));
        if let Err(SerialisationError::Deserialise(DecodingError::SizeLimit)) =
            deserialise_with_limit::<Vec<u64>>(&excessive, upper_limit) {
        } else {
            panic!("Expected size limit error.");
        }
        serialised = Cursor::new(excessive);
        if let Err(SerialisationError::Deserialise(DecodingError::SizeLimit)) =
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
