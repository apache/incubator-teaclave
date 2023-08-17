# Introduction

This policy-compliant data analysis framework is the *first* to provide the following functionalities to all companies that collect sensitive user data:

- Fully automatic generation of the physical data access layer.
- Formal foundations: we aim to formally verify the framework along with the access layer.
- Integration with the Trusted Execution Environment (TEE): to further enhance the security of user data, we leverage the TEE technology to perform the data analysis tasks on the cloud.

# An End-to-End Workflow

The data analysis framework can be roughly divided into the following components:

- The policies collected from
    - each user’s preference with the data being analyzed on the platform and/or
    - the privacy regulations and/or the data curators’ published policies (e.g., see “how Google will use your data and personal information”).
- The core library which performs the data analysis jobs by converting the raw query into machine-interpretable query plans which are eventually dispatched to
- Policy-compliant data access layers (a.k.a., the physical executor) whose behavior have been formally verified* (* The process of generation is also formally verified)

To give users more confidence about the confidentiality of their data used, the data analysis platform is deployed inside a Trusted Execution Environment (TEE) that enjoys the following attractive properties:

- Intrusion prevention: it “shelters” the confidential data from any outside attackers on the cloud like the (possibly) malicious cloud service provider.
- Data encryption.
- Attestation to the users that the code running on the cloud is indeed the desired one and is not tampered.

We will use three TEE containers to jointly perform the task in order to provide the maximal protection for the data. The first container is the storage layer where user data is stored and secured, and all of the contents are fully encrypted so that unauthorized parities cannot steal them. The second container is the executor container that registers, invokes, and destroys the policy-compliant physical executors that will interact with the first container (recall its storage functionality). The last one hosts the core data analysis library that provides all the non-policy-related functionalities. The workflow goes as follows.

1. The data provider (curator, users, etc.) uploads the data to the platform along with the prescribed privacy policy.
2. The platform interprets the privacy policy and checks if the policy is self-consistent (where “consistency” means that policy itself cannot be self-contradictory like “data A can be read” while “data A cannot be read”).
    1. If the policy passes the check, the framework will **automatically** generates the executors for this particular data by registering them to the second container.
    2. If the policy cannot be used, we abort and send an error message back to the data provider indicating that we do not accept such a policy and that they should refine it.
3. Programs are free of issuing any queries to the framework which parses the into query plans and send to each specific executors.
4 Executors either abort the query on a policy breach or continue to perform the task.
5. To ensure data use security, we can also apply the PoBF security principles to the framework.

# Detailed Designs

Please check out the following links:

- [Algebraic Structures](./algebraic_structures.md)
- [Differential Privacy](./differential_privacy.md)
- [Executors](./policy_to_executors.md)
- [Policy Language](./policy_language.md)
- [Query Plans](./query_plan.md)
- [Formal Foundations](./formal_foundations.md)

# API References

Please use Rust's internal doc generator `cargo doc` to check the detailed API references of the framework.

# Misc

We have not named this project yet, and if you have any interesting naming ideas, please do not hesitate to share it with us! We would really appreciate it.
