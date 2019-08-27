# MesaTEE CLI

`mesatee-cli` is a command-line utility to communicate with MesaTEE external
endpoints. 

## Usage

```shell
mesatee_cli 0.1.0
MesaTEE Authors <developers@mesatee.org>
MeasTEE client

USAGE:
    mesatee_cli [FLAGS] [OPTIONS] <IP_ADDRESS:PORT> --auditor_keys <auditor_keys>... --auditor_sigs <auditor_sigs>... --enclave_info <enclave_info> --endpoint <endpoint>

FLAGS:
    -h, --help
            Prints help information

    -P, --pretty
            Enable pretty printing.
    
    -V, --version
            Prints version information
    
    -v, --verbosity
            Pass many times for more log output
    
            By default, it'll only report errors. Passing `-v` one time also prints warnings, `-vv` enables info
            logging, `-vvv` debug, and `-vvvv` trace.

OPTIONS:
    -o, --output <IN_FILE>
            Write to FILE instead of stdout

    -i, --input <OUT_FILE>
            Read from FILE instead of stdin
    
    -k, --auditor_keys <auditor_keys>...
            SPACE seperated paths of MesaTEE auditor public keys
    
    -s, --auditor_sigs <auditor_sigs>...
            SPACE seperated paths of MesaTEE auditor endorsement signatures.
    
    -c, --enclave_info <enclave_info>
            Path to Enclave info file
    
    -e, --endpoint <endpoint>
            MesaTEE endpoint to connect to. Possible values are: tms, tdfs, fns.

ARGS:
    <IP_ADDRESS:PORT>
            Address and port of the MeasTEE endpoint
```

Note the `--enclave_info`, `--auditor_keys`, `--auditor_sigs` are required
options. These flags provide auditor information for MesaTEE enclaves. More
details can be found
[here](https://github.com/mesalock-linux/mesatee/blob/master/auditors/README.md).


## Example

Here we give an example of using `mesatee_cli`:

```shell
$ cd mesatee
$ ./service.sh start
$ ./bin/mesatee_cli 127.0.0.1:5554 -k auditors/albus_dumbledore/albus_dumbledore.public.der -k auditors/godzilla/godzilla.public.der -k auditors/optimus_prime/optimus_prime.public.der -s auditors/albus_dumbledore/albus_dumbledore.sign.sha256 -s auditors/godzilla/godzilla.sign.sha256 -s auditors/optimus_prime/optimus_prime.sign.sha256 -c out/enclave_info.txt --endpoint tms -i ~/tms_payload

{"type":"Create","task_id":"7216dd3e-ab3a-4974-b03e-3833891bbb26","task_token":"08e0d4c807700ff24d31ca01d8695b61","ip":"127.0.0.1","port":3444}
```
