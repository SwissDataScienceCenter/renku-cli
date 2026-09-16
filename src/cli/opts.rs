use crate::{
    data::{
        project_id::{ProjectId, ProjectIdParseError},
        renku_url::RenkuUrl,
    },
    httpclient::{Client, Error as ClientError, proxy},
    project_config::{ProjectConfigError, RenkuProjectConfig},
};

use super::cmd::*;
use clap::{Parser, ValueEnum, ValueHint};
use clap_verbosity_flag::{Verbosity, WarnLevel};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::{path::PathBuf, str::FromStr};

/// Main options are available to all commands. They must appear
/// before a sub-command.
#[derive(Parser, Debug, Clone)]
#[command()]
pub struct CommonOpts {
    /// Be more verbose when logging. Verbosity increases with each
    /// occurence of that option.
    #[command(flatten)]
    pub verbosity: Verbosity<WarnLevel>,

    /// How to format the output. The default is human readable which
    /// may choose to not show every detail for better readability.
    /// The json output format can be used to always show all details
    /// in a structured form.
    #[arg(short, long, value_enum, default_value_t = Format::Default)]
    pub format: Format,

    /// The (base) URL to Renku. It can be given as environment
    /// variable RENKU_CLI_RENKU_URL.
    #[arg(long, value_hint = ValueHint::Url)]
    pub renku_url: Option<RenkuUrl>,

    /// Some commands may operate within a project. If this option is
    /// set, or the environment variable RENKU_CLI_PROJECT_CONTEXT is
    /// present commands can use it to confine there functionality to
    /// this project. The value may be the project id (ulid) or the
    /// path like <username>/<project-name>.
    #[arg(long, value_hint = ValueHint::Url)]
    pub project_context: Option<ProjectId>,

    /// Set a proxy to use for doing http requests. By default, the
    /// system proxy will be used. Can be either `none` or <url>. If
    /// `none`, the system proxy will be ignored; otherwise specify
    /// the proxy url, like `http://myproxy.com`.
    #[arg(long)]
    pub proxy: Option<ProxySetting>,

    /// The user to authenticate at the proxy.
    #[arg(long)]
    pub proxy_user: Option<String>,

    /// The password to authenticate at the proxy.
    #[arg(long)]
    pub proxy_password: Option<String>,
}

impl CommonOpts {
    const ACCESS_TOKEN_ENV: &str = "RENKU_CLI_ACCESS_TOKEN";

    pub fn create_client(&self, trusted_cert: Option<PathBuf>) -> Result<Client, ClientError> {
        let at = std::env::var(Self::ACCESS_TOKEN_ENV).ok();
        let base_url = self
            .get_renku_url()
            .map_err(|e| ClientError::UrlParse { source: e })?;
        Client::new(base_url, self.proxy_settings(), trusted_cert, false, at)
    }

    fn proxy_settings(&self) -> proxy::ProxySetting {
        let user = self.proxy_user.clone();
        let password = self.proxy_password.clone();
        let prx = self.proxy.clone();

        log::debug!("Using proxy: {:?} @ {:?}", user, prx);
        match prx {
            None => proxy::ProxySetting::System,
            Some(ProxySetting::None) => proxy::ProxySetting::None,
            Some(ProxySetting::Custom { url }) => proxy::ProxySetting::Custom {
                url: url.clone(),
                user,
                password,
            },
        }
    }

    fn get_renku_url(&self) -> Result<RenkuUrl, url::ParseError> {
        match &self.renku_url {
            Some(u) => {
                log::debug!("Use renku url from arguments: {}", u);
                Ok(u.clone())
            }
            None => match RenkuUrl::from_env() {
                Some(res) => {
                    if let Ok(u) = &res {
                        log::debug!("Use renku url from env RENKU_CLI_RENKU_URL: {}", u);
                    }
                    res
                }
                None => {
                    log::debug!("Use renku url: https://renkulab.io");
                    Ok(RenkuUrl::renkulab_io())
                }
            },
        }
    }

