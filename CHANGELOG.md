# Changelog

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Flashing support for the STM32L4 series.

### Changed

- Improved error messages for ARM register operations.

### Fixed

- Fixed a bug where reading a chip definition from a YAML file would always fail because parsing a `ChipFamily` from YAML was broken.

## [0.5.1]

### Fixed

- Fix a bug where M3 targets would not be able to load the core.

## [0.5.0]

### Added

- Flashing support for the STM32G0 series.
- Flashing support for the STM32F0 series.
- Flashing support for the STM32WB55 series.
- Support for RISCV debugging using a Jlink debug probe.
- Support for SWD debugging using a Jlink debug probe.

### Changed

- The entire API was overhauled. The Probe, Session and Core structs have different interaction and APIs now.
  Please have a look at the docs and examples to get an idea of the new interface.
  The new API supports multiple architectures and makes the initialization process until the point where you can talk to a core easier.
  The core methods don't need a passed probe anymore. Instead it stores an Rc to the Session object internally. The Probe object is taken by the Session which then can attach to multiple cores.
  The modules have been cleaned up. Some heavily nested hierarchy has been flattened.
- More consistent and clean naming and reporting of errors in the stlink and daplink modules. Also the errorhandling for the probe has been improved.

### Fixed

- Various fixes

### Known issues

- Some chips do not reset automatically after flashing
- The STM32L0 cores have issues with flashing.

## [0.4.0]

### Added

- A basic GDB server was added \o/ You can either use the provided `gdb-server` binary or use `cargo flash --gdb` to first flash the target and then open a GDB session. There is many more new options which you can list with `cargo flash --help`.
- Support for multiple breakpoints was added. Breakpoints can now conveniently be set and unset. probe-rs checks for you that there is a free breakpoint and complains if not.
- A flag to disable progressbars was added. Error reporting was broken because of progressbar overdraw. Now one can disable progress bars to see errors. In the long run this has to be fixed.
- Added an improved way to create a `Probe`.
- Added an older USB PID to have probe-rs detect older STLinks with updated Firmware.
- Added support for flashing with different sector properties. This fixed broken flashing on the STM M4s.

### Changed

- Code generation for built in targets was split off into a separate crate so probe-rs can be built without built in targets if one doesn't want them.

### Fixed
- Fixed setting and clearing breakpoints on M4 cores.

## [0.3.0]

Improved flashing for `cargo-flash` considering speed and useability.

### Added

- Increased the raw flashing speed by factor 10 and the actual flashing speed for small programs by factor 5. This is done using batched CMSIS-DAP transfers.
- Added CMSIS-Pack powered flashing. This feature essentially enables to flash any ARM core which can also be flashed by ARM Keil.
- Added progress bars for flash progress indication.
- Added `nrf-recover` feature that unlocks nRF52 chips through Nordic's custom `AP`

### Changed

- Improved target autodetection with better error distinction.
- Improved messaging overall.

### Fixed

- Various bugfixes
- Binaries bigger than a sector can now be flashed.

## [0.2.0]

Initial release on crates.io
- Added parsing of yaml (or anything else) config files for flash algorithm definitions, such that arbitrary chips can be added.
- Modularized code to allow other cores than M0 and be able to dynamically load chip definitions.
- Added target autodetection.
- Added M4 targets.
- Working basic flash downloader with nRF51.
- Introduce cargo-flash which can automatically build & flash the target elf file.

[Unreleased]: https://github.com/probe-rs/probe-rs/compare/v0.5.1...master
[0.5.1]: https://github.com/probe-rs/probe-rs/releases/tag/v0.5.1
[0.5.0]: https://github.com/probe-rs/probe-rs/releases/tag/v0.5.0
[0.4.0]: https://github.com/probe-rs/probe-rs/releases/tag/v0.4.0
[0.3.0]: https://github.com/probe-rs/probe-rs/releases/tag/v0.3.0
[0.2.0]: https://github.com/probe-rs/probe-rs/releases/tag/v0.2.0
