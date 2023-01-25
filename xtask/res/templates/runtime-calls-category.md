There are **{{ calls_counter }}** {{ category_name }} calls from **{{ pallets | length }}** pallets.

{% for pallet in pallets -%}
### {{ pallet.name }} - {{ pallet.index }}

{% for call in pallet.calls -%}
#### {{ call.name }} - {{ call.index }}

<details><summary><code>{{ call.name }}(
    {%- for param in call.params -%}
    {{ param.name }}{% if loop.last != true %}, {% endif %} 
    {%- endfor -%}
    )</code></summary>

```rust
{% for param in call.params -%}
{{ param.name }}: {{ param.type_name }}
{% endfor -%}
```
</details>

{# replace markdown sytax in documentation breaking the final result #}
{{ call.documentation | replace(from="# WARNING:", to="WARNING:") }}

{% endfor -%}
{% endfor -%}