// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::fmt::Display;

#[derive(Debug)]
pub struct Error {
    description: Box<str>,
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

pub trait ResultExt<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn map_err_with<D>(self, description: D) -> Result<T, Error>
    where
        D: Into<Box<str>>;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn map_err_with<D>(self, description: D) -> Result<T, Error>
    where
        D: Into<Box<str>>,
    {
        self.map_err(|e| Error::from_error(description.into(), Box::new(e)))
    }
}

impl Error {
    pub fn new(description: impl Into<Box<str>>) -> Self {
        Self { description: description.into(), source: None }
    }

    pub fn from_error(
        description: impl Into<Box<str>>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self { description: description.into(), source: Some(source) }
    }

    pub fn message(&self) -> &str {
        &self.description
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source.as_ref().map(|e| &**e as _)
    }
}

impl From<std::io::Error> for Error {
    fn from(val: std::io::Error) -> Self {
        Error::from_error(val.kind().to_string(), Box::new(val))
    }
}

#[cfg(test)]
mod test {
    use crate::error::{Error, ResultExt};

    #[test]
    fn can_display_error() {
        assert_eq!("description", format!("{}", Error::new("description")));
    }

    #[test]
    fn can_debug_error() {
        assert_eq!(
            "Error { description: \"description\", source: None }",
            format!("{:?}", Error::new("description"))
        );
    }

    #[test]
    fn can_debug_error_with_source() {
        let source = get_io_error();

        assert_eq!(
            "Error { description: \"description\", source: Some(Custom { kind: AddrInUse, error: \"Address already in use\" }) }",
            format!(
                "{:?}",
                Error::from_error(
                    "description",
                    Box::new(source)
                )
            )
        );
    }

    #[test]
    fn can_display_error_with_source() {
        assert_eq!(
            "description",
            format!(
                "{}",
                Error::from_error(
                    "description",
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::AddrInUse,
                        "Address already in use"
                    ))
                )
            )
        );
    }

    #[test]
    fn can_get_source() {
        // arrange
        let outer = Error::from_error("Something went wrong", Box::new(get_io_error()));

        // act
        let source = std::error::Error::source(&outer);

        // assert
        assert_eq!(format!("{:?}", get_io_error()), format!("{:?}", source.unwrap()));
    }

    #[test]
    fn map_expect() {
        const DESCRIPTION: &str = "Something went wrong";

        // arrange
        let result: Result<(), _> = Err(get_io_error());

        // act
        let source = result.map_err_with(DESCRIPTION);

        // assert
        assert_eq!(
            format!("{:?}", Error::from_error(DESCRIPTION, get_io_error().into())),
            format!("{:?}", source.unwrap_err())
        );
    }

    fn get_io_error() -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::AddrInUse, "Address already in use")
    }
}
