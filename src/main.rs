use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::{Context, Result};
use ini::Ini;
use rusoto_core::credential::{ChainProvider, ProfileProvider};
use rusoto_core::{Client, HttpClient, Region};
use rusoto_sts::{AssumeRoleRequest, Credentials, PolicyDescriptorType, Sts, StsClient};
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "assume-role",
    author = "Adam Batkin <adam@batkin.net>",
    settings = &[AppSettings::UnifiedHelpMessage])
]
struct Cmdline {
    /// AWS Profile to use when calling assume-role
    #[structopt(name = "profile", short = "p", long)]
    profile: Option<String>,

    /// Credential file to load credentials from when calling assume-role
    #[structopt(name = "file", long, short = "f")]
    credential_file: Option<String>,

    /// AWS Region for STS endpoint
    #[structopt(
        name = "region",
        long,
        env = "AWS_DEFAULT_REGION",
        default_value = "us-east-1"
    )]
    region: Region,

    /// Lifetime in seconds for temporary credentials (AWS default is 3600 = 1 hour)
    #[structopt(name = "duration", long)]
    duration: Option<u16>,

    /// External ID to pass to assume-role
    #[structopt(name = "external-id", long)]
    external_id: Option<String>,

    /// Inline session policy JSON
    #[structopt(name = "policy-json", long)]
    policy_json: Option<String>,

    /// ARN(s) of IAM managed policies to use as managed session policies
    #[structopt(name = "policy", long)]
    policies: Option<Vec<String>>,

    /// ARN of role ot assume
    #[structopt(name = "role", long, short = "r")]
    role_arn: String,

    /// Session name to pass to assume-role
    #[structopt(name = "session-name", long, short = "s")]
    session_name: String,

    /// MFA device serial number
    #[structopt(name = "mfa-serial-number", long)]
    mfa_serial_number: Option<String>,

    /// MFA token code
    #[structopt(name = "mfa", long)]
    mfa_token: Option<String>,

    /// Credential file to save new credentials to
    #[structopt(name = "dest-file", long, env = "AWS_SHARED_CREDENTIALS_FILE")]
    dest_credential_file: Option<String>,

    /// Profile to save new credentials
    #[structopt(name = "dest-profile", long, default_value = "default")]
    dest_profile: String,
}

impl Cmdline {
    fn parse() -> Cmdline {
        Cmdline::from_args()
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cmdline = Cmdline::parse();

    let sts_client = build_sts_client(&cmdline)?;
    let assume_role_request = build_assume_role_request(&cmdline);
    let result = sts_client.assume_role(assume_role_request).await?;

    let credentials = result
        .credentials
        .with_context(|| "no credentials in response")?;

    let credential_filename = determine_credential_file(&cmdline.dest_credential_file)?;

    let mut credential_file = CredentialFile::load(&credential_filename)?;
    credential_file.set_credentials(&cmdline.dest_profile, &credentials);
    credential_file.save(&credential_filename)?;

    Ok(())
}

fn build_sts_client(cmdline: &Cmdline) -> Result<StsClient> {
    let credential_provider =
        build_credential_provider(&cmdline.profile, &cmdline.credential_file)?;

    let http_client = HttpClient::new().context("failed to create request dispatcher")?;
    let client = Client::new_with(credential_provider, http_client);
    let sts_client = StsClient::new_with_client(client, cmdline.region.clone());

    Ok(sts_client)
}

fn build_credential_provider(
    profile: &Option<String>,
    credential_file: &Option<String>,
) -> Result<ChainProvider> {
    let mut profile_provider = ProfileProvider::new()?;
    if let Some(profile_name) = profile {
        profile_provider.set_profile(profile_name);
    }
    if let Some(filename) = credential_file {
        profile_provider.set_file_path(filename);
    }
    Ok(ChainProvider::with_profile_provider(profile_provider))
}

fn build_assume_role_request(cmdline: &Cmdline) -> AssumeRoleRequest {
    let policies = match &cmdline.policies {
        Some(policies) => Some(
            policies
                .iter()
                .map(|arn| PolicyDescriptorType {
                    arn: Some(arn.clone()),
                })
                .collect(),
        ),
        None => None,
    };

    AssumeRoleRequest {
        duration_seconds: cmdline.duration.map(|v| v as i64),
        external_id: cmdline.external_id.clone(),
        policy: cmdline.policy_json.clone(),
        policy_arns: policies,
        role_arn: cmdline.role_arn.clone(),
        role_session_name: cmdline.session_name.clone(),
        serial_number: cmdline.mfa_serial_number.clone(),
        token_code: cmdline.mfa_token.clone(),
        ..AssumeRoleRequest::default()
    }
}

fn determine_credential_file(dest_credential_file: &Option<String>) -> Result<PathBuf> {
    Ok(match dest_credential_file {
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

struct CredentialFile {
    ini: Ini,
}

impl CredentialFile {
    fn load<P: AsRef<Path>>(filename: P) -> Result<CredentialFile> {
        let path = filename.as_ref();
        let ini = Ini::load_from_file(path)
            .with_context(|| format!("unable to load credential file {}", path.display()))?;
        Ok(CredentialFile { ini })
    }

    fn set_credentials(&mut self, profile: &str, credentials: &Credentials) {
        self.ini.delete(Some(profile));
        self.ini
            .with_section(Some(profile))
            .set("aws_access_key_id", &credentials.access_key_id)
            .set("aws_secret_access_key", &credentials.secret_access_key)
            .set("aws_session_token", &credentials.session_token);
    }

    fn save<P: AsRef<Path>>(&self, filename: P) -> Result<()> {
        let path = filename.as_ref();
        self.ini
            .write_to_file(path)
            .with_context(|| format!("unable to save credential file {}", path.display()))
    }
}
