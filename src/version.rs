use std::cmp::Ordering;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::num::ParseIntError;
use std::str::FromStr;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Visitor;

#[derive(Debug, Hash, Clone)]
pub struct Version {
    major: u16,
    minor: Option<u16>,
    patch: Option<u16>,
    revision: Option<u16>,
    pub suffix: Option<String>
}

impl Version {
    pub fn zero() -> Self {
        Self {
            major: 0,
            minor: None,
            patch: None,
            revision: None,
            suffix: None,
        }
    }
    
    pub fn from_major(major: u16) -> Self {
        Self {
            major,
            minor: None,
            patch: None,
            revision: None,
            suffix: None,
        }
    }

    pub fn from_minor(major: u16, minor: u16) -> Self {
        Self {
            major,
            minor: Some(minor),
            patch: None,
            revision: None,
            suffix: None,
        }
    }

    pub fn from_patch(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor: Some(minor),
            patch: Some(patch),
            revision: None,
            suffix: None,
        }
    }

    pub fn from_revision(major: u16, minor: u16, patch: u16, revision: u16) -> Self {
        Self {
            major,
            minor: Some(minor),
            patch: Some(patch),
            revision: Some(revision),
            suffix: None,
        }
    }

    pub fn from_suffix(major: u16, minor: u16, patch: u16, revision: u16, suffix: &str) -> Self {
        Self {
            major,
            minor: Some(minor),
            patch: Some(patch),
            revision: Some(revision),
            suffix: Some(suffix.to_string()),
        }
    }

    pub fn has_revision(&self) -> bool {
        self.revision.is_some()
    }

    pub fn revision(&self) -> u16 {
        self.revision.unwrap_or_else(|| 0)
    }

    pub fn has_patch(&self) -> bool {
        self.patch.is_some()
    }

    pub fn patch(&self) -> u16 {
        self.patch.unwrap_or_else(|| 0)
    }

    pub fn has_minor(&self) -> bool {
        self.minor.is_some()
    }

    pub fn minor(&self) -> u16 {
        self.minor.unwrap_or_else(|| 0)
    }

    pub fn major(&self) -> u16 {
        self.major
    }
}

impl Default for Version {
    fn default() -> Self {
        Self {
            major: 1,
            minor: None,
            patch: None,
            revision: None,
            suffix: None,
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.major)?;

        if let Some(v) = self.minor {
            write!(f, ".{}", v)?;
        }

        if let Some(v) = self.patch {
            write!(f, ".{}", v)?;
        }

        if let Some(v) = self.revision {
            write!(f, ".{}", v)?;
        }

        if let Some(v) = &self.suffix {
            write!(f, "{}", v)?;
        }

        Ok(())
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor.unwrap_or_else(|| 0) == other.minor.unwrap_or_else(|| 0)
            && self.patch.unwrap_or_else(|| 0) == other.patch.unwrap_or_else(|| 0)
            && self.revision.unwrap_or_else(|| 0) == other.revision.unwrap_or_else(|| 0)
            && self.suffix == other.suffix
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Version {}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.major != other.major {
            return self.major.cmp(&other.major)
        }

        let lhs_minor = self.minor.unwrap_or_else(|| 0);
        let rhs_minor = other.minor.unwrap_or_else(|| 0);

        if lhs_minor != rhs_minor {
            return lhs_minor.cmp(&rhs_minor)
        }

        let lhs_patch = self.patch.unwrap_or_else(|| 0);
        let rhs_patch = other.patch.unwrap_or_else(|| 0);

        if lhs_patch != rhs_patch {
            return lhs_patch.cmp(&rhs_patch)
        }

        let lhs_revision = self.revision.unwrap_or_else(|| 0);
        let rhs_revision = other.revision.unwrap_or_else(|| 0);

        if lhs_revision != rhs_revision {
            return lhs_revision.cmp(&rhs_revision)
        }

        self.suffix.cmp(&other.suffix)
    }
}

