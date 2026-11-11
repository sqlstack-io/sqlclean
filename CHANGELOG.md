# Changelog

## [0.8.3](https://github.com/sqlstack-io/sqlclean/compare/v0.8.2...v0.8.3) (2026-11-11)


### Bug Fixes

* **deps:** pin SQLQuack 0.5.1 ([#65](https://github.com/sqlstack-io/sqlclean/issues/65)) ([73f2ad4](https://github.com/sqlstack-io/sqlclean/commit/73f2ad4542b02a367613fe52999327dbf4aa37bb))

## [0.8.2](https://github.com/sqlstack-io/sqlclean/compare/v0.8.1...v0.8.2) (2026-11-10)


### Bug Fixes

* **deps:** pin SQLQuack 0.5.0 ([#63](https://github.com/sqlstack-io/sqlclean/issues/63)) ([6822395](https://github.com/sqlstack-io/sqlclean/commit/6822395567a9a85fcc248102660c37f199ec9994))

## [0.8.1](https://github.com/sqlstack-io/sqlclean/compare/v0.8.0...v0.8.1) (2026-11-10)


### Bug Fixes

* **deps:** pin SQLQuack 2096c698, its parser and analysis speed-ups ([#61](https://github.com/sqlstack-io/sqlclean/issues/61)) ([6d3360f](https://github.com/sqlstack-io/sqlclean/commit/6d3360fbb756c5c884b9a8abd6ef3f6774f178d8))

## [0.8.0](https://github.com/sqlstack-io/sqlclean/compare/v0.7.0...v0.8.0) (2026-11-10)


### Features

* **format:** lay out SQL Server table hints, table samples, table variables, FOR XML, FOR JSON, FOR SYSTEM_TIME and join hints ([#59](https://github.com/sqlstack-io/sqlclean/issues/59)) ([b781945](https://github.com/sqlstack-io/sqlclean/commit/b781945551fabda4d6fda30da9932d2475231ffa))

## [0.7.0](https://github.com/sqlstack-io/sqlclean/compare/v0.6.5...v0.7.0) (2026-11-10)


### Features

* format Databricks with SQLQuack ([#57](https://github.com/sqlstack-io/sqlclean/issues/57)) ([e027a95](https://github.com/sqlstack-io/sqlclean/commit/e027a9560e5992776eef2af85073d75c60933e42))

## [0.6.5](https://github.com/sqlstack-io/sqlclean/compare/v0.6.4...v0.6.5) (2026-11-09)


### Bug Fixes

* **deps:** pin SQLQuack f53c6a07, the new root of SQLQuack main ([#55](https://github.com/sqlstack-io/sqlclean/issues/55)) ([46bbaaa](https://github.com/sqlstack-io/sqlclean/commit/46bbaaaea76fc99042b15537615fc67432540291))

## [0.6.4](https://github.com/sqlstack-io/sqlclean/compare/v0.6.3...v0.6.4) (2026-11-09)


### Bug Fixes

* **deps:** pin SQLQuack f9e4ae6e; Databricks and SQLite fingerprints of queries with columns named ANY, SOME, ORDER, FOR or WITH change once, as those names now keep their case ([#53](https://github.com/sqlstack-io/sqlclean/issues/53)) ([72feaf3](https://github.com/sqlstack-io/sqlclean/commit/72feaf309bcae7caf24a0c4f35df9c3dc1f6dd4b))


### Documentation

* say the formatter runs on SQLQuack, not Polyglot ([#51](https://github.com/sqlstack-io/sqlclean/issues/51)) ([172ae7d](https://github.com/sqlstack-io/sqlclean/commit/172ae7d2cdc09fe4df349f8205abfc032986a700))

## [0.6.3](https://github.com/sqlstack-io/sqlclean/compare/v0.6.2...v0.6.3) (2026-11-09)


### Build System

* **deps:** pin SQLQuack 353861ea, after its layout refactor ([#48](https://github.com/sqlstack-io/sqlclean/issues/48)) ([ff5ea5a](https://github.com/sqlstack-io/sqlclean/commit/ff5ea5ab469580873a9e22da70df615765748ea4))

## [0.6.2](https://github.com/sqlstack-io/sqlclean/compare/v0.6.1...v0.6.2) (2026-11-09)


### Bug Fixes

* **format:** lay out CTE macros with comments between them ([#46](https://github.com/sqlstack-io/sqlclean/issues/46)) ([34d709f](https://github.com/sqlstack-io/sqlclean/commit/34d709f1aaab2d43225c67bdc6374ce876414440))

## [0.6.1](https://github.com/sqlstack-io/sqlclean/compare/v0.6.0...v0.6.1) (2026-11-09)


### Bug Fixes

* fingerprint BigQuery and SQL Server with their own tokenizers, so keyword case no longer changes them; their fingerprints change once ([a3c081f](https://github.com/sqlstack-io/sqlclean/commit/a3c081f829a439b54eacab94f00d41a5baf816bc))
* **format:** lay out a placeholder that stands for a CTE's whole query ([a3c081f](https://github.com/sqlstack-io/sqlclean/commit/a3c081f829a439b54eacab94f00d41a5baf816bc))
* **format:** lay out Snowflake paths after a qualified column spelled like a keyword, such as `s.body:ts` ([a3c081f](https://github.com/sqlstack-io/sqlclean/commit/a3c081f829a439b54eacab94f00d41a5baf816bc))
* **format:** lay out T-SQL variable assignments, named EXEC arguments and DECLARE defaults ([a3c081f](https://github.com/sqlstack-io/sqlclean/commit/a3c081f829a439b54eacab94f00d41a5baf816bc))


### Build System

* **deps:** pin SQLQuack 804864d2 ([a3c081f](https://github.com/sqlstack-io/sqlclean/commit/a3c081f829a439b54eacab94f00d41a5baf816bc))

## [0.6.0](https://github.com/sqlstack-io/sqlclean/compare/v0.5.0...v0.6.0) (2026-11-08)


### Features

* format BigQuery, PostgreSQL and T-SQL with SQLQuack ([#42](https://github.com/sqlstack-io/sqlclean/issues/42)) ([a813f5f](https://github.com/sqlstack-io/sqlclean/commit/a813f5ff20eee85e9b7e63bfe0932f4d466dd934))


### Documentation

* say which dialects check, fix and format read ([#40](https://github.com/sqlstack-io/sqlclean/issues/40)) ([b504e28](https://github.com/sqlstack-io/sqlclean/commit/b504e28aa4d17ad3721b334850c860ea11c8431a))

## [0.5.0](https://github.com/sqlstack-io/sqlclean/compare/v0.4.3...v0.5.0) (2026-11-08)


### Features

* format and lint with SQLQuack instead of Polyglot ([#38](https://github.com/sqlstack-io/sqlclean/issues/38)) ([6c0d395](https://github.com/sqlstack-io/sqlclean/commit/6c0d395890b24b36e8f3f233ea550fafb14ee07b))


### Documentation

* simplify introductory custom Python Rule examples ([#34](https://github.com/sqlstack-io/sqlclean/issues/34)) ([14f613c](https://github.com/sqlstack-io/sqlclean/commit/14f613c8464ccc176261fc60fb06727f25066db0))

## [0.4.3](https://github.com/sqlstack-io/sqlclean/compare/v0.4.2...v0.4.3) (2026-11-02)


### Performance Improvements

* **rules:** find unused CTEs without cloning each WITH query ([#30](https://github.com/sqlstack-io/sqlclean/issues/30)) ([c8ce21f](https://github.com/sqlstack-io/sqlclean/commit/c8ce21f023f169d9e778647cf6e2f750ba5e7825))

## [0.4.2](https://github.com/sqlstack-io/sqlclean/compare/v0.4.1...v0.4.2) (2026-11-01)


### Bug Fixes

* **rules:** end the SQLCLEAN053 ON scan at a PIVOT USING list ([aed227a](https://github.com/sqlstack-io/sqlclean/commit/aed227a01101d434a2e43098468400989b60537f))


### Performance Improvements

* **rules:** find SET assignments in one forward pass ([aed227a](https://github.com/sqlstack-io/sqlclean/commit/aed227a01101d434a2e43098468400989b60537f))

## [0.4.1](https://github.com/sqlstack-io/sqlclean/compare/v0.4.0...v0.4.1) (2026-11-01)


### Bug Fixes

* **rules:** let SQLCLEAN004 skip constant-only projections ([#22](https://github.com/sqlstack-io/sqlclean/issues/22)) ([57fc554](https://github.com/sqlstack-io/sqlclean/commit/57fc554518ab9c943f116dfcf0992d7c2a5316b4))

## [0.4.0](https://github.com/sqlstack-io/sqlclean/compare/v0.3.0...v0.4.0) (2026-10-31)


### Features

* **rules:** let a host supply the SQLCLEAN044 guidance ([#20](https://github.com/sqlstack-io/sqlclean/issues/20)) ([ae670ed](https://github.com/sqlstack-io/sqlclean/commit/ae670ed5a781a47e225122f60d4623f75915a3e2))

## [0.3.0](https://github.com/sqlstack-io/sqlclean/compare/v0.2.0...v0.3.0) (2026-10-31)


### ⚠ BREAKING CHANGES

* SQC and XSQC codes are now SQLCLEAN and XSQLCLEAN, and the lint request replaces the two harness flags with fixture_harness.

### Features

* name built-in Rules SQLCLEAN and own the host contract ([#17](https://github.com/sqlstack-io/sqlclean/issues/17)) ([c8c6b30](https://github.com/sqlstack-io/sqlclean/commit/c8c6b3087cae7505b681882cda79ed7e7fdbab2d))


### Bug Fixes

* **rules:** keep SQLCLEAN013, 030 and 003 results meaning-preserving ([#19](https://github.com/sqlstack-io/sqlclean/issues/19)) ([3b9f539](https://github.com/sqlstack-io/sqlclean/commit/3b9f539197edc351753a8467059109a1001517f1))

## [0.2.0](https://github.com/sqlstack-io/sqlclean/compare/v0.1.1...v0.2.0) (2026-10-31)


### Features

* **rules:** report queries that cannot do what they appear to ([#15](https://github.com/sqlstack-io/sqlclean/issues/15)) ([ec34b41](https://github.com/sqlstack-io/sqlclean/commit/ec34b41dc900a2244b63c403262db0dbb1269bf1))

## [0.1.1](https://github.com/sqlstack-io/sqlclean/compare/v0.1.0...v0.1.1) (2026-10-31)


### Bug Fixes

* **testing:** keep Rule case offsets on Windows, and show an email Rule in the README ([#12](https://github.com/sqlstack-io/sqlclean/issues/12)) ([e66e5f5](https://github.com/sqlstack-io/sqlclean/commit/e66e5f570d1df8bc1ba20ff4143f5e00986e29c8))


### Performance Improvements

* **format:** stop re-parsing and re-tokenizing work the formatter already has ([#8](https://github.com/sqlstack-io/sqlclean/issues/8)) ([947258f](https://github.com/sqlstack-io/sqlclean/commit/947258fc361a3a23d3b3667e92a04daa3e4c8d23))


### Documentation

* polish the README header, opening, demo and benchmark table ([#10](https://github.com/sqlstack-io/sqlclean/issues/10)) ([5587732](https://github.com/sqlstack-io/sqlclean/commit/55877326c781e539c80161f2c56a356603973ec5))
* restructure the README around the SQLBuild layout ([#7](https://github.com/sqlstack-io/sqlclean/issues/7)) ([0ca6507](https://github.com/sqlstack-io/sqlclean/commit/0ca65070d6aa04b6e048fe3928d5009b3bf9b700))


### Maintenance

* **release:** set the release baseline to the rewritten 0.1.0 commit ([#13](https://github.com/sqlstack-io/sqlclean/issues/13)) ([df52419](https://github.com/sqlstack-io/sqlclean/commit/df52419924eb1a1bd6aefd08d0e8e1e1252e9235))

## [0.1.0](https://github.com/sqlstack-io/sqlclean/compare/v0.0.1...v0.1.0) (2026-10-31)


### Features

* add the SQLClean engine, command and Python custom Rules ([#1](https://github.com/sqlstack-io/sqlclean/issues/1)) ([bf9b7a9](https://github.com/sqlstack-io/sqlclean/commit/bf9b7a9d449b36223a7159083f8d53f8cb3c81ab))


### Documentation

* add a placeholder README ([60fb468](https://github.com/sqlstack-io/sqlclean/commit/60fb4687fb907449c562d849ebba7dd386d3016b))

## Changelog
