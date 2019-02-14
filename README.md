# Bdrck

[![Build Status](https://travis-ci.org/CmdrMoozy/bdrck.svg?branch=master)](https://travis-ci.org/CmdrMoozy/bdrck)

Bdrck is a crate containing generic common utilities. In particular, it has several top-level modules which provide various functionality:

| Module        | Description                                   |
| ------------- | --------------------------------------------- |
| cli           | Utilities for CLI-based applications.         |
| configuration | Application configuration tooling.            |
| crypto        | Tools built upon high-level crypto libraries. |
| fs            | Filesystem utilities.                         |
| http          | HTTP client utilities based on reqwest.       |
| logging       | Log message formatting utilities.             |
| net           | Networking utilities / types.                 |
| testing       | Unit testing utilities.                       |

# Flaggy

Flaggy is a separate but related crate which provides command-line flag parsing.

## Versioning

This project adheres to [Semantic Versioning](http://semver.org/). However, prior to 1.0.0 this project will adhere to the following rules:

- Any API/ABI breaking changes will result in a minor version bump.
- Any API extending features will result in a patch version bump.
- Any non-breaking bug fixes and performance improvements will result in a patch version bump.