fn find_suffix(ver: &str) -> Option<usize> {
    for (index, char) in ver.char_indices() {
        if !char.is_digit(10) && char != '.' && char != '*' {
            return Some(index);
        }
    }

    None
}

impl FromStr for Version {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (ver, suffix) = if let Some(index) = find_suffix(s) {
            (s[..index].to_string(), Some(s[index..].to_string()))
        } else {
            (s.to_string(), None)
        };

        let mut pieces = ver.split(".");

        let major = if let Some(major_str) = pieces.next() {
            major_str.parse::<u16>()?
        } else {
            return Err(VersionError::MissingMajorVersion)
        };

        let minor = pieces.next().map_or_else(|| None, |v| v.parse().ok());
        let patch = pieces.next().map_or_else(|| None, |v| v.parse().ok());
        let revision = pieces.next().map_or_else(|| None, |v| v.parse().ok());

        Ok(Self {
            major,
            minor,
            patch,
            revision,
            suffix,
        })
    }
}

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
pub struct VersionReq {
    comparators: Vec<Comparator>
}

impl VersionReq {
    pub fn matches(&self, version: &Version) -> bool {
        self.comparators.iter()
            .all(|x| x.matches(version))
    }
}

impl Display for VersionReq {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.comparators.iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", "))
    }
}

impl FromStr for VersionReq {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            comparators: s.split(",")
                .map(|x| Comparator::from_str(x.trim()))
                .collect::<Result<Vec<Comparator>, Self::Err>>()?
        })
    }
}

#[derive(Debug, Hash, Clone, Eq, PartialEq)]
pub struct Comparator {
    version: Version,
    op: VersionOp
}

impl Comparator {
    pub fn matches(&self, version: &Version) -> bool {
        match self.op {
            VersionOp::Exact | VersionOp::Wildcard => {
                match () {
                    _ if self.version.has_revision() => {
                        version == &self.version
                    }

                    _ if self.version.has_patch() => {
                        let patch = self.version.patch();

                        version >= &Version::from_patch(self.version.major, self.version.minor(), patch)
                            && version < &Version::from_patch(self.version.major, self.version.minor(), patch + 1)
                    }

                    _ if self.version.has_minor() => {
                        let minor = self.version.minor();

                        version >= &Version::from_minor(self.version.major, minor)
                            && version < &Version::from_minor(self.version.major, minor + 1)
                    }

                    _ => {
                        version >= &Version::from_major(self.version.major)
                            && version < &Version::from_major(self.version.major + 1)
                    }
                }
            }

            VersionOp::Greater => {
                match () {
                    _ if self.version.has_revision() => {
                        version > &self.version
                    }

                    _ if self.version.has_patch() => {
                        version >= &Version::from_patch(self.version.major(), self.version.minor(), self.version.patch() + 1)
                    }

                    _ if self.version.has_minor() => {
                        version >= &Version::from_minor(self.version.major(), self.version.minor() + 1)
                    }

                    _ => {
                        version >= &Version::from_major(self.version.major + 1)
                    }
                }
            }

            VersionOp::GreaterEq => {
                version >= &self.version
            }

            VersionOp::Less => {
                version < &self.version
            }

            VersionOp::LessEq => {
                match () {
                    _ if self.version.has_revision() => {
                        version < &self.version
                    }

                    _ if self.version.has_patch() => {
                        version < &Version::from_patch(self.version.major(), self.version.minor(), self.version.patch() + 1)
                    }

                    _ if self.version.has_minor() => {
                        version < &Version::from_minor(self.version.major(), self.version.minor() + 1)
                    }

                    _ => {
                        version < &Version::from_major(self.version.major + 1)
                    }
                }
            }

            VersionOp::Tilde => {
                match () {
                    _ if self.version.has_revision() => {
                        let minor = self.version.minor();

                        version >= &Version::from_revision(self.version.major, minor, self.version.patch(), self.version.revision())
                            && version < &Version::from_minor(self.version.major, minor + 1)
                    }

                    _ if self.version.has_patch() => {
                        let minor = self.version.minor();

                        version >= &Version::from_patch(self.version.major, minor, self.version.patch())
                            && version < &Version::from_minor(self.version.major, minor + 1)
                    }

                    _ if self.version.has_minor() => {
                        let minor = self.version.minor();

                        version >= &Version::from_minor(self.version.major, minor)
                            && version < &Version::from_minor(self.version.major, minor + 1)
                    }

                    _ => {
                        version >= &Version::from_major(self.version.major)
                            && version < &Version::from_major(self.version.major + 1)
                    }
                }
            }

            VersionOp::Caret => {
                match () {
                    _ if self.version.major() > 0 => {
                        version >= &self.version
                            && version < &Version::from_major(self.version.major() + 1)
                    }

                    _ if self.version.minor() > 0 => {
                        version >= &self.version
                            && version < &Version::from_minor(0, self.version.minor() + 1)
                    }

                    _ if self.version.patch() > 0 => {
                        version >= &self.version
                            && version < &Version::from_patch(0, 0, self.version.patch() + 1)
                    }

                    _ => {
                        version == &self.version
                    }
                }
            }

            VersionOp::WildcardAny => true,
        }
    }
}

