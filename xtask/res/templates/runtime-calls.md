# Runtime calls

Calls are categorized according to the dispatch origin they require:

1. **User calls**: the dispatch origin for this kind of call must be signed by
   the transactor. This is the only call category that can be submitted with an extrinsic.
1. **Root calls**: This kind of call requires a special origin that can only be invoked
   through on-chain governance mechanisms.
1. **Inherent calls**: This kind of call is invoked by the author of the block itself
   (usually automatically by the node).
1. **Disabled calls**: These calls can not be called directly, they are reserved for internal use by other runtime calls.

We only document user calls below.

{% set pallets = user_calls_pallets -%}
{% set calls_counter = user_calls_counter -%}
{% set category_name = "user" -%}
{% include "runtime-calls-category.md" %}
