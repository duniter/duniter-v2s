# Runtime errors

There are **{{error_counter}}** errors from **{{ pallets | length }}** pallets.

<ul>
{% for pallet in pallets -%}
<li>{{ pallet.name }} - {{ pallet.index }}
<ul>
{% for error in pallet.errors -%}
<li>
<details>
<summary>
<code>{{ error.name }}</code> - {{ error.index }}</summary>
{{ error.documentation }}
</details>
</li>
{% endfor -%}
</ul>
</li>
{% endfor -%}
</ul>