impl FromStr for Comparator {
    type Err = VersionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "*" {
            return Ok(Self {
                version: Default::default(),
                op: VersionOp::WildcardAny,
            })
        }

        match () {
            _ if s.starts_with('=') => {
                Ok(Self {
                    version: s[1..].parse()?,
                    op: VersionOp::Exact,
                })
            }

            _ if s.starts_with(">=") => {
                Ok(Self {
                    version: s[2..].parse()?,
                    op: VersionOp::GreaterEq,
                })
            }

            _ if s.starts_with('>') => {
                Ok(Self {
                    version: s[1..].parse()?,
                    op: VersionOp::Greater,
                })
            }

            _ if s.starts_with("<=") => {
                Ok(Self {
                    version: s[2..].parse()?,
                    op: VersionOp::LessEq,
                })
            }

            _ if s.starts_with('<') => {
                Ok(Self {
                    version: s[1..].parse()?,
                    op: VersionOp::Less,
                })
            }

            _ if s.starts_with('~') => {
                Ok(Self {
                    version: s[1..].parse()?,
                    op: VersionOp::Tilde,
                })
            }

            _ if s.starts_with('^') => {
                Ok(Self {
                    version: s[1..].parse()?,
                    op: VersionOp::Caret,
                })
            }

            _ if s.contains('*') => {
                Ok(Self {
                    version: s.parse()?,
                    op: VersionOp::Wildcard,
                })
            }

            _ => Ok(Self {
                version: s.parse()?,
                op: VersionOp::Exact,
            })
        }
    }
}

impl Display for Comparator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.op {
            VersionOp::Exact => write!(f, "={}", self.version),
            VersionOp::Greater => write!(f, ">{}", self.version),
            VersionOp::GreaterEq => write!(f, ">={}", self.version),
            VersionOp::Less => write!(f, "<{}", self.version),
            VersionOp::LessEq => write!(f, "<={}", self.version),
            VersionOp::Tilde => write!(f, "~{}", self.version),
            VersionOp::Caret => write!(f, "^{}", self.version),
            VersionOp::Wildcard => write!(f, "{}.*", self.version),
            VersionOp::WildcardAny => write!(f, "*"),
        }
    }
}

#[derive(Debug, Hash, Copy, Clone, Eq, PartialEq)]
pub enum VersionOp {
    /// - `=A.I.P.R` - exactly version A.I.P.R
    /// - `=A.I.P` - same as `>=A.I.P.0, <A.I.(P+1).0`
    /// - `=A.I` - same as `>=A.I.0.0, <A.(I+1).0.0`
    /// - `=A` - same as `>=A.0.0.0, <(A+1).0.0.0`
    /// - Any version requirement without an operator defaults to this
    Exact,

