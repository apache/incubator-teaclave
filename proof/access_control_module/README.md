# Formal description and verification for access control module

## Motivation

The Vulnerabilities of an IT product, such as teaclave access control module, 
are often caused by the failures in the compliance between requirements and 
specifications. In order to achieve high confidence of teaclave access control 
module, the formal description of specification and requirements is required to 
eliminate the ambiguity introduced by natural language. Moreover, the formal 
verification is also required to show the compliance between specifications and 
requirements. 

## Introduction

Instead of modeling code directly, the formal language used here describes the 
design/specifications of access control module and the requirements written in
[model.conf][0]. To be more specific, the specifications are composed by a set of
security functional requirements, also known as SFR. Algebra structure is used for 
constructing each security functional requirement. The requirements are formalized
by the primitives declared in specifications. The proof procedure is to show the
compliance between requirements and  specifications.

  [0]: https://github.com/apache/incubator-teaclave/blob/master/services/access_control/model.conf

## Prerequisite

To completely understand the design principle and the semantics of formal language 
used here, learning Isabelle and Common Criteria is required. If you are familiar 
with Isabelle and Common Criteria, you can skip this section.

### Isabelle
Isabelle is a generic proof assistant. To learn the main concept, syntax and others 
of Isabelle, please refer to [`isabelle tutorial`][1].

The specifications are constructed using locale and interpretation provided by Isabelle.
To learn more about locale and interpretation, please refer to [`locale and interpretation tutorial`][2].

### Common Criteria
Common Criteria for Information technology is an international standard for the 
evaluation of Security of products. It documents a set of security functional 
requirements which is used for decomposing the requirements. The requirements 
of teaclave access control module is decomposed based on the security functional 
requirements. For the description of security functional requirements, please refer 
to [`Common Criteria part 2`][3].

### Algebra structures
In mathematics, an algebraic structure consists of a nonempty set A, a
collection of operations on A of finite arity, and a finite set of identities,
knowns as axioms, that these operations must satisfy. This can be used for 
constructing programs if nonempty set is considered as data structure or type, 
operations are considered as functions and axioms are considered as function 
contracts that must be obeyed. For more information, please refer to [`wiki`][4].

  [1]: https://isabelle.in.tum.de/dist/Isabelle2021/doc/prog-prove.pdf
  [2]: https://isabelle.in.tum.de/dist/Isabelle2021/doc/locales.pdf
  [3]: https://www.commoncriteriaportal.org/files/ccfiles/CCPART2V3.1R5.pdf
  [4]: https://en.wikipedia.org/wiki/Algebraic_structure
  
## Setup
To open and build the repository, Isabelle engine and IDE are required. Once
the [`Isabelle and IDE`][5] are installed and the repository is [`imported into IDE`][6],
the repository will be built and proved automatically.

  [5]: https://isabelle.in.tum.de/installation.html
  [6]: https://isabelle.in.tum.de/dist/Isabelle2021/doc/jedit.pdf

## Overview

The repository is organised as follows.

### Specification 
As stated before, the design of teaclave access control module are
formalized using locales. The description of teaclave access control
module is in [`TeaclaveAccessControl.thy`](TeaclaveAccessControl.thy). 
This is the "main" description of access control module. The primitives 
used for description are located in other files. The general description 
of the purpose of each file used for composing the "main" description 
of access control module is listed as follows:

  * [`AttrConf.thy`](AttrConf.thy): This works as a common structure for describing 
  successive structures, such as list, array and so on. It will be imported 
  to other structures. 

  * [`InfoType.thy`](InfoType.thy): This is used for describing the objects to be accessed. 
  It corresponds to the objects described in [`access-control.md`][7]. Enumeration type can 
  be considered as a concrete example for this structure. 

  * [`SysId.thy`](SysId.thy): This is used for describing Ids required by the specification 
  of access control module. It works as an attribute of entities involved in the access 
  control module.

  * [`TrustLevel.thy`](TrustLevel.thy): This is used for describing the location of entities. 
  It works as an attribute of entities involved in the access control module.

  * [`ResrcType.thy`](ResrcType.thy): This is used for describing the type of resources used 
  by access control module. It works as an attribute of entities involved in the access control 
  module.

  * [`UsrAttr.thy`](UsrAttr.thy): This is used for describing user attribute of access control module.  

  * [`ResrcAttr.thy`](ResrcAttr.thy): This is used for describing attributes of different entities. 
  The entities include subject, object and information.

  * [`FDP_ACF.thy`](FDP_ACF.thy): This is used for describing the security functional requirement identified
  by `FDP_ACF:USER DATA PROTECTION-Access control functions` in Common Criteria. It works as a component to 
  construct the access control module.

  * [`FIA_USB.thy`](FIA_USB.thy): This is used for describing the security functional requirement identified
  by `FIA_USB:IDENTIFICATION AND AUTHENTICATION-User subject binding` in Common Criteria. It works as a 
  component to construct the access control module.

  * [`FMT_MSA.thy`](FMT_MSA.thy): This is used for describing the security functional requirement identified
  by `FMT_MSA:SECURITY MANAGEMENT-Management of security attributes` in Common Criteria.It works as a 
  component to construct the access control module.

  * [`ModelConf.thy`](ModelConf.thy): This is used for describing the configuration file required by access 
  control module.

  * [`FDP_ACC.thy`](FDP_ACC.thy): This is used for describing the security functional requirement identified
  by `FDP_ACC:USER DATA PROTECTION-Access control policy` in Common Criteria. It works as a component to 
  construct the access control module.
  
  [7]: https://github.com/apache/incubator-teaclave/blob/master/docs/access-control.md

### Requirement
The requirements are formalized and proved in [`TeaclaveRequirements.thy`](TeaclaveRequirements.thy).
This is the formal proof based on the access control module description. 
It shows that the description complies with the specifications required by
the access control module. 

### [`Interpretation`](interpretation/)
The axioms described in locales might not be consistent. In order to 
ensure the consistency between different axioms, an instance is provided 
for each locales. The interpretation can also be considered as the 
implementation of access control module in Isabelle. 

## Future work
The formal description along with the verification only shows the compliance between
requirements and specifications. The compliance between specification and implementation
remain unsolved. Fortunately, the axioms described in locales can be used as inputs to write
test cases following MC/DC procedure. In the future, the test cases based on axioms and MC/DC
will be provided. With the help of test cases and structural coverage analysis, it is adequate
to show the compliance between specification and implementation, so is the compliance between
requirements and implementation. Except for access control module, the formal description, 
verification and test cases will be provided to other modules. 
