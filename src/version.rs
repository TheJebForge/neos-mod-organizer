use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Debug)]
pub struct Version {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
    pub revision: u16,
    pub suffix: String
}

fn find_suffix(ver: &str) -> Option<usize> {
    for (index, char) in ver.char_indices() {
        if !char.is_digit(10) && char != '.' {
            return Some(index);
        }
    }

    None
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ver, suffix) = if let Some(index) = find_suffix(s) {
            (s[..index].to_string(), s[index..].to_string())
        } else {
            (s.to_string(), "".to_string())
        };

        let mut pieces = ver.split(".");

        let major = if let Some(major_str) = pieces.next() {
            major_str.parse::<u16>()?
        } else {
            return Err(VersionError::MissingMajorVersion)
        };

        let minor = pieces.next().map_or_else(|| 0_u16, |v| v.parse().unwrap_or_else(|_| 0_u16));
        let patch = pieces.next().map_or_else(|| 0_u16, |v| v.parse().unwrap_or_else(|_| 0_u16));
        let revision = pieces.next().map_or_else(|| 0_u16, |v| v.parse().unwrap_or_else(|_| 0_u16));

        Ok(Self {
            major,
            minor,
            patch,
            revision,
            suffix,
        })
    }
}

#[derive(Debug)]
pub enum VersionError {
    MissingMajorVersion,
    ParseIntError(ParseIntError)
}

impl Display for VersionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for VersionError {}

impl From<ParseIntError> for VersionError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}