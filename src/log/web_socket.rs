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

use std::convert::From;
use std::borrow::Borrow;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use std::io::{Error, ErrorKind, Result};

use ws;
use ws::{CloseCode, Handshake, Handler, Message};

use thread::Joiner;

pub struct WebSocket {
    ws_tx: ws::Sender,
    _raii_joiner: Joiner,
}

impl WebSocket {
    pub fn new<U: Borrow<str>>(url_borrow: U) -> Result<Self> {
        let url = url_borrow.borrow().to_owned();

        let (tx, rx) = mpsc::channel();

        let joiner = thread!("WebSocketLogger", move || {
            struct Client {
                ws_tx: ws::Sender,
                tx: Sender<Result<ws::Sender>>,
            };

            impl Handler for Client {
                fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
                    if self.tx.send(Ok(self.ws_tx.clone())).is_err() {
                        Err(ws::Error {
                            kind: ws::ErrorKind::Internal,
                            details: From::from("Channel error - Could not send ws_tx."),
                        })
                    } else {
                        Ok(())
                    }
                }
            }

            let mut tx_opt = Some(tx.clone());

            match ws::connect(url, |ws_tx| {
                Client {
                    ws_tx: ws_tx,
                    tx: unwrap!(tx_opt.take(), "Logic Error! Report as bug."),
                }
            }) {
                Ok(()) => (),
                Err(e) => {
                    let _ = tx.send(Err(Error::new(ErrorKind::Other, format!("{:?}", e))));
                }
            }
        });

        match rx.recv() {
            Ok(Ok(ws_tx)) => {
                Ok(WebSocket {
                    ws_tx: ws_tx,
                    _raii_joiner: Joiner::new(joiner),
                })
            }
            Ok(Err(e)) => Err(e),
            Err(e) => Err(Error::new(ErrorKind::Other, format!("WebSocket Logger Error: {:?}", e))),
        }
    }

    pub fn write_all(&self, buf: &[u8]) -> Result<()> {
        self.ws_tx
            .send(Message::Binary(buf.to_owned()))
            .map_err(|e| Error::new(ErrorKind::Other, format!("{:?}", e)))
    }
}

impl Drop for WebSocket {
    fn drop(&mut self) {
        let _ = self.ws_tx.close(CloseCode::Normal);
    }
}
