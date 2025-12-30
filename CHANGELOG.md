# Changelog

## [0.7.0]

### Changed

- Updated to Bevy 0.18
- Naming pass
  - `OnResponseTyped` -> `ResponseTyped`
  - `OnResponseString` -> `ResponseString`
- reduced dependencies amount

### Fixed

- incorrect http scheme for `http://` assets.

## [0.6.1]

### Fixed

- Update README

## [0.6.0]

### Changed

- Updated to Bevy 0.17
- Bump Rust edition to 2024
- `OnResponseTyped` and `OnResponseString` became `EntityEvent`

## [0.5.0]

### Added

- `RequestResponseExt` trait for response events.

### Changed

- Updated to Bevy 0.16
- Naming pass
  - `OnTypedResponse` -> `OnResponseTyped`
  - `RequestCompleted` -> `OnResponseString`
- Response as a component (`RequestResponse`) is now hidden behind `response_as_component` and likely to be removed in next release.
- `OnResponseTyped` has parsing done on creation. `result` field got renamed to `request` and `PhantomData` field got removed. One new field is `data` which holds `Option<T>` with parsing result.

### Removed

- `TypedRequestQuery` QueryData got removed, detecting typed responses now uses observer for that.

## [0.4.0]

### Added

- `OnTypedResponse` trigger event type.

### Changed

- Updated `typed` example to use `OnTypedResponse` trigger.
- Updated to Bevy 0.15

## [0.3.0]

### Changed

- Updated to Bevy 0.14

## [0.2.1]

### Added

- New default feature `asset_loading`- support for loading assets from web.

## [0.2.0]

### Changed

- Updated to Bevy 0.13
- Updated ehttp to 0.5
- Make `register_request_type` trait method.

## [0.1.0]

- Initial version
