---
permalink: /contributing
---

# Contributing to Teaclave

As an open-source community, we welcome all kinds of contributions. You can
contribute to Teaclave in many ways: reporting issues, requesting new features,
proposing better designs, fixing bugs, implementing functions, improving
documents, trying novel research ideas or even by simply using and promoting
this project.

## Submit Issues

We prefer to use GitHub issues for almost everything about the project
development such as issues tracking, features, design proposals, announcements,
community communications, etc. Free feel to open an issue if you meet bugs or
want to propose features.

## Send Pull Requests

This is a basic instruction to send a pull request to Teaclave.

1. Fork the repository on GitHub.
2. Create a new branch for the feature or bugfix.
3. Make changes.
4. Test. The `make run-tests` command will run all test case.
5. Make sure to format and lint the code. You can use `make format` to format
   code inplace, and `make CLI=1` to lint Rust code with Rust clippy.
6. Commit/push the changes and send a pull request on GitHub. Please kindly
   write some background and details for this PR (we also provide a PR template
   to guild you with writing a high-quality pull request).
