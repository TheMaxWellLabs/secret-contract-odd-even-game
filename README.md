This is a secret contract, which could be use to query the counter is even or odd.

```
secretcli tx compute execute $CONTRACT "{\"increment\": {}}" --from xx
secretcli tx compute execute $CONTRACT "{\"decrement\": {}}" --from xx
secretcli tx compute execute $CONTRACT "{\"query-even-odd\": {}}" --from xx  # need to test
```
