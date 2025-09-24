## `payment_engine`
Processes transactions from a CSV, returning a CSV report of the resulting accounts.

### Usage
#### Create Sample Data
```sh
cargo run --bin create_random_csv > transactions.csv 
```
#### Process Transactions
```sh
cargo run -- transactions.csv 
```
