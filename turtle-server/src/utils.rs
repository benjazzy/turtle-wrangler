pub trait FlattenExt<R> {
    fn nested_flatten(self) -> R;
}

impl<Ok, MyError, OtherError, EndError> FlattenExt<Result<Ok, EndError>>
    for Result<Result<Ok, MyError>, OtherError>
where
    EndError: From<MyError>,
    EndError: From<OtherError>,
{
    fn nested_flatten(self) -> Result<Ok, EndError> {
        self.map(|r| r.map_err(EndError::from))
            .map_err(EndError::from)
            .flatten()
    }
}
