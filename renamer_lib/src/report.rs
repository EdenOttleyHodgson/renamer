use std::borrow::Cow;
use std::fmt::Debug;
use std::path::{Path, PathBuf};

pub enum Report {
    Renamed {
        from: PathBuf,
        to: PathBuf,
        overwrote: bool,
    },
    Nothing,
}

impl Debug for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Report::Renamed {
                from,
                to,
                overwrote,
            } => {
                let mut out = format!(
                    "Renamed {} to {}",
                    format_report_path(from),
                    format_report_path(to)
                );
                if *overwrote {
                    out.push_str("(OVERWROTE)");
                }
                write!(f, "{out}")
            }
            Report::Nothing => Ok(()),
        }
    }
}

fn format_report_path<'a>(p: &PathBuf) -> String {
    p.canonicalize()
        .unwrap_or_else(|x| {
            let err_str = format!("Error: {x} in report path!");
            log::error!("{err_str}");
            err_str.into()
        })
        .to_string_lossy()
        .to_string()
}
