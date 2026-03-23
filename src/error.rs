use mawu::errors::MawuError;
use nabu::error::NabuError;

#[derive(Debug, Clone, PartialEq)]
pub enum ArgosError {
    Environment(String),
    SetupProcess(String),
    SetupRepoConfig(String),
    IntegrateRepo(String),
    IntegrateRepoTest(String),
    IntegrateRepoLicense(String),
    Json(String),
    Git(String),
    XffValue(String),
    Xff(String),
}

impl std::fmt::Display for ArgosError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgosError::Environment(e) => write!(f, "Environment Error: {e}"),
            ArgosError::SetupProcess(e) => write!(f, "Setup Process Error: {e}"),
            ArgosError::SetupRepoConfig(e) => write!(f, "Setup Repo Config Error: {e}"),
            ArgosError::IntegrateRepo(e) => write!(f, "Integrate Repo Error: {e}"),
            ArgosError::IntegrateRepoTest(e) => write!(f, "Integrate Repo Test Error: {e}"),
            ArgosError::IntegrateRepoLicense(e) => {
                write!(f, "Integrate Repo License Error: {e}")
            }
            ArgosError::Json(e) => write!(f, "JSON Error: {e}"),
            ArgosError::Git(e) => write!(f, "Git Error: {e}"),
            ArgosError::XffValue(e) => write!(f, "XFF Value Error: {e}"),
            ArgosError::Xff(e) => write!(f, "XFF Error: {e}"),
        }
    }
}

impl From<MawuError> for ArgosError {
    fn from(e: MawuError) -> Self {
        ArgosError::Json(e.to_string())
    }
}

impl From<NabuError> for ArgosError {
    fn from(e: NabuError) -> Self {
        ArgosError::Xff(e.to_string())
    }
}

impl std::error::Error for ArgosError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

pub type ArgosResult<T> = std::result::Result<T, ArgosError>;
