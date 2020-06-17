---
permalink: /docs/codebase/tests
---

# Test Harness and Test Cases

This directory contains all tests in Teaclave including unit tests, integration
tests, functional tests and some test fixtures.

## Run Tests

To run all tests with our build system:

```
$ make run-tests
```

You can also run tests separately:

```
$ make run-unit-tests
$ make run-integration-tests
$ make run-functional-tests    # this will start all services in the background automatically
```

## Test Coverage

To generate a coverage report for tests, you can configure cmake with
`-DCOV=ON`. Then build the platform and run all tests. At last, you need to run
`make cov` to aggregate coverage results.

## Directory Structure

- `unit`:
  Unit tests are small and more focused, testing one module in isolation at a
  time, and can test private interfaces. This directory contains a test driver to
  test individual units/components or private interfaces. Test cases of unit
  tests are placed along with source code.
- `integration`:
  Integration tests are entirely external to libraries, using only the public
  interface and potentially exercising multiple modules per test. This directory
  contains a test driver and test cases to test public interfaces in common
  libraries.
- `functional`:
  Functional testing is a type of black-box testing. In Teaclave, the test cases
  are usually sent through RPC channel.
  This directory contains a test driver and test cases for Teaclave services. To
  run these tests, services need to be launched.
- `fixtures`:
  Testing fixtures are some files and sample inputs/outputs for testing only.
- `utils`:
  Common utilities for test drivers.
