# Changelog

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
