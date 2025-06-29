use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{Context, Result};
use std::time::{SystemTime, UNIX_EPOCH};
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
name = "assume-role",
author = "Adam Batkin <adam@batkin.net>",
settings = &[AppSettings::UnifiedHelpMessage])
]
pub struct Cmdline {
    /// AWS Profile to use when calling assume-role
    #[structopt(name = "profile", short = "p", long)]
    pub profile: Option<String>,

    /// AWS Region for STS endpoint
    #[structopt(name = "region", long)]
    pub region: Option<String>,

    /// Lifetime in seconds for temporary credentials (AWS default is 3600 = 1 hour)
    #[structopt(name = "duration", long)]
    pub duration: Option<i32>,

    /// External ID to pass to assume-role
    #[structopt(name = "external-id", long)]
    pub external_id: Option<String>,

    /// Inline session policy JSON
    #[structopt(name = "policy-json", long)]
    pub policy_json: Option<String>,

    /// ARN(s) of IAM managed policies to use as managed session policies
    #[structopt(name = "policy", long)]
    pub policies: Option<Vec<String>>,

    /// ARN of role ot assume
    #[structopt(name = "role", long, short = "r")]
    pub role_arn: String,

    /// Session name to pass to assume-role
    #[structopt(name = "session-name", long, short = "s")]
    pub session_name: Option<String>,

    /// MFA device serial number
    #[structopt(name = "mfa-serial-number", long)]
    pub mfa_serial_number: Option<String>,

    /// MFA token code
    #[structopt(name = "mfa", long)]
    pub mfa_token: Option<String>,

    /// Credential file to save new credentials to
    #[structopt(name = "credentials-file", long, env = "AWS_SHARED_CREDENTIALS_FILE")]
    pub credential_file: Option<String>,

    /// Profile to save new credentials
    #[structopt(name = "dest-profile", long, default_value = "default")]
    pub dest_profile: String,

    /// Proxy URL
    #[structopt(name = "proxy", long)]
    pub proxy: Option<String>,

    /// Print credentials in Process Credential Provider format instead of saving to a file
    #[structopt(name = "credential-process", long)]
    pub credential_process: bool,

    /// Cache file when using --credential-process
    #[structopt(name = "credential-process-cache", long)]
    pub credential_process_cache: Option<String>,

    /// Enable verbose output
    #[structopt(short, long)]
    pub verbose: bool,
}

impl Cmdline {
    pub fn parse() -> Cmdline {
        Cmdline::from_args()
    }

    pub fn session_name(&self) -> String {
        match &self.session_name {
            Some(name) => name.clone(),
            None => {
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                format!("assume-role-{}", ts)
            }
        }
    }

    pub fn determine_credential_file(&self) -> Result<PathBuf> {
        Ok(match &self.credential_file {
            Some(filename) => PathBuf::from_str(filename)
                .with_context(|| format!("bad credential file path: {filename}"))?,
            None => {
                let mut path = PathBuf::new();
                path.push(
                    directories::UserDirs::new()
                        .with_context(|| "unable to determine home directory")?
                        .home_dir(),
                );
                path.push(".aws");
                path.push("credentials");
                path
            }
        })
    }
}
