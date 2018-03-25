# i2c

[![travis-badge][]][travis] [![release-badge][]][cargo] [![docs-badge][]][docs] [![license-badge][]][license]

`i2c` is a crate providing traits for working with an I2C bus.

## Implementations

The generic traits in `i2c` must be implemented by an I2C master/driver in order
to be used:

- [i2c-linux](https://crates.io/crates/i2c-linux) with the `i2c` feature.
- [i2c-i2cdev](https://crates.io/crates/i2c-i2cdev) for the [i2cdev](https://crates.io/crates/i2cdev) crate.
- [nvapi](https://crates.io/crates/nvapi) with the `i2c` feature.
- Possibly other [downstream crates](https://crates.io/crates/i2c/reverse_dependencies)

## [Documentation][docs]

See the [documentation][docs] for up to date information.

[travis-badge]: https://img.shields.io/travis/arcnmx/i2c-rs/master.svg?style=flat-square
[travis]: https://travis-ci.org/arcnmx/i2c-rs
[release-badge]: https://img.shields.io/crates/v/i2c.svg?style=flat-square
[cargo]: https://crates.io/crates/i2c
[docs-badge]: https://img.shields.io/badge/API-docs-blue.svg?style=flat-square
[docs]: http://arcnmx.github.io/i2c-rs/i2c/
[license-badge]: https://img.shields.io/badge/license-MIT-ff69b4.svg?style=flat-square
[license]: https://github.com/arcnmx/i2c-rs/blob/master/COPYING
