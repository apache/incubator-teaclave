# Hybrid Memory Safety (HMS) and None-bypassable Security Paradigm (NbSP)

The name MesaTEE originates from **Me**mory **Sa**fe **TEE**.  Trusted
Execution Environments (TEEs) have already provided effective foundation for
secure computating. For example, Intel® provides a processor feature called
Software Guard Extensions ([Intel® SGX](https://software.intel.com/en-us/sgx))
through memory access enforcement and automatic memory encryption.  However,
TEEs may still have memory unsafe vulnerabilities like use after free, double
free, buffer overflow, etc. and could be [exploited by
attackers](https://www.usenix.org/conference/usenixsecurity17/technical-sessions/presentation/lee-jaehyuk).
This further indicates privacy leakage or intellectual property compromise.
Researchers have proposed [several
techniques](https://cps.kaist.ac.kr/papers/ndss17-sgxshield.pdf) for hardening
TEEs, but we still need an ultimate solution to provide verifiable memory
safety. 

There are multiple paths towards TEE memory safety.  The most straightforward
one is to use formal verification to prove existing TEE software stack as
memory safe; if violations occur, fix them. As mentioned above, this would be
too expensive to be practical.  A concrete example is the attempt to [formally
verify
seL4](https://www.sigops.org/s/conferences/sosp/2009/papers/klein-sosp09.pdf),
where at least 8 human years were spent.  Any tiny modification to the code
would consume similar cost again.

The other direction is to rewrite everything using memory safe languages.
Thankfully, memory safety languages like [Rust](https://www.rust-lang.org/) are 
becoming more and more popular and mature. They intrinsically guarantee memory
and thread safety during compilation. At the same time, the performance is
almost equivalent to C/C++ programs (see
[here](https://greenlab.di.uminho.pt/wp-content/uploads/2017/09/paperSLE.pdf)
or [here](https://www.techempower.com/benchmarks/)).  Most importantly, writing
programs in memory safe languages is much cheaper comparing to formal verification.
Hence we believe that Rust, the most outstanding memory safe system programming
language by far, best fits the TEE world.

However, rewriting everything in Rust is not easy.  Sometimes we have to rely
on existing unsafe components to reduce the development cycle.  It is important
to reach a good balance between safety, compatibility and functionality.  To
achieve that, we come up with the idea of Hybrid Memory Safety and the
following rule-of-thumbs:

1. Unsafe components should be appropriately isolated and modularized, and the
size should be small (or minimized)
2. Unsafe components should not weaken the safe, especially, public APIs and
data structures
3. Unsafe components should be clearly identified and easilyupgraded.

Here the unsafe components include both the modules written in memory-unsafe
languages (such as C/C++), and the
[unsafe](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html) code in
memory-safe languages. MesaTEE and its sibling projects ([Rust SGX
SDK](https://github.com/baidu/rust-sgx-sdk), [MesaLock
Linux](https://github.com/mesalock-linux/mesalock-distro),
[MesaLink](https://github.com/mesalock-linux/mesalink), and
[MesaPy](https://github.com/mesalock-linux/mesapy)) are all trying to adhere to
these rules.

>**MesaTEE combines the power of the Baidu HMS model and Intel® SGX to provide a
>breakthrough solution to expand the trust boundary of the Internet," said Tao
>Wei, Chief Security Scientist of Baidu. " The HMS model has revolutionized
>memory safety for systems at the software architecture level. Intel® SGX,
>meanwhile, dramatically shortens the trust chain of computing and makes trusted
>dependencies more simplified, reliable, and secure. Together, MesaTEE provides
>the foundation for incubating next-generation blockchains, privacy-enhanced
>cloud computing, and other new Internet services.**

Once with the memory safety promises, control flow and data flow integrities
can be easier to enforce.  Security audit also becomes very convenient.  Based
on it, we can further achieve the [Non-bypassable Security
Paradigm](https://github.com/baidu/rust-sgx-sdk/blob/master/documents/nbsp.pdf).
This ensures that components cannot be hijacked to take wrong paths, and all
control/data flows must pass the critical security checkpoints.  That is the
ultimate goal of our endeavor.
