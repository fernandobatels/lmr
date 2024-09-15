# lmr - Lightweight Email Report Tool

**lmr** is a simple and fast tool for generating reports from database queries and sending them via email. It streamlines data extraction, formatting, and delivery to your inbox or terminal (stdout).

Create reports with tables and charts. Export them as HTML, plain text, or Markdown, and send directly to email or display in the terminal.

Whether for quick insights or scheduled reporting, **lmr** offers an efficient and easy-to-use solution.

<p float="left">
  <img src="https://github.com/fernandobatels/lmr/blob/main/samples/bar.png" width="32%" />
  <img src="https://github.com/fernandobatels/lmr/blob/main/samples/pizza.png" width="32%" />
  <img src="https://github.com/fernandobatels/lmr/blob/main/samples/line.png" width="32%" />
</p>
<p float="right">
  <img src="https://github.com/fernandobatels/lmr/blob/main/samples/table.png" />
</p>

### How to configure

You just need an yml file that provide your db connection string, smtp server and your querys:

```yaml
title: My Project Report

send:
    stdout: false # true
    format: Html # Markdown, Txt
    mail: # Optional
        host: ...
        port: 587
        to: ....
        from: ....
        user: ...
        pass: ...

source:
    conn: "postgresql://...."
    kind: Postgres # Sqlite

querys:
    - title: Costumers by state
      sql: "select country, state, count(1) as qt from customers group by 1, 2 limit 10"
      fields:
          - field: Country
            title: Country
            kind: String
          - field: State
            title: State
            kind: String
          - field: qt
            title: Número
            kind: Integer
    - title: Top 5 movie categories
      sql: >
          select c.name, count(1) as qt
            from film f
            join film_category fc on f.film_id = fc.film_id
            join category c on fc.category_id = c.category_id
          group by 1
          order by 2 desc
             limit 5
      fields:
          - field: name
            title: Category
            kind: String
          - field: qt
            title: Quantity
            kind: Integer
      chart: # Optional
          kind: Bar # Line, Pizza
          keys_by: name
          series: # Or series_by
            - qt
```


### How to install

Build and install directly on your server:
```bash
git clone https://github.com/fernandobatels/lmr
cd lmr
cargo install --path .
```

Or build locally and copy only the bin to your server:

```bash
git clone https://github.com/fernandobatels/lmr
cd lmr
cargo build --release
chmod +x target/release/lmr
scp target/release/lmr user@host:/usr/local/bin/
```

### How to schedule a report

Use the crontab of your server:

```
0 5 * * * lmr myproject.yml -q
```

#### Supported databases:
- SQLite
- PostgreSQL
 - (Coming soon) MySQL
 - (Coming soon) Firebird
 - (Coming soon) MSSQ

#### Supported components:
- Table
- Pie chart
- Bar chart
- Line chart
