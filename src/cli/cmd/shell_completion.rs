use clap::{Command, Parser, ValueEnum};
use clap_complete::{generate, Generator, Shell};

/// Generates completions for some shells.
///
#[derive(Parser, std::fmt::Debug)]
pub struct Input {
    /// For which shell to generate completions.
    #[arg(long, value_enum)]
    pub shell: GeneratorChoice,

    /// The binary name.
    #[arg(long, default_value = "rnk")]
    pub binary: String,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum GeneratorChoice {
    Bash,
    Elvish,
    Fish,
    #[value(name = "powershell")]
    PowerShell,
    Zsh,
}

impl Input {
    pub async fn print_completions(&self, app: &mut Command) {
        let binary = &self.binary;
        match &self.shell {
            GeneratorChoice::Bash => generate_completions(Shell::Bash, binary, app),
            GeneratorChoice::Elvish => generate_completions(Shell::Elvish, binary, app),
            GeneratorChoice::Fish => generate_completions(Shell::Fish, binary, app),
            GeneratorChoice::PowerShell => generate_completions(Shell::PowerShell, binary, app),
            GeneratorChoice::Zsh => generate_completions(Shell::Zsh, binary, app),
        }
    }
}

fn generate_completions<G: Generator>(gen: G, binary: &str, app: &mut Command) {
    generate(gen, app, binary, &mut std::io::stdout());
}
