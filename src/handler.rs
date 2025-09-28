use actix_web::{HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Record {
    id: String,
    symbol: String,
    value: f64,
    above_or_below: bool,
    created: DateTime<Utc>,
    sent: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct RecordInput {
    symbol: String,
    value: f64,
    above_or_below: bool,
}

pub async fn index() -> impl Responder {
    let html = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Record Manager</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
        .form-container { margin-bottom: 20px; }
        input, button { margin: 5px; padding: 5px; }
    </style>
</head>
<body>
    <h2>Record Manager</h2>
    <div class="form-container">
        <h3>Add/Edit Record</h3>
        <form id="recordForm" action="/records" method="POST">
            <input type="hidden" name="id" id="id">
            <input type="text" name="symbol" id="symbol" placeholder="Symbol" required>
            <input type="number" name="value" id="value" placeholder="Value" step="0.01" min="0.01" required>
            <select name="above_or_below" id="above_or_below" required>
                <option value="true">Above</option>
                <option value="false">Below</option>
            </select>
            <button type="submit">Save</button>
            <button type="button" onclick="resetForm()">Clear</button>
        </form>
    </div>
    <h3>Records</h3>
    <table id="recordsTable">
        <tr>
            <th>Symbol</th>
            <th>Value</th>
            <th>Above/Below</th>
            <th>Created</th>
            <th>Sent</th>
            <th>Actions</th>
        </tr>
    </table>

    <script>
        async function fetchRecords() {
            const response = await fetch('/records');
            const records = await response.json();
            const table = document.getElementById('recordsTable');
            while (table.rows.length > 1) table.deleteRow(1);
            records.forEach(record => {
                const row = table.insertRow();
                row.insertCell().textContent = record.symbol;
                row.insertCell().textContent = record.value;
                row.insertCell().textContent = record.above_or_below ? 'Above' : 'Below';
                row.insertCell().textContent = new Date(record.created).toLocaleString();
                row.insertCell().textContent = record.sent ? new Date(record.sent).toLocaleString() : '';
                const actionsCell = row.insertCell();
                actionsCell.innerHTML = `
                    <button onclick="editRecord('${record.id}', '${record.symbol}', ${record.value}, ${record.above_or_below})">Edit</button>
                    <button onclick="deleteRecord('${record.id}')">Delete</button>
                `;
            });
        }

        function editRecord(id, symbol, value, aboveOrBelow) {
            document.getElementById('id').value = id;
            document.getElementById('symbol').value = symbol;
            document.getElementById('value').value = value;
            document.getElementById('above_or_below').value = aboveOrBelow.toString();
        }

        async function deleteRecord(id) {
            if (confirm('Are you sure you want to delete this record?')) {
                await fetch(`/records/${id}`, { method: 'DELETE' });
                fetchRecords();
            }
        }

        function resetForm() {
            document.getElementById('recordForm').reset();
            document.getElementById('id').value = '';
        }

        document.getElementById('recordForm').addEventListener('submit', async (e) => {
            e.preventDefault();
            const formData = new FormData(e.target);
            const data = {
                symbol: formData.get('symbol'),
                value: parseFloat(formData.get('value')),
                above_or_below: formData.get('above_or_below') === 'true'
            };
            const id = formData.get('id');
            const method = id ? 'PUT' : 'POST';
            const url = id ? `/records/${id}` : '/records';
            await fetch(url, {
                method: method,
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(data)
            });
            resetForm();
            fetchRecords();
        });

        window.onload = fetchRecords;
    </script>
</body>
</html>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}
