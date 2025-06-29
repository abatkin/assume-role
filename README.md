assume-role
===========

A really simple program to assume an IAM Role (by calling [AssumeRole](https://docs.aws.amazon.com/STS/latest/APIReference/API_AssumeRole.html)) and save the resulting credentials back to your local [credential file](https://docs.aws.amazon.com/sdk-for-php/v3/developer-guide/guide_credentials_profiles.html) (`~/.aws/credentials` by default). It can also output credentials in the [Process Credential Provider](https://docs.aws.amazon.com/sdkref/latest/guide/feature-process-credentials.html) format using `--credential-process`. When using this mode, `--credential-process-cache <file>` can be specified to cache the credentials between runs.

Building
--------

```
% cargo build
```

Running
-------
```
% assume-role --help
assume-role 0.1.0
Adam Batkin <adam@batkin.net>

USAGE:
    assume-role --role <role> [--session-name <session-name>]

OPTIONS:
        --dest-file <dest-file>
            Credential file to save new credentials to [env: AWS_SHARED_CREDENTIALS_FILE=]

        --dest-profile <dest-profile>              Profile to save new credentials [default: default]
        --duration <duration>
            Lifetime in seconds for temporary credentials (AWS default is 3600 = 1 hour)

        --external-id <external-id>                External ID to pass to assume-role
    -f, --file <file>                              Credential file to load credentials from when calling assume-role
    -h, --help                                     Prints help information
        --mfa <mfa>                                MFA token code
        --mfa-serial-number <mfa-serial-number>    MFA device serial number
        --policy <policy>...                       ARN(s) of IAM managed policies to use as managed session policies
        --policy-json <policy-json>                Inline session policy JSON
    -p, --profile <profile>                        AWS Profile to use when calling assume-role
        --proxy <proxy>                            Proxy URL
        --credential-process                       Print credentials in Process Credential Provider format instead of saving to a file
        --credential-process-cache <file>          Cache file used with --credential-process
        --region <region>
            AWS Region for STS endpoint [env: AWS_DEFAULT_REGION=]  [default: us-east-1]

    -r, --role <role>                              ARN of role ot assume
    -s, --session-name <session-name>              Session name to pass to assume-role [default: assume-role-<timestamp>]
    -v, --verbose                                  Enable verbose output
    -V, --version                                  Prints version information
```
Realistically, you need to pass `--role` and you probably want `--dest-profile` (and possibly `--profile` or set `AWS_PROFILE`).

When `--verbose` is supplied, the tool enables additional tracing which also
prints the underlying HTTP requests made by the AWS SDK. This can help with
debugging connection or authentication issues.



