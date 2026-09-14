# Security Policy

## Supported Versions

Only the latest release receives fixes.

| Version | Supported |
| ------- | --------- |
| 0.1.x   | Yes       |

## Reporting a Vulnerability

If you discover a security vulnerability in iregexp-rs, please report it privately using GitHub's built-in vulnerability reporting:

1. Go to the [Security tab](https://github.com/strefethen/iregexp-rs/security)
2. Click **Report a vulnerability**
3. Provide a description of the issue and steps to reproduce

Please do not open a public issue for security vulnerabilities.

## Response

I'll acknowledge reports within a reasonable timeframe and provide an update when a fix is available. There is no formal SLA — this is a single-maintainer project.

## Scope

This policy covers the `iregexp-rs` library crate. Patterns are often attacker-controlled (for example, JSONPath `match()` and `search()` arguments), so the following are in scope:

- A pattern that panics, aborts, or overflows the stack during compilation or matching
- Compilation work that escapes the [documented resource limits](README.md#errors-and-limits)
- A pattern outside RFC 9485 that compiles successfully, or an invalid pattern reported as a non-match instead of an error

Resource exhaustion from input text that callers choose not to bound is outside that list. The README explains that callers own request-level input limits and concurrency budgets. Third-party dependencies are also not covered; please report those upstream.
