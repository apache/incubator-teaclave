# How to Run

[config.toml](../config.toml) contains all the runtime configurations.

First of all, please refer to the [build
prerequisite](how_to_build.md#prerequisite) document for Intel Attestation
Service (IAS) registration. Once obtained the IAS TLS communication
IAS SPID and Keys, please put their paths in the ``ias_client_config`` section of
[config.toml](../config.toml).  

The ``api_endpoints`` and ``internal_endpoints``  of
[config.toml](../config.toml) specify the listening IPs and ports of MesaTEE
services. Please configure them accordingly.

Then, please set SPID and key (either primary or secondary) from Intel Trusted
Service API portal by ``cat YOUR_SPID > ./bin/ias_spid.txt && cat YOUR_KEY > ./bin/ias_key.txt``.

Afterwards, you can launch MesaTEE services as background daemons by running:
``./service.sh {start|stop|restart}``

## SGX Simulation Mode

You can change ``-DSGX_MODE=HW`` to ``-DSGX_MODE=SW`` when running cmake to enable SGX
simulation mode (``make clean && make`` required).
In the simulation mode, MesaTEE won't really connect to IAS to fetch reports,
nor perform remote attestation during the TLS communications. So basically it
enables you to freely run on arbitrary platforms to test the functionalities.
