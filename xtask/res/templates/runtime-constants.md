# Runtime Constant

There are **{{ constant_counter }}** constants from **{{ pallets | length }}** pallets.

<ul>
{% for pallet in pallets %}
<li>{{ pallet.name }} - {{ pallet.index }}
<ul>
{% for constant in pallet.constants %}
<li>
<details>
<summary>
<code>{{ constant.name }}</code>
</summary>
{{ constant.documentation }}

```rust
value: {{ constant.type_value }}({{ constant.value }})
```

</details>
</li>
{% endfor %}
</ul>
</li>
{% endfor %}
</ul>
