use std::str::FromStr;

#[derive(Debug, Clone, Copy)]
pub enum Action {
    //Download,
    Parse,
    //Check,
    //Fx,
    //Plot,
    //Web,
    //Bi,
    //Seg,
    //Zs,
    //Bsp,
}

impl ToString for Action {
    fn to_string(&self) -> String {
        let action = match self {
            //Self::Download => "download",
            //Self::Check => "check",
            //Self::Fx => "fx",
            //Self::Plot => "plot",
            //Self::Web => "web",
            //Self::Bi => "bi",
            //Self::Seg => "seg",
            //Self::Zs => "zs",
            //Self::Bsp => "bsp",
            Self::Parse => "parse",
        };
        action.to_string()
    }
}

pub struct ActionParseError {
    err: String,
}

impl std::fmt::Debug for ActionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl std::fmt::Display for ActionParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.err.fmt(f)
    }
}

impl std::error::Error for ActionParseError {}

impl FromStr for Action {
    type Err = ActionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ActionParseError as E;
        match s {
            "parse" => Ok(Action::Parse),
            //"download" => Ok(Action::Download),
            //"check" => Ok(Action::Check),
            //"fx" => Ok(Action::Fx),
            //"plot" => Ok(Action::Plot),
            //"web" => Ok(Action::Web),
            //"bi" => Ok(Action::Bi),
            //"seg" => Ok(Action::Seg),
            //"zs" => Ok(Action::Zs),
            //"bsp" => Ok(Action::Bsp),
            _ => Err(E {
                err: format!("expected `parse` | `plot`, found \"{s}\""),
            }),
        }
    }
}
