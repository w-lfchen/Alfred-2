#[derive(Debug)]
pub struct NoDolphinError;
impl std::fmt::Display for NoDolphinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no dolphin found")
    }
}
impl std::error::Error for NoDolphinError {}
