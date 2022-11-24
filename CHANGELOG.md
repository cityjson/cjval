# Changelog


## [0.5.] - 2022-11-04
### Added
- cjfval binary to validate CityJSONFeatures
- the docs is now added with examples how to use the library
### Modified
- behaviour change: one function validate() will perform all the appropriate checks (errors+warnings) and return one summary (an HashMap)
- modified the return values of most functions, now Result are used
- upgraded the schemas to v1.1.3
- fixed some bugs


## [0.4.3] - 2022-08-16
### Modified
- upgraded the schemas to v1.1.2
- upgraded one dependency (serde_with) to latest version and removed the warning when compiling

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