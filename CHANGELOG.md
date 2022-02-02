# Changelog


## [0.4.2] - 2022-02-02
### Modified
- fix bug where v1.0 files were validated with the v1.1 schema

## [0.4.1] - 2022-02-02
### Modified
- fix bugs related to the warnings for ignoring vertices in GeometryTemplate and "address" and MultiPoint and MultiLineString 
- now uses CityJSON schemas v1.1.1


## [0.4.0] - 2022-01-05
### Added
- unit tests for Extensions
- added cjvalext to validate Extensions files (well started it's pretty bare right now)
### Modified
- fixed a few bugs (double report of some errors)
- fixed a bug: City Objects in Extensions can be reused by other COs


## [0.3.1] - 2021-10-27
### Added
- better docs + unit tests


## [0.3.0] - 2021-10-27
### Added
- first version that works with Extensions