# Coreum Rust Task

Thanks for taking the time to complete this task. The task is a real world problem, 
trying to see how you translate business requirements into readable, maintainable code.

A boilerplate is provided in main.rs to guide you through the development. Feel free to define new 
types if needed. Read the comments of the provided types to understand their meaning and how they interact.

# What You Need To Do
Implement `calculate_balance_changes` with the requirements mentioned in the comment of the function.

## Example
Here is an example that with the given input, the provided output must be the output of the function.
```
original_balances [
    "account1" = {
        "denom1":1000_000
    }
    "account2": {
       "denom2": 1000_000
    }
]

definitions = [
     {
        denom: "denom1",
        issuer: "issuer_account_A",
        burn_rate: 0.08,
        commission_rate: 0.12,
    }
    {
        denom: "denom2",
        issuer: "issuer_account_B",
        burn_rate: 1,
        commission_rate: 0,
    }
]

multi_send = {
    inputs = [
        {
            address: "account1"
            coins: [
                {
                    "denom1": 1000
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom2": 1000
                }
            ]
        }
    ]

    outputs = [
        {
            address: "account_recipient"
            coins: [
                {
                    "denom1": 1000
                    "denom2": 1000
                }
            ]
        }
    ]
}

the output of the function will be:

balance_changes = [
        {
            address: "account_recipient"
            coins: [
                {
                    "denom1": 1000
                    "denom2": 1000
                }
            ]
        }
        {
            address: "issuer_account_A"
            coins: [
                {
                    "denom1": 80
                }
            ]
        }
        {
            address: "account1"
            coins: [
                {
                    "denom1": -1200  // (1000 sent, 80 burnt, 120 send to issuer as commission)
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom2": -2000 // (1000 sent and 1000 burnt(burn_rate is 1)) 
                }
            ]
        }
]
```


