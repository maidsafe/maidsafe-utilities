// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

extern crate maidsafe_utilities;
#[macro_use]
extern crate unwrap;
extern crate ws;

use ws::{Handler, Message, Request, Response};

struct Server {
    session_id: Option<String>,
}

impl Handler for Server {
    fn on_request(&mut self, req: &Request) -> ws::Result<Response> {
        match (self.session_id.as_ref(), req.header("SessionId")) {
            (Some(exp), Some(obs)) if &obs[..] == exp.as_bytes() => ws::Response::from_request(req),
            (None, _) => ws::Response::from_request(req),
            _ => Err(ws::Error::new(ws::ErrorKind::Internal, "Invalid SessionId")),
        }
    }

    fn on_message(&mut self, message: Message) -> ws::Result<()> {
        println!("{}", message.as_text()?);
        Ok(())
    }
}

fn main() {
    unwrap!(ws::listen("127.0.0.1:55555", |_| Server {
        session_id: Some("magic-value".into())
    }));
}