    /// Return the project context.
    ///
    /// The project context is a project identifier that commands can
    /// use to scope their actions to that project. It is read via the
    /// following strategy (first wins):
    ///
    /// - use the option if specified
    /// - read $CWD/.renku/config.toml
    /// - use environment variable RENKU_CLI_PROJECT_CONTEXT
    pub fn get_project_context(&self) -> Result<Option<ProjectId>, ProjectContextError> {
        fn get_from_env() -> Result<Option<ProjectId>, ProjectIdParseError> {
            match std::env::var("RENKU_CLI_PROJECT_CONTEXT").ok() {
                Some(id) => ProjectId::parse(&id).map(Some),
                None => Ok(None),
            }
        }
        if self.project_context.is_some() {
            return Ok(self.project_context.clone());
        }
        match get_from_env() {
            Ok(Some(id)) => return Ok(Some(id)),
            Err(err) => {
                log::warn!("Error getting project id from env: {}", err)
            }
            _ => {}
        }

        match RenkuProjectConfig::read_current_dir() {
            Ok(Some(cfg)) => return Ok(Some(ProjectId::Id(cfg.project.id))),
            Err(err) => {
                log::warn!("Error getting project id from env: {}", err)
            }
            _ => {}
        }

        RenkuProjectConfig::read_global_config()
            .map(|ok| ok.map(|cfg| ProjectId::Id(cfg.project.id)))
            .context(ConfigSnafu)
    }
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    #[command()]
    Version(version::Input),
    #[command()]
    Update(update::Input),

    #[command(alias = "p")]
    Project(project::Input),

    /// Clone a project. (Shortcut for 'project clone')
    #[command()]
    Clone(project::clone::Input),

    #[command()]
    Login(login::Input),

    #[cfg(feature = "user-doc")]
    UserDoc(userdoc::Input),

    #[command(alias = "d")]
    Dataset(dataset::Input),

    #[command(alias = "j")]
    Job(job::Input),
    #[command(alias = "l")]
    Launcher(launcher::Input),

    #[command()]
    Logout(logout::Input),
}

/// This is the command line interface to the Renku platform. Main
/// options are available to all sub-commands and must appear before
/// them. Each sub command has its own set of flags/options and
/// arguments.
///
/// Repository: <https://github.com/SwissDataScienceCenter/renku-cli>
/// Issue tracker: <https://github.com/SwissDataScienceCenter/renku-cli/issues>
#[derive(Parser, Debug)]
#[command(name = "rnk", version)]
pub struct MainOpts {
    #[clap(flatten)]
    pub common_opts: CommonOpts,

    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

/// The format for presenting the results.
#[derive(ValueEnum, Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum Format {
    Json,
    Default,
}

#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub enum ProxySetting {
    /// Don't use any proxy; this will also discard the system proxy.
    None,

    /// Use a custom defined proxy.
    Custom { url: String },
}

impl FromStr for ProxySetting {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("none") {
            Ok(ProxySetting::None)
        } else {
            Ok(ProxySetting::Custom { url: s.to_string() })
        }
    }
}

#[derive(Debug, Snafu)]
pub enum ProjectContextError {
    Parse { source: ProjectIdParseError },
    Config { source: ProjectConfigError },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn create_local_config(project_id: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let tmp = std::env::temp_dir();
        let pid = std::process::id();
        let id = rand::random::<u64>();
        let test_dir = tmp.join(format!("renku_test_opts_p{}_{id}", pid));
        std::fs::create_dir_all(test_dir.join(".renku")).unwrap();
        let config_path = test_dir.join(".renku").join("config.toml");
        let config = format!(
            "\
version = 1
renku_url = \"https://renkulab.io\"

[project]
id = \"{}\"
namespace = \"test-ns\"
slug = \"test-project\"
",
            project_id
        );
        std::fs::File::create(&config_path)
            .unwrap()
            .write_all(config.as_bytes())
            .unwrap();
        (test_dir, config_path)
    }

    fn cleanup_local_config(test_dir: &std::path::Path) {
        let _ = std::fs::remove_dir_all(test_dir);
    }

    /// Helper to create a global config file in the expected location.
    /// Returns the path to the created file and the previous content (if any).
    fn create_global_config(project_id: &str) -> (std::path::PathBuf, Option<String>) {
        use directories::ProjectDirs;
        let db_dir = ProjectDirs::from("io.renku", "sdsc", "renku-cli")
            .expect("global config folder not found")
            .data_dir()
            .to_path_buf();
        let target = db_dir.join("active_project.toml");
        let prev_content = std::fs::read_to_string(&target).ok();
        let config = format!(
            "\
version = 1
renku_url = \"https://renkulab.io\"

[project]
id = \"{}\"
namespace = \"test-ns\"
slug = \"test-project\"
",
            project_id
        );
        eprintln!("path: {}", db_dir.display());
        std::fs::create_dir_all(&db_dir).unwrap();
        std::fs::write(&target, config).unwrap();
        (target, prev_content)
    }

