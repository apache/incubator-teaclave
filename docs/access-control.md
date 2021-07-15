---
permalink: /docs/access-control
---

# Access Control in Teaclave
Access control in multi-party computation, by its nature, is set-centric. Unlike
in traditional settings where only one entity, group, or role is involved in an
action, in multi-party computation tasks an access is approved or denied by
combining the decisions of all stakeholders. Ownership is also set-like, because
the result of a joint computation task very likely belongs to all parties that
have provided data.

We found that access control models in conventional systems like RBAC and ABAC
are not powerful enough to govern data usages in Teaclave. Therefore, we
invented our own access control model and mechanism.

## Model
The access control model of Teaclave is configured through the file
[model.conf](https://github.com/apache/incubator-teaclave/blob/master/services/access_control/model.conf).
The file has three sections:
  - requests
  - terms
  - matchers

Before diving into the details about how our access control model works, we
recommend readers learn about [logic programming](https://en.wikipedia.org/wiki/Logic_programming)
first, because our
model configuration language is actually a home-baked tiny logic programming
language.

### request
A request is a query that should be answered by the access control service. The
response is either "approved" or "denied." A request is defined as a tuple of
any arity. For example,

```
[requests]
task_access_data = task, data
```

defines a request called `task_access_data` which contains two fields named
`task` and `data`. This request can mean a task with the id `task` wants to
access a piece of data hosted by Teaclave whose id is `data`.

### term
Terms are relations over certain domains. Each term can be viewed as a table
storing facts about the entities revelant to the access control logic. For
example, 

```
[terms]
data_owner = data, usr
task_participant = task, usr
```

For the `task_access_data` request, there are three relevant domains: `data`,
`usr`, and `task`. Furthermore, two relations are required by Teaclave to make
a decision, which are

  - `data_owner` relation over (`data` X `usr`), denoting which user owns
    a piece of data.
  - `task_participant` relation over (`task` X `usr`), denoting which
    users are the participants of a joint computation task.

An instance of the database describing the two terms could be

```
data_owner data_1, usr_1
data_owner data_2, usr_1
data_owner data_2, usr_2

task_participant task_1 usr_1
task_participant task_1 usr_2
```

The facts stored in this database instance indicate that `data_1` is owned
exclusively by `usr_1`, while `data_2` is owned by `usr_1` and `usr_2`
together. The facts also indicates that `task_1` has two participants, i.e.,
`usr_1` and `usr_2`.

### matcher
The core logic used by Teaclave to resolve a request is defined as a matcher.
We define the matcher for `task_access_data` request as the following

```
[matcher]
task_access_data = data_owner(task_access_data.data, _) <= task_participant(task_access_data.task, _)
```

`data_owner(task_access_data.data, _)` and
`task_participant(task_access_data.task, _)` are term queries. The return value
of `data_owner(task_access_data.data, _)` is a subset of the `usr` domain,
where each value `u` in the fact meets the condition that

```
data_owner task_access_data.data u
```

is in the terms database. `_` is called the query wild card and `<=` is the
subset operator. Therefore, the matcher basically means that, *the request for
`task` to access `data` is approved only if all owners of `data` are have
articipated in `task`*.

## Implementation
The access control module of Teaclave is implemented as a standalone service.
Other components should send RPC requests to the service and get access control
decisions as RPC responses.

The model configuration parser and request resolution engine are written in
Python, powered by MesaPy. The access control service of Teaclave is a nice
showcase of what MesaPy is capable of.

The implementation is purely experimental at this point. The performance is not
optimized and the engine is likely not robust enough to avoid crashes while
dealing with badly shaped requests. Contributions are welcome!
