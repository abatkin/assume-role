use anyhow::{Context, Result};
use rusoto_core::Region;
use std::path::PathBuf;
use std::str::FromStr;
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

    /// Credential file to load credentials from when calling assume-role
    #[structopt(name = "file", long, short = "f")]
    pub credential_file: Option<String>,

    /// AWS Region for STS endpoint
    #[structopt(
        name = "region",
        long,
        env = "AWS_DEFAULT_REGION",
        default_value = "us-east-1"
    )]
    pub region: Region,

    /// Lifetime in seconds for temporary credentials (AWS default is 3600 = 1 hour)
    #[structopt(name = "duration", long)]
    pub duration: Option<u16>,

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
    pub session_name: String,

    /// MFA device serial number
    #[structopt(name = "mfa-serial-number", long)]
    pub mfa_serial_number: Option<String>,

    /// MFA token code
    #[structopt(name = "mfa", long)]
    pub mfa_token: Option<String>,

    /// Credential file to save new credentials to
    #[structopt(name = "dest-file", long, env = "AWS_SHARED_CREDENTIALS_FILE")]
    pub dest_credential_file: Option<String>,

    /// Profile to save new credentials
    #[structopt(name = "dest-profile", long, default_value = "default")]
    pub dest_profile: String,

    /// Proxy URL
    pub proxy: Option<String>,
}

impl Cmdline {
    pub fn parse() -> Cmdline {
        Cmdline::from_args()
    }

    pub fn determine_credential_file(&self) -> Result<PathBuf> {
        Ok(match &self.dest_credential_file {
            Some(filename) => PathBuf::from_str(filename)
                .with_context(|| format!("bad credential file path: {}", filename))?,
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
