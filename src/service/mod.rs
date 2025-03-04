mod course;
mod format;
mod semester;
mod service;
mod status;
mod switch;


use format::{FormatType, FormatTypeable};
pub(crate) use service::Service;

pub(crate) type ServiceResult = Result<FormatType, anyhow::Error>;

impl FormatTypeable for ServiceResult {
    fn format(self) -> FormatType {
        match self {
            Ok(value) => value.format(),
            Err(err) => FormatType::Error(err.to_string()),
        }
    }
}
