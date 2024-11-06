use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum Period {
    M1 = 1,
    M2 = 2,
    M3 = 3,
    M4 = 4,
    M5 = 5,
    M10 = 6,
    M15 = 7,
    M30 = 8,
    H1 = 9,
    H4 = 10,
    H12 = 11,
    D1 = 12,
    W1 = 13,
    Mn1 = 14,
}

impl Period {
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Period::M1 => "M1",
            Period::M2 => "M2",
            Period::M3 => "M3",
            Period::M4 => "M4",
            Period::M5 => "M5",
            Period::M10 => "M10",
            Period::M15 => "M15",
            Period::M30 => "M30",
            Period::H1 => "H1",
            Period::H4 => "H4",
            Period::H12 => "H12",
            Period::D1 => "D1",
            Period::W1 => "W1",
            Period::Mn1 => "MN1",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        let value = value.to_uppercase().trim().to_owned();
        match value.as_str() {
            "M1" => Some(Self::M1),
            "M2" => Some(Self::M2),
            "M3" => Some(Self::M3),
            "M4" => Some(Self::M4),
            "M5" => Some(Self::M5),
            "M10" => Some(Self::M10),
            "M15" => Some(Self::M15),
            "M30" => Some(Self::M30),
            "H1" => Some(Self::H1),
            "H4" => Some(Self::H4),
            "H12" => Some(Self::H12),
            "D1" => Some(Self::D1),
            "W1" => Some(Self::W1),
            "MN1" => Some(Self::Mn1),
            _ => None,
        }
    }
}

impl ToString for Period {
    fn to_string(&self) -> String {
        self.as_str_name().to_string()
    }
}

pub struct PeriodParseError {
    err: String,
}

impl std::fmt::Debug for PeriodParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl std::fmt::Display for PeriodParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl std::error::Error for PeriodParseError {}

impl FromStr for Period {
    type Err = PeriodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use PeriodParseError as E;
        let r = Period::from_str_name(s);
        match r {
            Some(p) => Ok(p),
            _ => Err(E {
                err: format!(
                    "expected `M1/M2/M3/M4/M5/M10/M15/M30/H1/H4/H12/D1/W1/MN1`, found \"{s}\""
                ),
            }),
        }
    }
}
