# Teaclave Command Line Tool

`teaclave_cli` is a command-line tool to communicate with Teaclave's services.

## Usage

There are two sub-commands: `teaclave_cli audit` is for auditing enclave info
with auditors' public keys and signatures and `teaclave_cli connect` is for
communicating with Teclave's services.

Here are details of the arguments for `teaclave_cli audit`:

```
$ ./teaclave_cli audit --help
teaclave_cli-audit 0.1.0
MesaTEE Authors <developers@mesatee.org>
Audit enclave info with auditors' public keys and signatures.

USAGE:
    teaclave_cli audit --enclave_info <ENCLAVE_INFO_FILE> --auditor_public_keys <auditor_public_keys>... --auditor_signatures <auditor_signatures>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --enclave_info <ENCLAVE_INFO_FILE>                Path to Enclave info file.
    -k, --auditor_public_keys <auditor_public_keys>...    SPACE separated paths of Teaclave auditor public keys
    -s, --auditor_signatures <auditor_signatures>...
            SPACE separated paths of Teaclave auditor endorsement signatures.
```

Here are details of the arguments for `teaclave_cli connect`:

```
./teaclave_cli connect --help
teaclave_cli-connect 0.1.0
MesaTEE Authors <developers@mesatee.org>
Connect and send messages to Teaclave services

USAGE:
    teaclave_cli connect [OPTIONS] <IP_ADDRESS:PORT> --enclave_info <ENCLAVE_INFO_FILE> --endpoint <endpoint>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --enclave_info <ENCLAVE_INFO_FILE>    Path to Enclave info file.
    -o, --output <INPUT_FILE>                 Write to FILE instead of stdout.
    -i, --input <OUTPUT_FILE>                 Read from FILE instead of stdin.
    -e, --endpoint <endpoint>                 Teaclave endpoint to connect to. Possible values are: tms, tdfs, fns.

ARGS:
    <IP_ADDRESS:PORT>    Address and port of the Teaclave endpoint.
```

## Example

The following is an example of verifying the enclave info with auditors' public
keys and signatures.

```
$ ./teaclave_cli audit \
  -c enclave_info.toml \
  -k auditors/albus_dumbledore/albus_dumbledore.public.der \
     auditors/godzilla/godzilla.public.der \
     auditors/optimus_prime/optimus_prime.public.der \
  -s auditors/albus_dumbledore/albus_dumbledore.sign.sha256 \
     auditors/godzilla/godzilla.sign.sha256 \
     auditors/optimus_prime/optimus_prime.sign.sha256
Enclave info is successfully verified.
```

This example is to create a task and invoke the "echo" function.

```
# create a task
$ cat create_task.json
{
  "type":"Create",
  "function_name":"echo",
  "collaborator_list":[],
  "files":[],
  "user_id":"user1",
  "user_token":"token1"
}
$ cat create_task.json | ./teaclave_cli connect 127.0.0.1:5554 -c enclave_info.toml -e tms
{"type":"Create","task_id":"20937006-2718-4f33-bae2-567933807436","task_token":"d20ce53ab743d69320712fd98555f5e5","ip":"127.0.0.1","port":3444}
```

Compose a invoke task request with `task_id`, `task_token`, `ip` and `port` in
the previous response.

```
# invoke the "echo" function
$ cat invoke_task.json
{
  "task_id":"20937006-2718-4f33-bae2-567933807436",
  "function_name":"echo",
  "task_token":"d20ce53ab743d69320712fd98555f5e5",
  "payload":"Hello, World!"
}

$ cat invoke_task.json | ./teaclave_cli connect 127.0.0.1:3444 -c enclave_info.toml -e fns
{"result":"Hello, World!"}
```
