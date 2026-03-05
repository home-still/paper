use std::process::ExitCode;

pub const GENERAL_ERROR: u8 = 1;
pub const USAGE_ERROR: u8 = 2;
pub const UNAVAILABLE: u8 = 69;
pub const TEMPFAIL: u8 = 75;
pub const CONFIG_ERROR: u8 = 78;

pub fn from_error(err: &anyhow::Error) -> ExitCode {
    for cause in err.chain() {
        if let Some(pfe) = cause.downcast_ref::<paper_fetch_core::error::PaperFetchError>() {
            use paper_fetch_core::error::PaperFetchError::*;
            return ExitCode::from(match pfe {
                InvalidInput(_) => USAGE_ERROR,
                NotFound(_) | ParseError(_) => GENERAL_ERROR,
                Http(_) | RateLimited { .. } => TEMPFAIL,
                ProviderUnavailable(_) | CircuitBreakerOpen(_) => UNAVAILABLE,
                Io(_) => GENERAL_ERROR,
                NoDownloadUrl(_) => USAGE_ERROR,
            });
        }
    }
    ExitCode::from(GENERAL_ERROR)
}
