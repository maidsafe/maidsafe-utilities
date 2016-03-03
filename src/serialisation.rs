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
use bincode::rustc_serialize::{decode, decode_from, DecodingError, encode, encode_into,
                               EncodingError};
use rustc_serialize::{Encodable, Decodable};
use std::io::{Read, Write};

quick_error! {
    /// Serialization error.
    #[derive(Debug)]
    pub enum SerializationError {
        /// Error during serialization (encoding).
        SerializeError(err: EncodingError) {
            description("Serialize error")
            display("Serialize error: {}", err)
            cause(err)
            from()
        }

        /// Error during deserialization (decoding).
        DeserializeError(err: DecodingError) {
            description("Deserialize error")
            display("Deserialize error: {}", err)
            cause(err)
            from()
        }
    }
}

/// Serialise an Encodable type
pub fn serialise<T>(data: &T) -> Result<Vec<u8>, SerializationError>
    where T: Encodable
{
    encode(data, SizeLimit::Infinite).map_err(From::from)
}

/// Serialize directly to a Writ
pub fn serialize_into<T: Encodable, W: Write>(data: &T,
                                              writer: &mut W)
                                              -> Result<(), SerializationError> {
    encode_into(data, writer, SizeLimit::Infinite).map_err(From::from)
}


/// Deserialise a Decodable type
pub fn deserialise<T>(data: &[u8]) -> Result<T, SerializationError>
    where T: Decodable
{
    decode(data).map_err(From::from)
}

/// Deserialize directly from a Read
pub fn deserialize_from<R: Read, T: Decodable>(reader: &mut R) -> Result<T, SerializationError> {
    decode_from(reader, SizeLimit::Infinite).map_err(From::from)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn serialise_deserialise() {
        let original_data = (vec![0u8, 1, 3, 9],
                             vec![-1i64, 888, -8765],
                             "Some-String".to_owned());

        let serialised_data = unwrap_result!(serialise(&original_data));
        let deserialised_data: (Vec<u8>, Vec<i64>, String) =
            unwrap_result!(deserialise(&serialised_data));
        assert_eq!(original_data, deserialised_data);
    }
}
