# Changelog


## [0.8.3] - 2025-05-02
### Modified
- fix a bug where cjval crashed if the geometry-templates had boundaries to unexisting vertices.

## [0.8.2] - 2025-03-14
### Added
- Docker image added

## [0.8.1] - 2024-08-14
### Modified
- fix a bug where cjval crashed if the textures for a geom where containing `[null, null, null]` instead of one null. The specs allow several so it's fine, I guess. Now it reports valid. 

## [0.8.0] - 2024-06-20
### Modified
- cjseqval removed as a standalone binary, now you can do the same with cjval (stdin is for CityJSONSeq)
- the reporting of the errors for CityJSONSeq is better, and contains a summary

## [0.7.1] - 2024-04-16
### Modified
- fix a bug where Solid with textures having null values were wrongly validated


## [0.7.0] - 2024-04-09
### Modified
- cjfval renamed to cjseqval (because now it's called CityJSON Sequences)
- support for CityJSON schemas v2.0.1
- fixed bugs for CityJSON v2.0 files having extensions (schemas v1.1 were used)
- fixed some bugs 
- improved the CLI output
- move to an newer version of the CLI parser "clap"

## [0.6.1] - 2024-02-05
### Modified
- fix bug with textures and inner-rings and `[[null]] values (bug #13)

## [0.6.0] - 2023-09-28
### Added
- support for CityJSON v2.0 (schemas v2.0.0 added)
- full support for CityJSONL with cjfval, the stream must come from cjio (or equivalent)
- more unit tests
### Modified
- validation of textures and material coordinates
- cjfval is faster: schemas are not read for each line anymore
- fixed some bugs

## [0.5.0] - 2022-11-04
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



[0.8.3]: https://github.com/hugoledoux/cjval/compare/0.8.2...0.8.3
[0.8.2]: https://github.com/hugoledoux/cjval/compare/0.8.1...0.8.2
[0.8.1]: https://github.com/hugoledoux/cjval/compare/0.8.0...0.8.1
[0.8.0]: https://github.com/hugoledoux/cjval/compare/0.7.1...0.8.0
[0.7.1]: https://github.com/hugoledoux/cjval/compare/0.7.0...0.7.1
[0.7.0]: https://github.com/hugoledoux/cjval/compare/0.6.1...0.7.0
[0.6.1]: https://github.com/hugoledoux/cjval/compare/0.6.0...0.6.1
[0.6.0]: https://github.com/hugoledoux/cjval/compare/0.5.0...0.6.0
[0.5.0]: https://github.com/hugoledoux/cjval/compare/0.4.3...0.5.0
[0.4.3]: https://github.com/hugoledoux/cjval/compare/0.4.2...0.4.3
[0.4.2]: https://github.com/hugoledoux/cjval/compare/0.4.1...0.4.2
[0.4.1]: https://github.com/hugoledoux/cjval/compare/0.4.0...0.4.1
[0.4.0]: https://github.com/hugoledoux/cjval/compare/0.3.1...0.4.0
[0.3.1]: https://github.com/hugoledoux/cjval/compare/0.3.0...0.3.1
[0.3.0]: https://github.com/hugoledoux/cjval/compare/0.2.0...0.3.0

