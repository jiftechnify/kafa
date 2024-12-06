type VMError = Box<dyn std::error::Error>;

pub type VMResult<T> = Result<T, VMError>;
