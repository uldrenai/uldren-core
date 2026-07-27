# Contributing to Uldren Core

Thank you for your interest in contributing. This document explains how to
propose changes and the terms under which contributions are accepted.

Uldren Core is licensed under the Business Source License 1.1 (see `LICENSE`).
Contributions are accepted only under the terms below.

## Individual Contributor License Agreement

External contributions require an approved and completed Individual Contributor
License Agreement (CLA). While CLA enrollment is unavailable, an external
contribution cannot be merged. The CLA requires each contributor to attest
that:

- You personally created the contribution and own the rights to it.
- You did not create it within employment, contracting, consulting, sponsored,
  client, or other organizational work.
- No employer, client, school, or organization can claim ownership of it.
- You can grant the copyright and patent rights the CLA requires.
- Any third-party material is identified and provided under a compatible
  license.

If ownership of a contribution is unclear, it is not merged.

## Organizational contributions

Organizationally owned contributions require an executed Corporate CLA and are
not accepted without one. This includes any contribution owned by an employer,
client, school, or other organization, and any contribution with unclear
ownership. An executed Corporate CLA does not cover a contribution submitted
without authority.

## How to propose a change

- Discuss significant changes in an issue before starting large work.
- Create a short-lived branch named `<type>/<short-kebab-description>`, where
  `<type>` is one of `feat`, `fix`, `docs`, `chore`, `refactor`, `test`,
  `build`, `ci`, `perf`, or `revert`.
- Keep the change scoped, and run `just ci` before opening a pull request.
- Write Conventional Commit subjects in the form `<type>: <imperative
  description>`, without ticket identifiers or trailers unless requested.
- Open a pull request against `main`; never push directly to `main`.

Every pull request requires passing checks and owner acceptance. The repository
owner may accept and merge a pull request without a second-person approval.
Changes merge into `main` with a normal merge commit. Released versions are
immutable `v<major>.<minor>.<patch>` tags.

## Reporting security issues

Do not report security vulnerabilities through public issues or pull requests.
Follow the private process in `SECURITY.md`.

## Code of conduct

Participation in this project is governed by `CODE_OF_CONDUCT.md`.

## Review and ownership verification

Reviews are thorough and may request changes for correctness, security,
contract compliance, and test coverage. Contribution authorship and ownership
are verified during review; a contribution that cannot clear the CLA and
ownership requirements above is not merged.
