# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| >= 0.5  | :white_check_mark: |
| < 0.5   | :x:                |

## Threat Model

Cobalt is an application usage tracking tool that monitors window focus and application activity on your system. Due to the nature of this functionality, we take security and privacy very seriously.

The Threat Model is somewhat smaller since it's a desktop application (both UI and engine run as Medium IL, unprivileged programs) that doesn't store any data on a server (all data is local).

The main threats that we consider are other unprivileged applications, through interaction of our app or modifications to the filesystem that Cobalt uses, cause system instability (e.g. system crashes), or privilege escalation.

Leakage of usage data (main.db) by another unprivileged application is not considered a vulnerability - similar to Chrome's History SQLite files, other unprivileged applications can read it.

Any vulnerability caused by a privileged application is not considered a vulnerability.

## Reporting a Vulnerability

If you have found a potential security threat, vulnerability or exploit in Cobalt
or one of its upstream dependencies, please DON'T create a pull-request, DON'T
file an issue on GitHub, or mention it publically on any platform.

Please submit your report via the GitHub Private Vulnerability Disclosure functionality.

Find out more about the reporting process [here](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing/privately-reporting-a-security-vulnerability#privately-reporting-a-security-vulnerability).

Our team will triage your report and keep you informed about the progress.
We may ask questions or request further guidance on reproduction of the vulnerability in the comments of the advisory, which will be publicized.

Additionally, we may ask you to independently verify our patch, which will be available in the private advisory branch. Please do not publish your vulnerability during the process or before coordinated public disclosure from our side. We try to adhere to common standards of publication within 90-Days of disclosure.

Depending on your decision to accept or deny credit for the vulnerability, you will be publicly attributed to the vulnerability.

At the current time we do not have the financial ability to reward bounties.