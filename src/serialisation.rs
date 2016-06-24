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

use bincode::SizeLimit;
use bincode::rustc_serialize::{decode_from, DecodingError, encode, encode_into, EncodingError};
use rustc_serialize::{Encodable, Decodable};
use std::io::{Read, Write, Cursor};

const UPPER_LIMIT: SizeLimit = SizeLimit::Bounded(1 << 21);

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

/// Serialise an Encodable type using default max size limit.
pub fn serialise<T>(data: &T) -> Result<Vec<u8>, SerialisationError>
    where T: Encodable
{
    encode(data, UPPER_LIMIT).map_err(From::from)
}

/// Serialise an Encodable type with max limit specified.
pub fn serialise_with_limit<T>(data: &T,
                               size_limit: SizeLimit)
                               -> Result<Vec<u8>, SerialisationError>
    where T: Encodable
{
    encode(data, size_limit).map_err(From::from)
}

/// Deserialise a Decodable type using default max size limit.
pub fn deserialise<T>(data: &[u8]) -> Result<T, SerialisationError>
    where T: Decodable
{
    let mut data = Cursor::new(data);
    deserialise_from(&mut data)
}

/// Deserialise a Decodable type with max size limit specified.
pub fn deserialise_with_limit<T>(data: &[u8],
                                 size_limit: SizeLimit)
                                 -> Result<T, SerialisationError>
    where T: Decodable
{
    let mut data = Cursor::new(data);
    deserialise_from_with_limit(&mut data, size_limit)
}

/// Serialise an Encodable type directly into a Write with default size limit.
pub fn serialise_into<T: Encodable, W: Write>(data: &T,
                                              write: &mut W)
                                              -> Result<(), SerialisationError> {
    encode_into(data, write, UPPER_LIMIT).map_err(From::from)
}

/// Serialise an Encodable type directly into a Write with max size limit specified.
pub fn serialise_into_with_limit<T: Encodable, W: Write>(data: &T,
                                                         write: &mut W,
                                                         size_limit: SizeLimit)
                                                         -> Result<(), SerialisationError> {
    encode_into(data, write, size_limit).map_err(From::from)
}

/// Deserialise a Decodable type directly from a Read with default size limit.
pub fn deserialise_from<R: Read, T: Decodable>(read: &mut R) -> Result<T, SerialisationError> {
    decode_from(read, UPPER_LIMIT).map_err(From::from)
}

/// Deserialise a Decodable type directly from a Read with max size limit specified.
pub fn deserialise_from_with_limit<R: Read, T: Decodable>(read: &mut R,
                                                          size_limit: SizeLimit)
                                                          -> Result<T, SerialisationError> {
    decode_from(read, size_limit).map_err(From::from)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::io::Cursor;
    use bincode::SizeLimit;
    use bincode::rustc_serialize::{DecodingError, encode, EncodingError};

    #[test]
    fn serialise_deserialise() {
        let original_data = (vec![0u8, 1, 3, 9], vec![-1i64, 888, -8765], "Some-String".to_owned());

        let serialised_data = unwrap!(serialise(&original_data));
        let deserialised_data: (Vec<u8>, Vec<i64>, String) =
            unwrap!(deserialise(&serialised_data));
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
        let upper_limit = match super::UPPER_LIMIT {
            SizeLimit::Bounded(limit) => limit,
            SizeLimit::Infinite => panic!("Test expects a bounded limit"),
        };
        // Test with data which is at limit
        let mut original_data = (1u64..(upper_limit / 8)).collect::<Vec<_>>();
        let mut serialised_data = unwrap!(serialise(&original_data));
        let mut deserialised_data: Vec<u64> = unwrap!(deserialise(&serialised_data));
        assert!(original_data == deserialised_data);

        serialised_data.clear();
        unwrap!(serialise_into(&original_data, &mut serialised_data));
        let mut serialised = Cursor::new(serialised_data);
        deserialised_data = unwrap!(deserialise_from(&mut serialised));
        assert_eq!(original_data, deserialised_data);

        // Try to serialise data above limit
        original_data.push(0);
        if let Err(SerialisationError::Serialise(EncodingError::SizeLimit)) =
               serialise(&original_data) {} else {
            panic!("Expected size limit error.");
        }
        let mut buffer = vec![];
        if let Err(SerialisationError::Serialise(EncodingError::SizeLimit)) =
               serialise_into(&original_data, &mut buffer) {} else {
            panic!("Expected size limit error.");
        }

        // Try to deserialise data above limit
        let excessive = unwrap!(encode(&original_data, SizeLimit::Infinite));
        if let Err(SerialisationError::Deserialise(DecodingError::SizeLimit)) =
               deserialise::<Vec<u64>>(&excessive) {} else {
            panic!("Expected size limit error.");
        }
        serialised = Cursor::new(excessive);
        if let Err(SerialisationError::Deserialise(DecodingError::SizeLimit)) =
               deserialise_from::<Cursor<_>, Vec<u64>>(&mut serialised) {} else {
            panic!("Expected size limit error.");
        }

        // Try to serialise data above default limit with size limit specified
        serialised_data = unwrap!(serialise_with_limit(&original_data, SizeLimit::Infinite));

        buffer = Vec::with_capacity(serialised_data.len());
        unwrap!(serialise_into_with_limit(&original_data, &mut buffer, SizeLimit::Infinite));

        assert_eq!(serialised_data, buffer);

        // Try to deserialise data above default limit with size limit specified
        let deserialised_data_0: Vec<u64> = unwrap!(deserialise_with_limit(&serialised_data,
                                                                        SizeLimit::Infinite));
        assert_eq!(original_data, deserialised_data_0);
        let deserialised_data_1: Vec<u64> =
            unwrap!(deserialise_from_with_limit(&mut Cursor::new(buffer),
                                                SizeLimit::Infinite));
        assert_eq!(original_data, deserialised_data_1);
    }
}
