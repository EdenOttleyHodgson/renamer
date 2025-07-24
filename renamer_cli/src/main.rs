use std::{error::Error, path::PathBuf};

use clap::{Args, Parser, ValueEnum};
use renamer_lib::{ActionGroup, RenamePattern};

#[derive(Parser, Debug)]
struct RenamerArgs {
    #[command(flatten)]
    pattern_preset: PatternPresetArgs,
    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    files: Vec<PathBuf>,
}
impl RenamerArgs {
    fn deconstruct(self) -> (PatternOrPreset, Vec<PathBuf>) {
        let pat_or_preset = if let Some(preset) = self.pattern_preset.preset {
            PatternOrPreset::Preset(preset)
        } else if let Some(pattern) = self.pattern_preset.pattern {
            PatternOrPreset::Pattern(pattern)
        } else {
            unreachable!()
        };
        (pat_or_preset, self.files)
    }
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct PatternPresetArgs {
    #[arg(short, long)]
    pattern: Option<String>,
    #[arg(long)]
    preset: Option<Preset>,
}

#[derive(ValueEnum, Clone, Copy, Debug)]
enum Preset {
    Randomize,
}
impl Into<RenamePattern> for Preset {
    fn into(self) -> RenamePattern {
        match self {
            Preset::Randomize => RenamePattern::randomize(),
        }
    }
}
enum PatternOrPreset {
    Pattern(String),
    Preset(Preset),
}
impl TryInto<RenamePattern> for PatternOrPreset {
    type Error = Box<dyn Error>;
    fn try_into(self) -> Result<RenamePattern, Self::Error> {
        match self {
            PatternOrPreset::Pattern(pat) => {
                RenamePattern::try_from(pat.as_str()).map_err(Into::into)
            }
            PatternOrPreset::Preset(preset) => Ok(preset.into()),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (pat_or_preset, files) = RenamerArgs::parse().deconstruct();
    let pattern: RenamePattern = pat_or_preset.try_into()?;
    let mut action_group = ActionGroup::new(0);
    for file in files.into_iter() {
        action_group.add_file(file.canonicalize()?);
    }
    action_group.add_pattern(pattern);
    let reports = action_group.execute();
    for report in reports {
        match report {
            Ok(rep) => println!("Success!: {rep:?}"),
            Err(err) => println!("Failure!: {err}"),
        }
    }

    Ok(())
}
