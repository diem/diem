# Trusted Computing Base

The [trusted computing base](https://en.wikipedia.org/wiki/Trusted_computing_base) (TCB) of each Diem validator
is responsible for performing security critical operations and managing cryptographic keys. It is an optional
component of Diem Core, designed to improve the security of Diem validators.

If the TCB of a Diem validator remains secure (i.e., uncompromised), it is able to ensure that the validator
will not violate any safety properties in the network (e.g., forks). In practice, the TCB should be deployed
in a separate environment from the rest of the system (e.g., using a different set of containers, or deployed
on a different host).

The security properties offered by the TCB exclude issues of liveness; liveness may be violated if system
components outside the TCB are compromised on a significant number of machines.

Overall, the TCB consists of four different components:

* [Safety Rules](safety_rules/README.md): ensures that a validator participates in consensus according to the safety rules.
* [Execution Correctness](execution_correctness/README.md): ensures that transactions are correctly executed on each validator.
* [Key Manager](key_manager/README.md): manages and rotates security critical cryptographic keys.
* [Secure Storage](secure_storage/README.md): provides a secure storage for sensitive data on each validator (e.g., cryptographic keys).
