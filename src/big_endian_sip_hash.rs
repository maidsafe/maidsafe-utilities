// Copyright 2016 MaidSafe.net limited.
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


use bincode::{self, SizeLimit};
use rustc_serialize::Encodable;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

/// Generates a deterministic Sip hash from `data`, regardless of the endianness of the host
/// machine.
pub fn big_endian_sip_hash<T: Encodable>(data: &T) -> u64 {
    let encoded =
        bincode::rustc_serialize::encode(data, SizeLimit::Infinite).ok().unwrap_or_else(Vec::new);
    let mut hasher = DefaultHasher::new();
    hasher.write(&encoded);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_test() {
        assert_eq!(2034663721994522015, big_endian_sip_hash(&"Test".to_owned()));
        assert_eq!(17786900032859366373, big_endian_sip_hash(&[24u64, 6, 1314]));
        let mut option = Some(1);
        assert_eq!(11997493401453892861, big_endian_sip_hash(&option));
        option = None;
        assert_eq!(7541581120933061747, big_endian_sip_hash(&option));
    }
}
