# Coreum Rust Task

Thanks for taking the time to complete this task. The task is a real world problem,
trying to see how you translate business requirements into readable, maintainable code.

A boilerplate is provided in `src/main.rs` to guide you through the development.
Take a look at comments to understand what you need to do.
Feel free to define new types if needed.

# What You Need To Do

Implement `calculate_balance_changes` with the requirements mentioned in the comment of the function.

## Examples

Here is an example that with the given input, the provided output must be the output of the function.

Example 1 (No issuer on sender or receiver)

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
                    "denom1": 120
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

Example 2 (issuer exists on sender/receiver)

```
original_balances [
    "account1" = {
        "denom1":1000_000
    }
    "account2" = {
        "denom1":1000_000
    }
]

definitions = [
     {
        denom: "denom1",
        issuer: "issuer_account_A",
        burn_rate: 0.08,
        commission_rate: 0.12,
    }
]

multi_send = {
    inputs = [
        {
            address: "account1"
            coins: [
                {
                    "denom1": 650
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom1": 350
                }
            ]
        }

    ]

    outputs = [
        {
            address: "account_recipient"
            coins: [
                {
                    "denom1": 500
                }
            ]
        }
        {
            address: "issuer_account_A"
            coins: [
                {
                    "denom1": 500
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
                    "denom1": 500
                }
            ]
        }
        {
            address: "issuer_account_A"
            coins: [
                {
                    "denom1": 560 // 500 + 60 (sum of commissions)
                }
            ]
        }
        {
            address: "account1"
            coins: [
                {
                    "denom1": -715  // 650 sent, 26 burnt (650 * 500 / 1000 * 0.08), 39 send to issuer as commission (650 * 500 / 1000 * 0.12)
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom1": -385  // 350 sent, 14 burnt (350 * 500 / 1000 * 0.08), 21 send to issuer as commission (350 * 500 / 1000 * 0.12)
                }
            ]
        }
]
```

Example 3 (not enough balance)

```
original_balances [
    "account1" = {
    }
]

definitions = [
     {
        denom: "denom1",
        issuer: "issuer_account_A",
        burn_rate: 0,
        commission_rate: 0,
    }
]

multi_send = {
    inputs = [
        {
            address: "account1"
            coins: [
                {
                    "denom1": 350
                }
            ]
        }
    ]

    outputs = [
        {
            address: "account_recipient"
            coins: [
                {
                    "denom1": 350
                }
            ]
        }
    ]
}

the output of the function will be:
ERROR // notice that account1 does not have enough balance for denom2

```

Example 4 (input output mismatch)

```
original_balances [
    "account1" = {
        "denom1":1000_000
    }
]

definitions = [
     {
        denom: "denom1",
        issuer: "issuer_account_A",
        burn_rate: 0,
        commission_rate: 0,
    }
]

multi_send = {
    inputs = [
        {
            address: "account1"
            coins: [
                {
                    "denom1": 350
                }
            ]
        }
    ]

    outputs = [
        {
            address: "account_recipient"
            coins: [
                {
                    "denom1": 450
                }
            ]
        }
    ]
}

the output of the function will be:
ERROR // notice that input and output does not match

```

Example 5 (demonstrate rounding up)

```
original_balances [
    "account1" = {
        "denom1":1000
    }
    "account2" = {
        "denom1":1000
    }
]

definitions = [
     {
        denom: "denom1",
        issuer: "issuer_account_A",
        burn_rate: 0.01,
        commission_rate: 0.01,
    }
]

multi_send = {
    inputs = [
        {
            address: "account1"
            coins: [
                {
                    "denom1": 1
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom1": 1
                }
            ]
        }

    ]

    outputs = [
        {
            address: "account_recipient"
            coins: [
                {
                    "denom1": 2
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
                    "denom1": 2
                }
            ]
        }
        {
            address: "issuer_account_A"
            coins: [
                {
                    "denom1": 2
                }
            ]
        }
        {
            address: "account1"
            coins: [
                {
                    "denom1": -3  // 1 sent, 1 burnt , 1 send to issuer as commission (1 = roundup(2 * 0.01))
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom1": -3  // 1 sent, 1 burnt , 1 send to issuer as commission
                }
            ]
        }
]
```

Additional Tests::

Example 6 (sender is also a receiver)

```
original_balances [
    "account1" = {
        "denom1":10_000
    }
    "account2" = {
        "denom1":10_000
    }
    "account3" = {
        "denom1":10_000
    }
]

definitions = [
     {
        denom: "denom1",
        issuer: "issuer_account_A",
        burn_rate: 0.5,
        commission_rate: 0.5,
    }
]

multi_send = {
    inputs = [
        {
            address: "account1"
            coins: [
                {
                    "denom1": 1600
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom1": 1000
                }
            ]
        }
    ]

    outputs = [
        {
            address: "account3"
            coins: [
                {
                    "denom1": 2000
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom1": 600
                }
            ]
        }
    ]
}

the output of the function will be:

balance_changes = [
        {
            address: "account3"
            coins: [
                {
                    "denom1": 2000
                }
            ]
        }
        {
            address: "account_recipient"
            coins: [
                {
                    "denom1": 1300
                }
            ]
        }
        {
            address: "account1"
            coins: [
                {
                    "denom1": -3200, // 1600 sent, 800 burnt , 800 send to issuer as commission
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom1": -1400, // 1000 sent, 500 burnt , 500 send to issuer as commission and 600 received from account1
                }
            ]
        }
]
```

Example 7 (complex testcase: issuer and a sender both as receiver)

```
original_balances [
    "account1" = {
        "denom1":10_000
        "denom2":10_000
    }
    "account2" = {
        "denom1":10_000
        "denom2":10_000
    }
]

definitions = [
     {
        denom: "denom1",
        issuer: "issuer_account_A",
        burn_rate: 0.1,
        commission_rate: 0.05,
    }
    {
        denom: "denom1",
        issuer: "issuer_account_A",
        burn_rate: 0.15,
        commission_rate: 0.1,
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
                {
                    "denom2": 1000
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom1": 1000
                }
                {
                    "denom2": 2000
                }
            ]
        }
    ]

    outputs = [
        {
            address: "account_recipient"
            coins: [
                {
                    "denom1": 2000
                }
                {
                    "denom2": 2000
                }
            ]
        }
        {
            address: "account_recipient_2"
            coins: [
                {
                    "denom2": 250
                }
            ]
        }
        {
            address: "account_recipient_B"
            coins: [
                {
                    "denom2": 500
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom2": 250
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
                    "denom1": 2000
                    "denom2": 2000
                }
            ]
        }
        {
            address: "account_recipient_2"
            coins: [
                {
                    "denom1": 250
                }
            ]
        }
        {
            address: "issuer_account_A"
            coins: [
                {
                    "denom1": 100
                }
            ]
        }
        {
            address: "issuer_account_B"
            coins: [
                {
                    "denom2": 751, // 500 sent, 84 commission from account1 and 167 commission from account2
                }
            ]
        }
        {
            address: "account1"
            coins: [
                {
                    "denom1": -1150, // 1000 sent, 100 burnt , 50 send to issuer as commission
                }
                {
                    "denom2": -1209, // 1000 sent, 125 burnt , 84 send to issuer as commission
                }
            ]
        }
        {
            address: "account2"
            coins: [
                {
                    "denom2": -2167, // 2000 sent, 250 burnt, 167 send to issuer as commission and 250 received from account1
                }
                {
                    "denom1": -1150, // 1000 sent, 100 burnt , 50 send to issuer as commission
                }
            ]
        }
]
```