    fn cleanup_global_config(path: &std::path::Path, prev_content: Option<String>) {
        match prev_content {
            Some(content) => {
                let _ = std::fs::write(path, content);
            }
            None => {
                let _ = std::fs::remove_file(path);
            }
        }
    }

    // ---------------------------------------------------------------------------
    // Precedence: CLI arg > env var > local config > global config
    // ---------------------------------------------------------------------------

    #[test]
    fn cli_arg_is_used_when_present() {
        let opts = CommonOpts {
            verbosity: clap_verbosity_flag::Verbosity::new(0, 0),
            format: Format::Default,
            renku_url: None,
            project_context: Some(ProjectId::parse("cli-project-id").unwrap()),
            proxy: None,
            proxy_user: None,
            proxy_password: None,
        };
        let result = opts.get_project_context().unwrap();
        assert_eq!(result, Some(ProjectId::parse("cli-project-id").unwrap()));
    }

    #[test]
    #[serial_test::serial]
    fn cli_arg_takes_precedence_over_env_var() {
        unsafe {
            std::env::set_var("RENKU_CLI_PROJECT_CONTEXT", "env-project-id");
        }
        let opts = CommonOpts {
            verbosity: clap_verbosity_flag::Verbosity::new(0, 0),
            format: Format::Default,
            renku_url: None,
            project_context: Some(ProjectId::parse("cli-project-id").unwrap()),
            proxy: None,
            proxy_user: None,
            proxy_password: None,
        };
        let result = opts.get_project_context().unwrap();
        assert_eq!(result, Some(ProjectId::parse("cli-project-id").unwrap()));
    }

    #[test]
    #[serial_test::serial]
    fn env_var_is_used_when_no_cli_arg() {
        unsafe {
            std::env::set_var("RENKU_CLI_PROJECT_CONTEXT", "env-project-id");
        }
        let opts = CommonOpts {
            verbosity: clap_verbosity_flag::Verbosity::new(0, 0),
            format: Format::Default,
            renku_url: None,
            project_context: None,
            proxy: None,
            proxy_user: None,
            proxy_password: None,
        };
        let result = opts.get_project_context().unwrap();
        assert_eq!(result, Some(ProjectId::parse("env-project-id").unwrap()));
    }

    #[test]
    #[serial_test::serial]
    fn env_var_takes_precedence_over_local_config() {
        let saved = std::env::var("RENKU_CLI_PROJECT_CONTEXT").ok();
        let (test_dir, _config_path) = create_local_config("local-project-id");

        let orig_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&test_dir).unwrap();

        unsafe {
            std::env::set_var("RENKU_CLI_PROJECT_CONTEXT", "env-project-id");
        }
        let opts = CommonOpts {
            verbosity: clap_verbosity_flag::Verbosity::new(0, 0),
            format: Format::Default,
            renku_url: None,
            project_context: None,
            proxy: None,
            proxy_user: None,
            proxy_password: None,
        };
        let result = opts.get_project_context().unwrap();
        assert_eq!(result, Some(ProjectId::parse("env-project-id").unwrap()));

        std::env::set_current_dir(orig_dir).unwrap();
        cleanup_local_config(&test_dir);

