pub trait OptionExt<T, E> {
    fn ok(self) -> Result<Option<T>, E>;
}

impl<T, E> OptionExt<T, E> for Option<Result<T, E>> {
    fn ok(self) -> Result<Option<T>, E> {
        self.map_or(Ok(None), |r| r.map(Some))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct Error;

    #[test]
    pub fn ok_with_none_returns_ok_of_some() {
        let input: Option<Result<(), Error>> = None;
        let result = input.ok();
        assert!(result.is_ok());
        assert_eq!(None, result.unwrap());
    }

    #[test]
    pub fn ok_with_some_ok_returns_ok_of_some() {
        let input: Option<Result<_, Error>> = Some(Ok(42));
        let result = input.ok();
        assert!(result.is_ok());
        assert_eq!(Some(42), result.unwrap());
    }

    #[test]
    pub fn ok_with_some_err_returns_err_of_some() {
        let input: Option<Result<(), Error>> = Some(Err(Error));
        let result = input.ok();
        assert!(result.is_err());
        assert_eq!(Error, result.unwrap_err());
    }
}
