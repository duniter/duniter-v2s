There are **{{ calls_counter }}** {{ category_name }} calls from **{{ pallets | length }}** pallets.

{% for pallet in pallets -%}

## {{ pallet.name }} - {{ pallet.index }}

{% for call in pallet.calls -%}

### {{ call.name }} - {{ call.index }}

<details><summary><code>{{ call.name }}(
    {%- for param in call.params -%}
    {{ param.name }}{% if loop.last != true %}, {% endif %}
    {%- endfor -%}
    )</code></summary>
{% if call.weight == -1 %}
No weight available.
{% else %}
Taking {{ call.weight }} % of a block.
{% endif %}
```rust
{% for param in call.params -%}
{{ param.name }}: {{ param.type_name }}
{% endfor -%}
```
</details>

{# lower heading title to integrate into document hierarchy #}
{# with a maximum to 6 #}
{{ call.documentation
| replace(from="# ", to="## ")
| replace(from="# ", to="## ")
| replace(from="# ", to="## ")
| replace(from="# ", to="## ")
| replace(from="####### ", to="###### ")
}}

{% endfor -%}
{% endfor -%}
