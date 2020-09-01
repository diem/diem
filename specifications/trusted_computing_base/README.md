# Trusted Computing Base

The trusted computing base (TCB) part of libra-core is in charge of keys and important operations.
It is an optional component of libra-core, designed to improve the security of the system.

The idea of a [trusted computing base](https://en.wikipedia.org/wiki/Trusted_computing_base) is to minimize and segregate the parts needed to for the well-being of a validator.

For libra-core, the TCB runs the code necessary for a validator to be correct in order for the validator not to become unsafe.
If other parts of the system start misbehaving, the validator as a whole will not be able to perform operations that would help compromise the safety of the network.
This obviously does not cover liveness issues.

The TCB consists of four different components:

* [Safety Rules](./safety_rules.md): ensures that a validator participate in the consensus protocol according to the safety rules.
* [Execution Correctness](execution_correctness/execution_correctness_specification.md): ensures that transactions are correctly executed.
* [Key Manager](key_manager/key_manager_specification.md): manages and rotate important keys.
* [Secure Storage](./secure_storage.md): ensures storage of sensitive data for the Libra blockchain
