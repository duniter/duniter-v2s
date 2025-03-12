# Runtime Storage

There are **{{ storage_counter }}** storages from **{{ pallets | length }}** pallets.

<ul>
{% for pallet in pallets %}
<li>{{ pallet.name }} - {{ pallet.index }}
<ul>
{% for storage in pallet.storages %}
<li>
<details>
<summary>
<code>{{ storage.name }}</code>
</summary>
{{ storage.documentation }}

{% if storage.type_key or storage.type_value %}```rust
{% if storage.type_key %}key: {{ storage.type_key }}
{% endif %}{% if storage.type_value %}value: {{ storage.type_value }}{% endif %}
```{% endif %}

</details>
</li>
{% endfor %}
</ul>
</li>
{% endfor %}
</ul>
