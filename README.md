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

## Design desicions
- CSV file reads are done in a streaming fashion to handle large files without consuming too much memory,
but the output is generated in-memory to ensure that the final state of all accounts is accurate and consistent.
- If one of the transactions is invalid, the engine will skip it and continue processing the rest of the transactions, but if CSV file is malformed, the engine will stop and return an error.
- The engine will ignore any errors that occur during the processing of transactions, such as insufficient funds for a withdrawal or a dispute on a non-existent transaction, and will continue processing the rest of the transactions.

## Tests
To run the tests, use the command:
```
cargo test
```

## AI usage
Both ChatGPT and Copilot were used to assist in the development of this example for fast development.