    /// - `>A.I.P.R`
    /// - `>A.I.P` - same as `>=A.I.(P+1).0`
    /// - `>A.I` - same as `>=A.(I+1).0.0`
    /// - `>A` - same as `>=(A+1).0.0.0`
    Greater,

    /// - `>=A.I.P.R`
    /// - `>=A.I.P` - same as `>=A.I.P.0`
    /// - `>=A.I` - same as `>=A.I.0.0`
    /// - `>=A` - same as `>=A.0.0.0`
    GreaterEq,

    /// - `<A.I.P.R`
    /// - `<A.I.P` - same as `<A.I.P.0`
    /// - `<A.I` - same as `<A.I.0.0`
    /// - `<A` - same as `<A.0.0.0`
    Less,

    /// - `<=A.I.P.R`
    /// - `<=A.I.P` - same as `<A.I.(P+1).0`
    /// - `<=A.I` - same as `<A.(I+1).0.0`
    /// - `<=A` - same as `<(A+1).0.0.0`
    LessEq,

    /// Tilde requirements allow the patch and revision parts of the version (the third and forth number) to increase
    /// - `~A.I.P.R` - same as `>=A.I.P.R, <A.(I+1).0.0`
    /// - `~A.I.P` - same as `>=A.I.P.0, <A.(I+1).0.0`
    /// - `~A.I` - same as `=A.I`
    /// - `~A` - same as `=A`
    Tilde,

    /// Caret requirements allow parts that are *right of the first nonzero* part of the version to increase
    /// - `^A.I.P.R` (for A>0) - same as `>=A.I.P.R, <(A+1).0.0.0`
    /// - `^0.I.P.R` (for I>0) - same as `>=0.I.P.R, <0.(I+1).0.0`
    /// - `^0.0.P.R` (for P>0) - same as `>=0.0.P.R, <0.0.(P+1).0`
    /// - `^0.0.0.R` - same as `=0.0.0.R`
    /// - `^A.I.P` (for A>0, I>0 or P>0) - same as `^A.I.P.0`
    /// - `^A.I` (for A>0 or I>0) - same as `^A.I.0.0`
    /// - `^0.0` - same as `=0.0`
    /// - `^A` - same as `=A`
    Caret,

    /// - `A.I.P.*` - same as `=A.I.P`
    /// - `A.I.*` or `A.I.*.*` - same as `=A.I`
    /// - `A.*` or `A.*.*` or `A.*.*.*` - same as `=A`
    Wildcard,

    /// - `*` - same as `>=0.0.0.0`
    WildcardAny,
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

impl Serialize for Version {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for VersionReq {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl Serialize for Comparator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct VersionVisitor;
        
        impl Visitor<'_> for VersionVisitor {
            type Value = Version;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "version string formatted with numbers and dots")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
                Self::Value::from_str(v).map_err(|e| E::custom(e))
            }
        }

        deserializer.deserialize_str(VersionVisitor)
    }
}

impl<'de> Deserialize<'de> for VersionReq {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct VersionReqVisitor;

        impl Visitor<'_> for VersionReqVisitor {
            type Value = VersionReq;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "comparator strings formatted with numbers and dots and starting with operator and separated with comma")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
                Self::Value::from_str(v).map_err(|e| E::custom(e))
            }
        }

        deserializer.deserialize_str(VersionReqVisitor)
    }
}

impl<'de> Deserialize<'de> for Comparator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct ComparatorVisitor;

        impl Visitor<'_> for ComparatorVisitor {
            type Value = Comparator;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "version requirement string formatted with numbers and dots and starting with operator")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error {
                Self::Value::from_str(v).map_err(|e| E::custom(e))
            }
        }

        deserializer.deserialize_str(ComparatorVisitor)
    }
}