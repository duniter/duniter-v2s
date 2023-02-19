# Duniter Web of Trust pallet

Duniter WoT is at the core of its identity system and is a big improvement compared to PGP WoT. It is a dynamic directed graph whose nodes are [identities](../identity/) and edges [certifications](../certification/).

There are two instances:

- the main WoT, for every human
- the smith sub-WoT, for authorities

It has both static and dynamic rules, controlling the condition to join and remain [member](../membership/).

- static rules
    - minimum number of received certifications (min indegree)
    - maximum number of emited certifications (max outdegree)
    - distance criterion (see distance pallet)
- dynamic rules
    - time interval between two certifications
    - certification duration (see certification pallet)
    - membership renewal (see membership pallet)

This pallet's main role is to check the Web of Trust rules.