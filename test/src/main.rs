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

#[macro_use]
extern crate log;
extern crate maidsafe_utilities;
#[macro_use]
extern crate unwrap;

use maidsafe_utilities::log as safe_log;
use std::thread;
use std::time::Duration;

fn main() {
    unwrap!(safe_log::init(false));

    trace!("This is a log message.");
    debug!("This is a log message.");
    info!("This is a log message.");
    warn!("This is a log message.");
    error!("This is a log message.");

    abc::log_msgs();

    // Allow async loggers to function in the background thread
    thread::sleep(Duration::from_millis(100));
}

mod abc {
    pub fn log_msgs() {
        trace!("This is a log message.");
        debug!("This is a log message.");
        info!("This is a log message.");
        warn!("This is a log message.");
        error!("This is a log message.");
    }
}
