# csv-payment-engine
A naive implementation of a payment engine that reads transactions from a CSV file, process them and outputs the final state of all accounts.

## Usage
1. Add `transactions.csv` file in the root directory of the project with the following format:
```
type,client,tx,amount
deposit,1,1,100.0
withdrawal,1,2,50.0
dispute,1,1,
resolve,1,1,
chargeback,1,1,
```
2. Run the application using the command:
```
cargo run -- transactions.csv > accounts.csv
```
3. Check the output in `accounts.csv` file with the following format:
```
client,available,held,total,locked
1,0.00,0.00,0.00,false
```