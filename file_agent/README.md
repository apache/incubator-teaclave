---
permalink: /docs/codebase/file-agent
---

# File Agent

The file agent is a component in the execution service. The main function is to
handle file downloading/uploading from and to various storage service providers
(e.g., AWS S3).

Before executing a task, the execution service will use the file agent to
prepare any registered input files that come with the task. For example, the
registered file input could be a presigned URL from AWS S3. The file agent will
download and prepare the file in local. With these files in the local storage,
the executor can finally invoke the function. Similarly, after the task is
successfully executed, the file agent will help to upload the output files to
a remote file storage like S3.
