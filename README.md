# Bdrck

[![Build Status](https://travis-ci.org/CmdrMoozy/pwm.svg?branch=master)](https://travis-ci.org/CmdrMoozy/pwm) [![Coverage Status](https://coveralls.io/repos/github/CmdrMoozy/pwm/badge.svg?branch=master)](https://coveralls.io/github/CmdrMoozy/pwm?branch=master)

Bdrck is a crate containing generic common utilities. In particular, it has several top-level modules which provide various functionality:

| Module        | Description                        |
| ------------- | ---------------------------------- |
| configuration | Application configuration tooling. |
| flags         | Command-line flag parsing.         |
| fs            | Filesystem utilities.              |
| logging       | Log message formatting utilities.  |
| net           | Networking utilities / types.      |
| testing       | Unit testing utilities.            |

## Versioning

This project adheres to [Semantic Versioning](http://semver.org/). However, prior to 1.0.0 this project will adhere to the following rules:

- Any API/ABI breaking changes will result in a minor version bump.
- Any API extending features will result in a patch version bump.
- Any non-breaking bug fixes and performance improvements will result in a patch version bump.
