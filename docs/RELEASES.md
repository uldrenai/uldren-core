# Releases

## Versioning

Releases are immutable semantic-version tags named `v<major>.<minor>.<patch>`.
An existing release tag is never moved or deleted.

## Branch and release protection

`main` is protected and changes land through reviewed pull requests. Release
tags are immutable. The enforcing repository rules are configured on the source
host.

## Licensing over time

Uldren Core is distributed under the Business Source License 1.1. Each released
version converts to the Apache-2.0 change license four years after that version
is first published, as stated in `LICENSE`.

## Release requirements

Release automation follows the build and release specification. A promoted
release includes signed artifacts, checksums, provenance, dependency
identifiers, and verification evidence for every supported target.
