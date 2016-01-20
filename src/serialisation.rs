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

use cbor::{Encoder, Decoder};
use rustc_serialize::{Encodable, Decodable};

/// Wrapper for Serialisation Errors. This is present because cbor code paths don't always return a
/// Result - they return an Option too.
#[derive(Debug)]
pub enum SerialisationError {
    /// Encapsulated Cbor Error
    CborError(::cbor::CborError),
    /// To convert Option to a Result when deserialisation is unsuccessful and returns an
    /// `Option::None`
    UnsuccessfulDecode,
}

/// Function to serialise an Encodable type
pub fn serialise<T>(data: &T) -> Result<Vec<u8>, SerialisationError>
    where T: Encodable
{
    let mut encoder = Encoder::from_memory();
    try!(encoder.encode(&[data]));
    Ok(encoder.into_bytes())
}

/// Function to deserialise a Decodable type
pub fn deserialise<T>(data: &[u8]) -> Result<T, SerialisationError>
    where T: Decodable
{
    let mut decoder = Decoder::from_bytes(data);
    Ok(try!(try!(decoder.decode().next().ok_or(SerialisationError::UnsuccessfulDecode))))
}

impl From<::cbor::CborError> for SerialisationError {
    fn from(orig_error: ::cbor::CborError) -> Self {
        SerialisationError::CborError(orig_error)
    }
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
