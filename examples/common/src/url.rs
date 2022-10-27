// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{
    error::Error,
    fmt::Display,
    net::{IpAddr, SocketAddr},
};

use url::{Host, Url};

#[derive(Debug)]
pub enum UrlSocketAddrParseError {
    InvalidScheme,
    MissingHost,
    InvalidAddress,
}

impl Display for UrlSocketAddrParseError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use UrlSocketAddrParseError::*;
        fmt.write_str(match self {
            InvalidScheme => "invalid scheme",
            MissingHost => "missing host",
            InvalidAddress => "invalid address",
        })
    }
}

impl Error for UrlSocketAddrParseError {}

pub trait UrlExt {
    fn parse_socket_address(&self) -> Result<SocketAddr, UrlSocketAddrParseError>;
}

impl UrlExt for Url {
    fn parse_socket_address(&self) -> Result<SocketAddr, UrlSocketAddrParseError> {
        use UrlSocketAddrParseError::*;

        let port = match self.scheme() {
            "http" => Ok(80),
            "https" => Ok(443),
            _ => Err(InvalidScheme),
        }?;
        let mut addr = match self.host().ok_or(MissingHost)? {
            Host::Domain(_) => Err(InvalidAddress),
            Host::Ipv4(addr) => Ok(SocketAddr::new(IpAddr::V4(addr), port)),
            Host::Ipv6(addr) => Ok(SocketAddr::new(IpAddr::V6(addr), port)),
        }?;
        if let Some(port) = self.port() {
            addr.set_port(port);
        }
        Ok(addr)
    }
}
