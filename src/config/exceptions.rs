use std::fmt;

///
/// An error indicating the file could not be read
///
/// # Properties
/// * filename: Path to the file in question
///
pub(crate) struct FileError {
    pub(crate) filename: String
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "could not read file: {}", self.filename)
    }
}

///
/// A set of errors related to config properties:
/// * MissingConfigPropertyError: A required property could not be found
/// * InvalidConfigPropertyKeyError: The property was invalid or contained unexpected data
///
pub enum ConfigPropertiesError {
    MissingConfigPropertyError(MissingConfigPropertyError),
    InvalidConfigPropertyKeyError(InvalidConfigPropertyKeyError)
}

///
/// A required property could not be found
///
/// # Properties
/// * property: Key of the property in question
///
pub struct MissingConfigPropertyError {
    pub property: String
}

impl fmt::Display for MissingConfigPropertyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "property does not exist in configuration: {}", self.property.clone())
    }
}

///
/// The property was invalid or contained unexpected data
///
/// # Properties
/// * key: The property was invalid or contained unexpected data
///
pub struct InvalidConfigPropertyKeyError {
    pub key: String
}

impl fmt::Display for InvalidConfigPropertyKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid configuration properties key: {}", self.key)
    }
}