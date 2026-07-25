{# assets/markdown.md #}

📊 Summary
===========================================================================

| Metric | Count / Details |
| :--- | :--- |
| **Total Tasks** | **{{ summary.total }}** |
| **Completed** | {{ summary.completed }} ✅ |
| **In Progress** | {{ summary.in_progress }} 🏃 |
| **Pending** | {{ summary.pending }} ⏳ |
| **Canceled** | {{ summary.canceled }} 🚫 |

---

{% for task in tasks %}
{{ task.title }}
===========================================================================

| Item | Details |
| :--- | :--- |
| **Project** | **{{ task.project }}** |
| **Priority** | {{ task.priority_text }} |
| **Status** | {{ task.status_text }} |
| **Timeline** | `{{ task.start_date }} ~ {{ task.due_date }}` |
| **Time Spent** | {{ "%.1f" | format(task.time_spent) }} hrs |
| **Progress** | {{ task.progress | int }}% |

> {{ task.detail | safe }}

---
{% endfor %}

*Powered by [Task Fighter](https://github.com/MoriokaReimen/Task-Fighter)*
