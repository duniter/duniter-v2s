# Runtime events

There are **{{event_counter}}** events from **{{ pallets | length }}** pallets.

<ul>
{% for pallet in pallets -%}
<li>{{ pallet.name }} - {{ pallet.index }}
<ul>
{% for event in pallet.events -%}
<li>
<details>
<summary>
<code>{{ event.name }}(
{%- for param in event.params -%}
{{ param.name }}{% if loop.last != true %}, {% endif %} 
{%- endfor -%}
)</code> - {{ event.index }}</summary>
{{ event.documentation }}

```rust
{% for param in event.params -%}
{{ param.name }}: {{ param.type_name }}
{%- else -%}no args
{% endfor -%}
```

</details>
</li>
{% endfor -%}
</ul>
</li>
{% endfor -%}
</ul>