#[derive(Debug, PartialEq, Eq)]
pub struct AlreadyReadyError;

impl std::fmt::Display for AlreadyReadyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sender is already in the ready state")
    }
}

impl std::error::Error for AlreadyReadyError {}
