# Security Policy

## Supported Versions

Security fixes target the default branch until the project publishes stable
release lines.

## Reporting a Vulnerability

Do not open public issues for suspected vulnerabilities.

Report security issues through GitHub private vulnerability reporting when it is
available for this repository. If that is unavailable, contact the repository
owner directly through the GitHub profile listed on the project page.

Please include:

- A concise description of the issue.
- Steps to reproduce or a minimal proof of concept.
- Impact, affected commands, and expected vs. actual behavior.
- Relevant environment details.

## Security Posture

`agent-calc` is designed as an AI-callable CLI with typed JSON contracts. The
security model favors deterministic behavior, bounded inputs, stable error
codes, exact arithmetic where possible, and explicit validation limits for
precision, expression depth, node count, integer digit count, symbol length,
exponent size, and binding count.

Repository automation should keep the following checks healthy:

- Formatting, type-checking, tests, and package verification.
- Property-based tests for protocol and domain invariants.
- Mutation testing before readiness claims.
- Dependency monitoring through Dependabot.
- Static analysis through GitHub CodeQL.
