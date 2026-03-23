use mawu::errors::MawuError;
use nabu::error::NabuError;

#[derive(Debug, Clone, PartialEq)]
pub enum ArgosError {
    EnvironmentError(String),
    SetupProcessError(String),
    SetupRepoError(String),
    SetupRepoConfigError(String),
    IntegrateRepoError(String),
    IntegrateRepoBuildError(String),
    IntegrateRepoTestError(String),
    IntegrateRepoDocError(String),
    IntegrateRepoDocTestError(String),
    IntegrateRepoClippyError(String),
    IntegrateRepoFormatError(String),
    IntegrateRepoUpdateError(String),
    IntegrateRepoLicenseError(String),
    JsonError(String),
    GitError(String),
    XffValueError(String),
    XffError(String),
}

impl std::fmt::Display for ArgosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgosError::EnvironmentError(e) => write!(f, "Environment Error: {}", e),
            ArgosError::SetupProcessError(e) => write!(f, "Setup Process Error: {}", e),
            ArgosError::SetupRepoError(e) => write!(f, "Setup Repo Error: {}", e),
            ArgosError::SetupRepoConfigError(e) => write!(f, "Setup Repo Config Error: {}", e),
            ArgosError::IntegrateRepoError(e) => write!(f, "Integrate Repo Error: {}", e),
            ArgosError::IntegrateRepoBuildError(e) => write!(f, "Integrate Repo Build Error: {}", e),
            ArgosError::IntegrateRepoTestError(e) => write!(f, "Integrate Repo Test Error: {}", e),
            ArgosError::IntegrateRepoDocError(e) => write!(f, "Integrate Repo Doc Error: {}", e),
            ArgosError::IntegrateRepoDocTestError(e) => {
                write!(f, "Integrate Repo Doc Test Error: {}", e)
            }
            ArgosError::IntegrateRepoClippyError(e) => {
                write!(f, "Integrate Repo Clippy Error: {}", e)
            }
            ArgosError::IntegrateRepoFormatError(e) => {
                write!(f, "Integrate Repo Format Error: {}", e)
            }
            ArgosError::IntegrateRepoUpdateError(e) => {
                write!(f, "Integrate Repo Update Error: {}", e)
            }
            ArgosError::IntegrateRepoLicenseError(e) => {
                write!(f, "Integrate Repo License Error: {}", e)
            }
            ArgosError::JsonError(e) => write!(f, "JSON Error: {}", e),
            ArgosError::GitError(e) => write!(f, "Git Error: {}", e),
            ArgosError::XffValueError(e) => write!(f, "XFF Value Error: {}", e),
            ArgosError::XffError(e) => write!(f, "XFF Error: {}", e),
        }
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

impl std::error::Error for ArgosError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub type ArgosResult<T> = std::result::Result<T, ArgosError>;
