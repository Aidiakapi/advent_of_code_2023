pub type Result<T> = std::result::Result<T, crate::error::Error>;

pub auto trait IsNotResult {}
impl<T> !IsNotResult for Result<T> {}

pub trait IntoResult {
    type Output;
    fn into_result(self) -> Self::Output;
}

impl<T: IsNotResult> IntoResult for T {
    type Output = Result<T>;
    fn into_result(self) -> Result<T> {
        Ok(self)
    }
}
impl<T> IntoResult for Result<T> {
    type Output = Result<T>;
    fn into_result(self) -> Result<T> {
        self
    }
}
