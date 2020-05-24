---
permalink: /tests
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

## Directory Structure

- `unit`:
  Unit tests are small and more focused, testing one module in isolation at a
  time, and can test private interfaces. This directory contains test driver to
  test individual units/components or private interfaces. Test cases of unit
  tests are placed along with source code.
- `integration`:
  Integration tests are entirely external to libraries, using only the public
  interface and potentially exercising multiple modules per test. This directory
  contains test driver and tests cases to test public interfaces in common
  libraries.
- `functional`:
  Functional testing is a type of black-box testing. In Teaclave, the test cases
  are usually sent through RPC channel.
  This directory contains test driver and tests cases for Teaclave services. To
  run these tests, services need to be launched.
- `fixtures`:
  Testing fixtures are some files and sample inputs/outputs for testing only.
- `utils`:
  Common utilities for test drivers.
