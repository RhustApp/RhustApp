use core::panic::Location;
use std::fmt;

pub struct RhustAppError {
    pub description: String,
    pub error: Option<String>,
    pub location: String,
}

impl RhustAppError {
    const ERROR_SPACE_WIDTH: usize = 4;

    pub fn to_string(&self) -> String {
        match &self.error {
            Some(err) => format!(
                "RhustAppError {{\n\
                {padding}   location : \"{loc}\",\n\
                {padding}description : \"{des}\",\n\
                {padding}      error : \"{err}\",\n\
                }}",
                loc = &self.location,
                des = &self.description,
                err = err.replace(
                    "\n",
                    &format!("\n{}", " ".repeat(RhustAppError::ERROR_SPACE_WIDTH))
                ),
                padding = " ".repeat(RhustAppError::ERROR_SPACE_WIDTH),
            ),
            None => format!(
                "RhustAppError {{\n\
                {padding}   location : \"{loc}\",\n\
                {padding}description : \"{des}\",\n\
                }}",
                loc = &self.location,
                des = &self.description,
                padding = " ".repeat(RhustAppError::ERROR_SPACE_WIDTH),
            ),
        }
    }
}

impl fmt::Display for RhustAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl fmt::Debug for RhustAppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RhustAppError")
            .field("description", &self.description)
            .field("error", &self.error)
            .field("location", &self.location)
            .finish()
    }
}

impl From<RhustAppError> for Box<dyn std::error::Error> {
    fn from(value: RhustAppError) -> Self {
        Box::from(value.to_string())
    }
}

#[track_caller]
pub fn new_rhustapp_error(description: &str, err: Option<String>) -> RhustAppError {
    match err {
        Some(err) => RhustAppError {
            description: description.to_string(),
            error: Some(err),
            location: Location::caller().to_string(),
        },
        None => RhustAppError {
            description: description.to_string(),
            error: None,
            location: Location::caller().to_string(),
        },
    }
}
