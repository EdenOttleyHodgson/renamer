use std::{error::Error, path::PathBuf};

use clap::{Args, Parser, ValueEnum};
use renamer_lib::{ActionGroup, RenamePattern, patterns::ActionOptions};

#[derive(Parser, Debug)]
struct RenamerArgs {
    #[command(flatten)]
    pattern_preset: PatternPresetArgs,
    #[arg(short, long, num_args = 1.., value_delimiter = ' ')]
    files: Vec<PathBuf>,
    #[arg(long = "preserve-extension")]
    dont_preserve_extension: bool,
    #[arg(short, long)]
    overwrite: bool,
}
impl RenamerArgs {
    fn deconstruct(self) -> (PatternOrPreset, Vec<PathBuf>, ActionOptions) {
        let pat_or_preset = if let Some(preset) = self.pattern_preset.preset {
            PatternOrPreset::Preset(preset)
        } else if let Some(pattern) = self.pattern_preset.pattern {
            PatternOrPreset::Pattern(pattern)
        } else {
            unreachable!()
        };
        (
            pat_or_preset,
            self.files,
            ActionOptions::new(!self.dont_preserve_extension, self.overwrite),
        )
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
impl Preset {
    fn into_pattern(self, options: ActionOptions) -> RenamePattern {
        match self {
            Preset::Randomize => RenamePattern::randomize(options),
        }
    }
}
enum PatternOrPreset {
    Pattern(String),
    Preset(Preset),
}
impl PatternOrPreset {
    fn into_pattern(self, options: ActionOptions) -> Result<RenamePattern, Box<dyn Error>> {
        match self {
            PatternOrPreset::Pattern(pat) => {
                RenamePattern::parse(pat.as_str(), options).map_err(Into::into)
            }
            PatternOrPreset::Preset(preset) => Ok(preset.into_pattern(options)),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let (pat_or_preset, files, options) = RenamerArgs::parse().deconstruct();
    let pattern: RenamePattern = pat_or_preset.into_pattern(options)?;
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
