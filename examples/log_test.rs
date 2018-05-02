// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under The General Public License (GPL), version 3.
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied. Please review the Licences for the specific language governing
// permissions and limitations relating to use of the SAFE Network Software.

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
