use mawu::errors::MawuError;
use nabu::error::NabuError;

#[derive(Debug, Clone, PartialEq)]
pub enum ArgosError {
    EnvironmentError(String),
    SetupProcessError(String),
    SetupRepoError(String),
    SetupRepoConfigError(String),
    IntegrateRepoError(String),
    JsonError(String),
    GitError(String),
    XffValueError(String),
    XffError(String),
}

impl std::fmt::Display for ArgosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<MawuError> for ArgosError {
    fn from(e: MawuError) -> Self {
        ArgosError::JsonError(e.to_string())
    }
}

impl From<NabuError> for ArgosError {
    fn from(e: NabuError) -> Self {
        ArgosError::XffError(e.to_string())
    }
}

// I need this for the main functions return type
// Fucked if I know what I'm doing (cause and source) especially
impl std::error::Error for ArgosError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        None
    }
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
    fn description(&self) -> &str {
        match self {
            ArgosError::EnvironmentError(e) => e,
            ArgosError::SetupProcessError(e) => e,
            ArgosError::JsonError(e) => e,
            ArgosError::XffValueError(e) => e,
            ArgosError::XffError(e) => e,
            ArgosError::SetupRepoError(e) => e,
            ArgosError::GitError(e) => e,
            ArgosError::SetupRepoConfigError(e) => e,
            ArgosError::IntegrateRepoError(e) => e,
        }
    }
}

pub type ArgosResult<T> = std::result::Result<T, ArgosError>;
