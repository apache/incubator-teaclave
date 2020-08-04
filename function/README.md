---
permalink: /docs/codebase/function
---

# Built-in Functions

Since Teaclave is a FaaS-like platform, users can define and register customized
functions (e.g., written in Python). To make data computation more easier and
faster (in native speed), the platform also provide some commonly used functions
written in Rust. We call them built-in functions. These functions can be
selectively compiled in the *built-in executor* with a "builtin" prefix in the
function names. Users can selectively invoke these build-in functions by their
names.

Currently, we have these built-in functions:
  - `builtin-echo`: Return the original input message.
  - `builtin-gbdt-train`: Use input data to train a GBDT model.
  - `builtin-gbdt-predict`: GBDT prediction with input model and input test data.
  - `bulitin-logistic-regression-train`: Use input data to train a LR model.
  - `builtin-logistic-regression-predict`: LR prediction with input model and input test data.
  - `builtin-private-join-and-compute`: Find intersection of muti-parties' input
    data and compute sum of the common items.
  - `builtin-ordered-set-intersect`: Allow two parties to compute the
    intersection of their ordered sets without revealing anything except for the
    elements in the intersection. Users should calculate hash values of each item
    and upload them as a sorted list.
  - `builtin-rsa-sign`: Signing data with RSA key.
  - `builtin-face-detection`: an implementation of Funnel-Structured cascade,
    which is designed for real-time multi-view face detection.
  
The function arguments are in JSON format and can be serialized to a Rust struct
very easily. You can learn more about supported arguments in the implementation
of a specific built-in function.
