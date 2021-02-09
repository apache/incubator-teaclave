# Formal description and verification for access control module

## Introduction

Instead of modelling code directly, the formal language used here models desgin of access control module based on the specification described in [model.conf]. Security functional requirements described in part2 of Common Criteria are used for decomposing the specification and algebra structure is used for constructing each security functional requirement. The formal description of access control module serves as two purposes: guides the implementation of the access control module; ensure the implementation comply with the specification at desgin level, which means there is no undefined behaviour.  

Isabelle is required to open files suffixed with thy. The mechanism called Locale is used for constructing algebra structures.

## Algebra structures
[In mathematics, an algebraic structure consists of a nonempty set A, a collection of operations on A of finite arity, and a finite set of identities, knowns as axioms,that these operations must satisfy.](https://en.wikipedia.org/wiki/Algebraic_structure).This can be used for constructing programs if nonempty set is considered as data structure or type, operations are considered as functions and axioms are considered as function contracts that must be obeyed.

### [AttrConf.thy]
This works as a common structure for describing successive structures, such as list, array and so on. It will be imported to other structures. 

### [InfoType.thy]
This is used for describing the objects to be accessed. It corresponds to the objects described in [*access-control.md*].Enumeration type can be considered as a concrete example for this structure. 

### [SysId.thy]
This is used for descirbing Ids required by the specification of access control module. It works as an attribute of entities involved in the access control module.

### [TrustLevel.thy]
This is used for describing the location of entites. It works as an attribute of entities involved in the access control module.

### [ResrcType.thy]
This is used for describing the type of resources used by access control module. It works as an attribute of entities involved in the access control module.

### [UsrAttr.thy]
This is used for describing user attribute of access control module.  

### [ResrcAttr]
This is used for describing attributes of different entities. The entities include subject, object and information.

### [FDP_ACF.thy]
This is used for describing the security functional requirement identified by [*FDP_ACF*] in Common Criteria.It works as a component to construct the access control module.

### [FIA_USB.thy]
This is used for describing the security functional requirement identified by [*FIA_USB*] in Common Criteria.It works as a component to construct the access control module.

### [FMT_MSA.thy]
This is used for describing the security functional requirement identified by [*FMT_MSA*] in Common Criteria.It works as a component to construct the access control module.

### [ModelConf.thy]
This is used for describing the configuration file required by access control module.

### [FDP_ACC.thy]
This is used for describing the security functional requirement identified by [*FDP_ACC*] in Common Criteria.It works as a component to construct the access control module.

### [TeaclaveAccessControl.thy]
This is the "main" description of access control module. It is the composition of security functional requirements defined in other thy files. It also contains the interface description of the access control module.

### [TeaclaveRequirements.thy]
This is the formal proof based on the access control module description. It shows that the description comply with the specifications required by the access control module. 

## Interpretation
The axioms described in the thy files of the [*Algebra structures*] might not be consistent. In order to ensure the consistency between different axioms, an instance is provided for each thy file of [*Algebra structures*].It can also be considered as the implementation of access control module in Isabelle. 


