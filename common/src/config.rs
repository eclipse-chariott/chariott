// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::fmt::Debug;
use std::str::FromStr;

/// Utility function to read environment variables.
pub fn env<T>(key: &str) -> Option<T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    #[cfg(not(test))]
    use std::env::var_os;
    #[cfg(test)]
    use tests::var_os;

    var_os(key).map(|s| T::from_str(s.to_str().unwrap()).unwrap())
}

/// Utility function to read environment variables.
pub fn try_env<T>(key: &str) -> Option<Result<T, <T as FromStr>::Err>>
where
    T: FromStr,
{
    #[cfg(not(test))]
    use std::env::var_os;
    #[cfg(test)]
    use tests::var_os;

    var_os(key).map(|s| T::from_str(s.to_str().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const STR_VARIABLE_NAME: &str = "STR";
    const STR_VARIABLE_VALUE: &str = "foobar";
    const INT_VARIABLE_NAME: &str = "INT";
    const INT_VARIABLE_VALUE: u32 = 42;

    /// Mocks out the environment so that we avoid concurrency issues
    /// while setting environment variables.
    pub(crate) fn var_os(key: &str) -> Option<std::ffi::OsString> {
        match key {
            STR_VARIABLE_NAME => Some(STR_VARIABLE_VALUE.into()),
            INT_VARIABLE_NAME => Some(format!("{INT_VARIABLE_VALUE}").into()),
            _ => None,
        }
    }

    #[test]
    fn env_variable_is_found() {
        let result: String = env(STR_VARIABLE_NAME).unwrap();
        assert_eq!(result, STR_VARIABLE_VALUE);
    }

    #[test]
    fn env_variable_not_set() {
        let result: Option<String> = env("something_else");
        assert_eq!(result, None);
    }

    #[test]
    fn try_env_variable_is_found() {
        let result: String = try_env(STR_VARIABLE_NAME).unwrap().unwrap();
        assert_eq!(result, STR_VARIABLE_VALUE);
    }

    #[test]
    fn try_env_variable_not_set() {
        let result: Option<Result<String, _>> = try_env("something_else");
        assert_eq!(result, None);
    }

    #[test]
    fn try_env_variable_is_expected_type() {
        let result = try_env::<u32>(INT_VARIABLE_NAME).unwrap().unwrap();
        assert_eq!(result, INT_VARIABLE_VALUE);
    }

    #[test]
    fn try_env_variable_is_not_expected_type() {
        use std::num::IntErrorKind::*;
        let error = {
            let result = try_env::<u32>(STR_VARIABLE_NAME).unwrap();
            result.unwrap_err()
        };
        assert_eq!(&InvalidDigit, error.kind());
    }
}
