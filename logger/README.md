---
permalink: /docs/codebase/logger
---

# Logger

A logger for Teaclave services. It can collect logs to a buffer.
Logs not saved to buffer can be logged by another logger that
implements `log` trait.

## Task logging

When the logger is imported in the `execution service`, it can send the logs
during a task to a buffer. The `kv_unstable` feature in the `log` crate is used
to pass the pointer to the buffer to the logger. After the buffer is set, the
logger will save logs to the buffer. The logger will drop the task logger after
receiving a null pointer. Another logger which we call `secondary logger` will
handle the logs coming afterwards if it is set. The logs before the task starts
are sent to the secondary logger as well.
