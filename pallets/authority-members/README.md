# Duniter authority members pallet

In a permissioned network, we have to define the set of authorities, and among these authorities, the ones taking part in the next session. That's what authority members pallet does. In practice:

- it manages a `Members` set with some custom rules
- it implements the `SessionManager` trait from the session frame pallet

## Entering the set of authorities

To become part of Duniter authorities, one has to complete these steps:

1. become member of the main web of trust
1. request membership to the smith sub wot
1. get enough certs to get smith membership
1. claim membership to the set of authorities

Then one can "go online" and "go offline" to enter or leave two sessions after.

## Staying in the set of authorities

If a smith is offline more than `MaxOfflineSessions`, he leaves the set of authorities.