        match saved {
            Some(v) => unsafe { std::env::set_var("RENKU_CLI_PROJECT_CONTEXT", v) },
            None => unsafe { std::env::remove_var("RENKU_CLI_PROJECT_CONTEXT") },
        }
    }

    #[test]
    #[serial_test::serial]
    fn local_config_is_used_when_no_cli_or_env() {
        let saved = std::env::var("RENKU_CLI_PROJECT_CONTEXT").ok();
        unsafe {
            std::env::remove_var("RENKU_CLI_PROJECT_CONTEXT");
        }
        let (test_dir, _config_path) = create_local_config("local-project-id");

        let orig_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&test_dir).unwrap();

        let opts = CommonOpts {
            verbosity: clap_verbosity_flag::Verbosity::new(0, 0),
            format: Format::Default,
            renku_url: None,
            project_context: None,
            proxy: None,
            proxy_user: None,
            proxy_password: None,
        };
        let result = opts.get_project_context().unwrap();
        assert_eq!(result, Some(ProjectId::parse("local-project-id").unwrap()));

        std::env::set_current_dir(orig_dir).unwrap();
        cleanup_local_config(&test_dir);

        match saved {
            Some(v) => unsafe { std::env::set_var("RENKU_CLI_PROJECT_CONTEXT", v) },
            None => unsafe { std::env::remove_var("RENKU_CLI_PROJECT_CONTEXT") },
        }
    }

    #[test]
    #[serial_test::serial]
    fn local_config_overrides_global_config() {
        let saved = std::env::var("RENKU_CLI_PROJECT_CONTEXT").ok();
        unsafe {
            std::env::remove_var("RENKU_CLI_PROJECT_CONTEXT");
        }
        let (global_config, prev_content) = create_global_config("global-project-id");
        let (test_dir, _config_path) = create_local_config("local-project-id");

        let orig_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&test_dir).unwrap();

        let opts = CommonOpts {
            verbosity: clap_verbosity_flag::Verbosity::new(0, 0),
            format: Format::Default,
            renku_url: None,
            project_context: None,
            proxy: None,
            proxy_user: None,
            proxy_password: None,
        };
        let result = opts.get_project_context().unwrap();
        assert_eq!(result, Some(ProjectId::parse("local-project-id").unwrap()));

        std::env::set_current_dir(orig_dir).unwrap();
        cleanup_local_config(&test_dir);
        cleanup_global_config(&global_config, prev_content);

        match saved {
            Some(v) => unsafe { std::env::set_var("RENKU_CLI_PROJECT_CONTEXT", v) },
            None => unsafe { std::env::remove_var("RENKU_CLI_PROJECT_CONTEXT") },
        }
    }

    #[test]
    #[serial_test::serial]
    fn global_config_is_used_when_nothing_else() {
        let saved = std::env::var("RENKU_CLI_PROJECT_CONTEXT").ok();
        unsafe {
            std::env::remove_var("RENKU_CLI_PROJECT_CONTEXT");
        }
        let (global_config, prev_content) = create_global_config("global-project-id");

        // Use a directory with no local config
        let no_config_dir = std::env::temp_dir().join(format!(
            "renku_test_opts_no_config_p{}_{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        let _ = std::fs::remove_dir_all(&no_config_dir);
        std::fs::create_dir_all(&no_config_dir).unwrap();

        let orig_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&no_config_dir).unwrap();

        let opts = CommonOpts {
            verbosity: clap_verbosity_flag::Verbosity::new(0, 0),
            format: Format::Default,
            renku_url: None,
            project_context: None,
            proxy: None,
            proxy_user: None,
            proxy_password: None,
        };
        let result = opts.get_project_context().unwrap();
        assert_eq!(result, Some(ProjectId::parse("global-project-id").unwrap()));

        std::env::set_current_dir(orig_dir).unwrap();
        let _ = std::fs::remove_dir_all(&no_config_dir);
        cleanup_global_config(&global_config, prev_content);

        match saved {
            Some(v) => unsafe { std::env::set_var("RENKU_CLI_PROJECT_CONTEXT", v) },
            None => unsafe { std::env::remove_var("RENKU_CLI_PROJECT_CONTEXT") },
        }
    }

    // ---------------------------------------------------------------------------
    // No context
    // ---------------------------------------------------------------------------

    #[test]
    #[serial_test::serial]
    fn no_context_when_nothing_set() {
        // Ensure global config is removed so this test truly has no context.
        use directories::ProjectDirs;
        let pd = ProjectDirs::from("io.renku", "sdsc", "renku-cli")
            .expect("global config folder not found");
        let global_file = pd.data_dir().join("active_project.toml");
        let _ = std::fs::remove_file(&global_file);

        let saved = std::env::var("RENKU_CLI_PROJECT_CONTEXT").ok();
        unsafe {
            std::env::remove_var("RENKU_CLI_PROJECT_CONTEXT");
        }
        let no_config_dir = std::env::temp_dir().join(format!(
            "renku_test_opts_no_config_p{}_{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        let _ = std::fs::remove_dir_all(&no_config_dir);
        std::fs::create_dir_all(&no_config_dir).unwrap();

        let orig_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&no_config_dir).unwrap();

        let opts = CommonOpts {
            verbosity: clap_verbosity_flag::Verbosity::new(0, 0),
            format: Format::Default,
            renku_url: None,
            project_context: None,
            proxy: None,
            proxy_user: None,
            proxy_password: None,
        };
        let result = opts.get_project_context().unwrap();
        assert_eq!(result, None);

        std::env::set_current_dir(orig_dir).unwrap();
        let _ = std::fs::remove_dir_all(&no_config_dir);

        match saved {
            Some(v) => unsafe { std::env::set_var("RENKU_CLI_PROJECT_CONTEXT", v) },
            None => unsafe { std::env::remove_var("RENKU_CLI_PROJECT_CONTEXT") },
        }
    }
}
