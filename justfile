# set the shell to nu by default, change it to whatever you have:
set shell := ["nu", "-c"]

# starts an in-memory instance of the database
db:
  surreal start --log debug --user root --pass root memory

# open a SQL session for debugging SQL queries
sql:
  surreal sql -c http://localhost:8000 --ns test --db test -p root -u root

test:
  cargo test
