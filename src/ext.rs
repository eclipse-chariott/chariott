pub trait OptionExt<T, E> {
    fn ok(self) -> Result<Option<T>, E>;
}

impl<T, E> OptionExt<T, E> for Option<Result<T, E>> {
    fn ok(self) -> Result<Option<T>, E> {
        self.map_or(Ok(None), |r| r.map(Some))
    }
}
