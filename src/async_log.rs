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

// TODO: consider contributing this code to the log4rs crate.

use log4rs::Append;
use log4rs::pattern::PatternLayout;
use log4rs::toml::CreateAppender;
use logger::LogRecord;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::{self, Stdout, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::net::{SocketAddr, TcpStream};
use std::str::FromStr;

use toml::{Table, Value};

/// Appender that writes to the stdout asynchronously.
pub struct AsyncConsoleAppender;

impl AsyncConsoleAppender {
    pub fn builder() -> AsyncConsoleAppenderBuilder {
        AsyncConsoleAppenderBuilder { pattern: PatternLayout::default() }
    }
}

pub struct AsyncConsoleAppenderBuilder {
    pattern: PatternLayout,
}

impl AsyncConsoleAppenderBuilder {
    pub fn pattern(self, pattern: PatternLayout) -> Self {
        AsyncConsoleAppenderBuilder { pattern: pattern }
    }

    pub fn build(self) -> AsyncAppender {
        AsyncAppender::new(io::stdout(), self.pattern)
    }
}

/// Appender that writes to a file asynchronously.
pub struct AsyncFileAppender;

impl AsyncFileAppender {
    pub fn builder<P: AsRef<Path>>(path: P) -> AsyncFileAppenderBuilder {
        AsyncFileAppenderBuilder {
            path: path.as_ref().to_path_buf(),
            pattern: PatternLayout::default(),
            append: true,
        }
    }
}

pub struct AsyncFileAppenderBuilder {
    path: PathBuf,
    pattern: PatternLayout,
    append: bool,
}

impl AsyncFileAppenderBuilder {
    pub fn pattern(self, pattern: PatternLayout) -> Self {
        AsyncFileAppenderBuilder {
            path: self.path,
            pattern: pattern,
            append: self.append,
        }
    }

    pub fn append(self, append: bool) -> Self {
        AsyncFileAppenderBuilder {
            path: self.path,
            pattern: self.pattern,
            append: append,
        }
    }

    pub fn build(self) -> io::Result<AsyncAppender> {
        let file = try!(OpenOptions::new()
                            .write(true)
                            .append(self.append)
                            .create(true)
                            .open(self.path));

        Ok(AsyncAppender::new(file, self.pattern))
    }
}

/// Creator for AsyncConsoleAppender
pub struct AsyncConsoleAppenderCreator;

impl CreateAppender for AsyncConsoleAppenderCreator {
    fn create_appender(&self, mut config: Table) -> Result<Box<Append>, Box<Error>> {
        let pattern = try!(parse_pattern(&mut config));
        Ok(Box::new(AsyncConsoleAppender::builder().pattern(pattern).build()))
    }
}

/// Creator for AsyncFileAppender
pub struct AsyncFileAppenderCreator;

impl CreateAppender for AsyncFileAppenderCreator {
    fn create_appender(&self, mut config: Table) -> Result<Box<Append>, Box<Error>> {
        let path = match config.remove("path") {
            Some(Value::String(path)) => path,
            Some(_) => return Err(Box::new(ConfigError("`path` must be a string".to_owned()))),
            None => return Err(Box::new(ConfigError("`path` is required".to_owned()))),
        };

        let append = match config.remove("append") {
            Some(Value::Boolean(append)) => append,
            Some(_) => return Err(Box::new(ConfigError("`append` must be a bool".to_owned()))),
            None => true,
        };

        let pattern = try!(parse_pattern(&mut config));
        let appender = try!(AsyncFileAppender::builder(path)
                                .pattern(pattern)
                                .append(append)
                                .build());

        Ok(Box::new(appender))
    }
}

/// Creator for AsyncServerAppender
pub struct AsyncServerAppenderCreator;

impl CreateAppender for AsyncServerAppenderCreator {
    fn create_appender(&self, mut config: Table) -> Result<Box<Append>, Box<Error>> {
        let server_addr = match config.remove("server_addr") {
            Some(Value::String(addr)) => try!(SocketAddr::from_str(&addr[..])),
            Some(_) => return Err(Box::new(ConfigError("`server_addr` must be a string".to_owned()))),
            None => return Err(Box::new(ConfigError("`server_addr` is required".to_owned()))),
        };
        let pattern = try!(parse_pattern(&mut config));

        let stream = try!(TcpStream::connect(server_addr));
        Ok(Box::new(AsyncAppender::new(stream, pattern)))
    }
}

fn parse_pattern(config: &mut Table) -> Result<PatternLayout, Box<Error>> {
    match config.remove("pattern") {
        Some(Value::String(pattern)) => Ok(try!(PatternLayout::new(&pattern))),
        Some(_) => return Err(Box::new(ConfigError("`pattern` must be a string".to_owned()))),
        None => Ok(PatternLayout::default()),
    }
}

#[derive(Debug)]
struct ConfigError(String);

impl Error for ConfigError {
    fn description(&self) -> &str {
        &self.0
    }
}

impl Display for ConfigError {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.write_str(&self.0)
    }
}

pub struct AsyncAppender {
    pattern: PatternLayout,
    sender: Sender<Vec<u8>>,
}

impl AsyncAppender {
    fn new<W: 'static + SyncWrite + Send>(mut writer: W, pattern: PatternLayout) -> Self {
        let (sender, receiver) = mpsc::channel::<Vec<u8>>();

        let _ = thread::spawn(move || {
            for message in receiver.iter() {
                // TODO: how should we handle errors here?
                let _ = writer.sync_write(&message);
            }
        });

        AsyncAppender {
            pattern: pattern,
            sender: sender,
        }
    }
}

impl Append for AsyncAppender {
    fn append(&mut self, record: &LogRecord) -> Result<(), Box<Error>> {
        let mut message = Vec::new();
        try!(self.pattern.append(&mut message, record));
        try!(self.sender.send(message));
        Ok(())
    }
}

trait SyncWrite {
    fn sync_write(&mut self, buf: &[u8]) -> io::Result<()>;
}

impl SyncWrite for Stdout {
    fn sync_write(&mut self, buf: &[u8]) -> io::Result<()> {
        let mut out = self.lock();
        try!(out.write_all(buf));
        try!(out.flush());
        Ok(())
    }
}

impl SyncWrite for File {
    fn sync_write(&mut self, buf: &[u8]) -> io::Result<()> {
        try!(self.write_all(buf));
        try!(self.flush());
        Ok(())
    }
}

impl SyncWrite for TcpStream {
    fn sync_write(&mut self, buf: &[u8]) -> io::Result<()> {
        let _ = try!(self.write_all(buf));
        println!("Written");
        Ok(())
    }
}
