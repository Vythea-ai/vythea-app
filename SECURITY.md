# Security

Vythea runs commands, holds keys and moves money from its wallet, so we take reports seriously.

## Reporting a problem

Email **support@vythea.com** with "Security" in the subject. Please don't open a public issue for a security problem.

Say what you found, how to reproduce it, and what it lets someone do. We answer within three working days, keep you posted while we fix it, and credit you in the release notes if you like.

## What counts

- The desktop app, the terminal and their updater
- The wallet, its policy and x402 payments
- Vythea Cloud and the account API at api.vythea.com
- The install scripts

Releases are signed: an update installs only when `latest.json` verifies against a key built into the app. A way around that check is in scope.

## Supported versions

Only the latest release gets fixes. Vythea is in alpha.
