use std::cmp::min;
use std::collections::HashMap;

fn main() {
    println!("Hello, Coreum!");
    let original_balances: Vec<Balance> = [
        Balance {
            address: "account1".to_string(),
            coins: [Coin {
                denom: "denom1".to_string(),
                amount: 1000_000,
            }]
            .to_vec(),
        },
        Balance {
            address: "account2".to_string(),
            coins: [Coin {
                denom: "denom2".to_string(),
                amount: 1000_000,
            }]
            .to_vec(),
        },
    ]
    .to_vec();
    let definitions: Vec<DenomDefinition> = [
        DenomDefinition {
            denom: "denom1".to_string(),
            issuer: "issuer_account_A".to_string(),
            burn_rate: 0.08,
            commission_rate: 0.12,
        },
        DenomDefinition {
            denom: "denom2".to_string(),
            issuer: "issuer_account_B".to_string(),
            burn_rate: 1.0,
            commission_rate: 0.0,
        },
    ]
    .to_vec();
    let multi_send: MultiSend = MultiSend {
        inputs: [
            Balance {
                address: "account1".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 1000,
                }]
                .to_vec(),
            },
            Balance {
                address: "account2".to_string(),
                coins: [Coin {
                    denom: "denom2".to_string(),
                    amount: 1000,
                }]
                .to_vec(),
            },
        ]
        .to_vec(),
        outputs: [Balance {
            address: "account_recipient".to_string(),
            coins: [
                Coin {
                    denom: "denom1".to_string(),
                    amount: 1000,
                },
                Coin {
                    denom: "denom2".to_string(),
                    amount: 1000,
                },
            ]
            .to_vec(),
        }]
        .to_vec(),
    };

    let result = calculate_balance_changes(original_balances, definitions, multi_send);
    println!("{:#?}", result);
}

// A user can submit a `MultiSend` transaction (similar to bank.MultiSend in cosmos sdk) to transfer multiple
// coins (denoms) from multiple input addresses to multiple output addresses. A denom is the name or symbol
// for a coin type, e.g USDT and USDC can be considered different denoms; in cosmos ecosystem they are called
// denoms, in ethereum world they are called symbols.
// The sum of input coins and output coins must match for every transaction.
struct MultiSend {
    // inputs contain the list of accounts that want to send coins from, and how many coins from each account we want to send.
    inputs: Vec<Balance>,
    // outputs contains the list of accounts that we want to deposit coins into, and how many coins to deposit into
    // each account
    outputs: Vec<Balance>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Coin {
    pub denom: String,
    pub amount: i128,
}

impl Coin {
    fn new(denom: String) -> Coin {
        Coin { denom, amount: 0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Balance {
    address: String,
    coins: Vec<Coin>,
}

impl Balance {
    fn new(address: String) -> Balance {
        Balance {
            address,
            coins: Vec::new(),
        }
    }
}

// A Denom has a definition (`CoinDefinition`) which contains different attributes related to the denom:
#[derive(Clone)]
struct DenomDefinition {
    // the unique identifier for the token (e.g `core`, `eth`, `usdt`, etc.)
    denom: String,
    // The address that created the token
    issuer: String,
    // burn_rate is a number between 0 and 1. If it is above zero, in every transfer,
    // some additional tokens will be burnt on top of the transferred value, from the senders address.
    // The tokens to be burnt are calculated by multiplying the TransferAmount by burn rate, and
    // rounding it up to an integer value. For example if an account sends 100 token and burn_rate is
    // 0.2, then 120 (100 + 100 * 0.2) will be deducted from sender account and 100 will be deposited to the recipient
    // account (i.e 20 tokens will be burnt)
    burn_rate: f64,
    // commission_rate is exactly same as the burn_rate, but the calculated value will be transferred to the
    // issuer's account address instead of being burnt.
    commission_rate: f64,
}

// Implement `calculate_balance_changes` with the following requirements.
// - Output of the function is the balance changes that must be applied to different accounts
//   (negative means deduction, positive means addition), or an error. the error indicates that the transaction must be rejected.
// - If sum of inputs and outputs in multi_send_tx does not match the tx must be rejected(i.e return error).
// - Apply burn_rate and commission_rate as described by their definition.
// - If the sender does not have enough balances (in the original_balances) to cover the input amount on top of burn_rate and
// commission_rate, the transaction must be rejected.
// - burn_rate and commission_rate does not apply to the issuer. So to calculate the correct values you must do this for every denom:
//      - sum all the inputs coming from accounts that are not an issuer (let's call it non_issuer_input_sum)
//      - sum all the outputs going to accounts that are not an issuer (let's call it non_issuer_output_sum)
//      - total burn amount is total_burn = min(non_issuer_input_sum, non_issuer_output_sum)
//      - total_burn is distributed between all input accounts as: account_share = roundup(total_burn * input_from_account / non_issuer_input_sum)
//      - total_burn_amount = sum (account_shares) // notice that in previous step we rounded up, so we need to recalculate the total again.
//      - commission_rate is exactly the same, but we send the calculate value to issuer, and not burn.
//      - Example:
//          burn_rate: 10%
//
//          inputs:
//          60, 90
//          25 <-- issuer
//
//          outputs:
//          50
//          100 <-- issuer
//          25
//          In this case burn amount is: min(non_issuer_inputs, non_issuer_outputs) = min(75+75, 50+25) = 75
//          Expected burn: 75 * 10% = 7.5
//          And now we divide it proportionally between all input sender: first_sender_share  = 7.5 * 60 / 150  = 3
//                                                                        second_sender_share = 7.5 * 90 / 150  = 4.5
// - In README.md we have provided more examples to help you better understand the requirements.
// - Write different unit tests to cover all the edge cases, we would like to see how you structure your tests.
//   There are examples in README.md, you can convert them into tests, but you should add more cases.
fn calculate_balance_changes(
    original_balances: Vec<Balance>,
    definitions: Vec<DenomDefinition>,
    multi_send_tx: MultiSend,
) -> Result<Vec<Balance>, String> {
    let mut denoms_to_update: Vec<String> = Vec::new();
    let mut result_balances: Vec<Balance> = Vec::new();
    // let mut result_balances_map: HashMap<String, Balance> = HashMap::new();
    let mut non_issuer_input_sum: HashMap<String, u128> = HashMap::new();
    let mut non_issuer_output_sum: HashMap<String, u128> = HashMap::new();
    let mut insufficient_amount: bool = false;

    multi_send_tx
        .inputs
        .clone()
        .into_iter()
        .for_each(|input: Balance| {
            input.coins.clone().into_iter().for_each(|coin: Coin| {
                if !denoms_to_update.contains(&coin.denom) {
                    denoms_to_update.push(coin.denom.clone());
                }
                let mut update_denom = non_issuer_input_sum.get(&coin.denom);
                if update_denom.is_some() {
                    non_issuer_input_sum.insert(
                        coin.denom.clone(),
                        coin.amount as u128 + update_denom.unwrap(),
                    );
                } else {
                    non_issuer_input_sum.insert(coin.denom.clone(), coin.amount as u128);
                }
                let coin_definition: DenomDefinition = definitions
                    .iter()
                    .find(|&d| d.denom == coin.denom.clone())
                    .unwrap()
                    .clone();
                let burn = coin.amount as f64 * &coin_definition.burn_rate;
                let commission = coin.amount as f64 * &coin_definition.commission_rate;
                let total_deduct = coin.amount + burn as i128 + commission as i128;

                let balance_amount: i128 = original_balances
                    .clone()
                    .into_iter()
                    .find(|b: &Balance| {
                        println!(
                            "Debug: (b.address) {}, (input.address) {}.",
                            b.address, input.address
                        );
                        b.address == input.address
                    })
                    .unwrap_or(Balance::new(input.address.clone()))
                    .coins
                    .into_iter()
                    .find(|c: &Coin| {
                        println!("coin: {:#?}", c);
                        println!(
                            "Debug: (c.denom) {}, (coin.denom.clone()) {}.",
                            c.denom.clone(),
                            coin.denom.clone()
                        );
                        c.denom == coin.denom.clone()
                    })
                    .unwrap_or(Coin::new(coin.denom.clone()))
                    .amount
                    .clone();

                println!(
                    "Debug: (total_deduct) {}, (balance_amount) {}.",
                    total_deduct.clone(),
                    balance_amount.clone()
                );
                if total_deduct > balance_amount {
                    insufficient_amount = true;
                }
            })
        });
    if insufficient_amount == true {
        return Err("Insufficient amount in balance".to_string());
    }
    multi_send_tx.outputs.into_iter().for_each(|output| {
        result_balances.push(output.clone());
        output.coins.into_iter().for_each(|coin| {
            if !denoms_to_update.contains(&coin.denom) {
                denoms_to_update.push(coin.denom.clone());
                println!("{}", coin.denom);
            }
            let mut update_denom = non_issuer_output_sum.get(&coin.denom);
            if update_denom.is_some() {
                non_issuer_output_sum
                    .insert(coin.denom, coin.amount as u128 + update_denom.unwrap());
            } else {
                non_issuer_output_sum.insert(coin.denom, coin.amount as u128);
            }
        })
    });
    println!("result_balances: {:#?}", result_balances.clone());
    for (index, value) in denoms_to_update.into_iter().enumerate() {
        if non_issuer_input_sum.get(&value) != non_issuer_output_sum.get(&value) {
            return Err("Inputs do not match outputs".to_string());
        }
    }

    for (index, input) in multi_send_tx.inputs.into_iter().enumerate() {
        // finsih output
        let mut balance_new = Balance::new(input.address);
        input.coins.into_iter().for_each(|coin| {
            let coin_definition: DenomDefinition = definitions
                .iter()
                .find(|&d| d.denom == coin.denom.clone())
                .unwrap()
                .clone();
            // total_burn = min(non_issuer_input_sum, non_issuer_output_sum) //total burn amount
            let denom = coin.denom.clone();
            let amount = coin.amount as u128;
            let commission = (coin.amount as f64 * coin_definition.commission_rate) as i128;

            let total_burn = min(
                non_issuer_input_sum.get(&denom),
                non_issuer_output_sum.get(&denom),
            )
            .unwrap();
            // account_share = roundup(total_burn * input_from_account / non_issuer_input_sum)
            let burn = *total_burn as f64 * &coin_definition.burn_rate;
            println!(
                "burn: {}, amount: {}, non_issuer_input_sum: {}",
                burn.clone(),
                amount.clone(),
                non_issuer_input_sum.get(&denom).unwrap().clone()
            );
            let account_share = burn as u128 * amount / non_issuer_input_sum.get(&denom).unwrap();
            // total_burn_amount = sum (account_shares)

            let mut updated_coin_details = Coin::new(denom);
            println!(
                "amount: {}, account_share: {}, commit_rate: {}",
                coin.amount.clone(),
                account_share.clone(),
                commission.clone()
            );
            updated_coin_details.amount -= coin.amount + account_share as i128 + commission;

            balance_new.coins.push(updated_coin_details);

            //check if there is an issuer for this denom in result_balances
            //if issuer exist, update issuer balance, if issuer does not exist, push a new balance of issuer with updated balance
            let has_issuer = result_balances
                .iter()
                .find(|b| b.address == coin_definition.issuer);

            if commission > 0 {
                match has_issuer {
                    Some(mut b) => {
                        b.coins
                            .clone()
                            .into_iter()
                            .find(|c| c.denom == coin_definition.denom)
                            .unwrap()
                            .amount += commission;
                        let index: usize = result_balances
                            .iter()
                            .position(|b| b.address == coin_definition.issuer)
                            .unwrap();
                        result_balances[index] = b.clone();
                    }
                    None => {
                        let mut new_issuer_balance = Balance::new(coin_definition.issuer);
                        let mut coin_in_balance = Coin::new(coin_definition.denom);
                        coin_in_balance.amount += commission;
                        new_issuer_balance.coins.push(coin_in_balance);
                        result_balances.push(new_issuer_balance);
                    }
                };
            }
        });
        result_balances.push(balance_new);
        println!("result_balances: {:#?}", result_balances.clone());
    }
    Ok(result_balances)
    // Err("Inputs do not match outputs".to_string())

    // Err("Insufficient balance: {:?}".to_string(), account_address)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        //Test input values
        let original_balances: Vec<Balance> = [
            Balance {
                address: "account1".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 1000_000,
                }]
                .to_vec(),
            },
            Balance {
                address: "account2".to_string(),
                coins: [Coin {
                    denom: "denom2".to_string(),
                    amount: 1000_000,
                }]
                .to_vec(),
            },
        ]
        .to_vec();
        let definitions: Vec<DenomDefinition> = [
            DenomDefinition {
                denom: "denom1".to_string(),
                issuer: "issuer_account_A".to_string(),
                burn_rate: 0.08,
                commission_rate: 0.12,
            },
            DenomDefinition {
                denom: "denom2".to_string(),
                issuer: "issuer_account_B".to_string(),
                burn_rate: 1.0,
                commission_rate: 0.0,
            },
        ]
        .to_vec();
        let multi_send: MultiSend = MultiSend {
            inputs: [
                Balance {
                    address: "account1".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 1000,
                    }]
                    .to_vec(),
                },
                Balance {
                    address: "account2".to_string(),
                    coins: [Coin {
                        denom: "denom2".to_string(),
                        amount: 1000,
                    }]
                    .to_vec(),
                },
            ]
            .to_vec(),
            outputs: [Balance {
                address: "account_recipient".to_string(),
                coins: [
                    Coin {
                        denom: "denom1".to_string(),
                        amount: 1000,
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: 1000,
                    },
                ]
                .to_vec(),
            }]
            .to_vec(),
        };

        let result = calculate_balance_changes(original_balances, definitions, multi_send);

        // Resulting Output:
        let balance_changes: Vec<Balance> = [
            Balance {
                address: "account_recipient".to_string(),
                coins: [
                    Coin {
                        denom: "denom1".to_string(),
                        amount: 1000,
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: 1000,
                    },
                ]
                .to_vec(),
            },
            Balance {
                address: "issuer_account_A".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 120,
                }]
                .to_vec(),
            },
            Balance {
                address: "account1".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: -1200, // (1000 sent, 80 burnt, 120 send to issuer as commission)
                }]
                .to_vec(),
            },
            Balance {
                address: "account2".to_string(),
                coins: [Coin {
                    denom: "denom2".to_string(),
                    amount: -2000, // (1000 sent and 1000 burnt(burn_rate is 1))
                }]
                .to_vec(),
            },
        ]
        .to_vec();

        assert_eq!(result, Ok(balance_changes));
    }

    #[test]
    fn example_2() {
        //Test input values
        let original_balances: Vec<Balance> = [
            Balance {
                address: "account1".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 1000_000,
                }]
                .to_vec(),
            },
            Balance {
                address: "account2".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 1000_000,
                }]
                .to_vec(),
            },
        ]
        .to_vec();
        let definitions: Vec<DenomDefinition> = [DenomDefinition {
            denom: "denom1".to_string(),
            issuer: "issuer_account_A".to_string(),
            burn_rate: 0.08,
            commission_rate: 0.12,
        }]
        .to_vec();
        let multi_send: MultiSend = MultiSend {
            inputs: [
                Balance {
                    address: "account1".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 650,
                    }]
                    .to_vec(),
                },
                Balance {
                    address: "account2".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 350,
                    }]
                    .to_vec(),
                },
            ]
            .to_vec(),
            outputs: [
                Balance {
                    address: "account_recipient".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 500,
                    }]
                    .to_vec(),
                },
                Balance {
                    address: "issuer_account_A".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 500,
                    }]
                    .to_vec(),
                },
            ]
            .to_vec(),
        };

        let result = calculate_balance_changes(original_balances, definitions, multi_send);
        println!("{:#?}", result.clone());
        // Resulting Output:
        let balance_changes: Vec<Balance> = [
            Balance {
                address: "account_recipient".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 500,
                }]
                .to_vec(),
            },
            Balance {
                address: "issuer_account_A".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 560,
                }]
                .to_vec(),
            },
            Balance {
                address: "account1".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: -715, // (1000 sent, 80 burnt, 120 send to issuer as commission)
                }]
                .to_vec(),
            },
            Balance {
                address: "account2".to_string(),
                coins: [Coin {
                    denom: "denom2".to_string(),
                    amount: -385, // (1000 sent and 1000 burnt(burn_rate is 1))
                }]
                .to_vec(),
            },
        ]
        .to_vec();

        assert_eq!(result, Ok(balance_changes));
    }

    #[test]
    fn example_3() {
        //Test input values
        let original_balances: Vec<Balance> = [Balance {
            address: "account1".to_string(),
            coins: Vec::new(),
        }]
        .to_vec();
        let definitions: Vec<DenomDefinition> = [DenomDefinition {
            denom: "denom1".to_string(),
            issuer: "issuer_account_A".to_string(),
            burn_rate: 0.0,
            commission_rate: 0.0,
        }]
        .to_vec();
        let multi_send: MultiSend = MultiSend {
            inputs: [Balance {
                address: "account1".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 350,
                }]
                .to_vec(),
            }]
            .to_vec(),
            outputs: [Balance {
                address: "account_recipient".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 350,
                }]
                .to_vec(),
            }]
            .to_vec(),
        };

        let result = calculate_balance_changes(original_balances, definitions, multi_send);
        println!("{:#?}", result.clone());
        // Resulting Output:
        assert_eq!(result, Err("Insufficient amount in balance".to_string()));
    }

    #[test]
    fn example_4() {
        //Test input values
        let original_balances: Vec<Balance> = [Balance {
            address: "account1".to_string(),
            coins: [Coin {
                denom: "denom1".to_string(),
                amount: 1000_000,
            }]
            .to_vec(),
        }]
        .to_vec();
        let definitions: Vec<DenomDefinition> = [DenomDefinition {
            denom: "denom1".to_string(),
            issuer: "issuer_account_A".to_string(),
            burn_rate: 0.0,
            commission_rate: 0.0,
        }]
        .to_vec();
        let multi_send: MultiSend = MultiSend {
            inputs: [Balance {
                address: "account1".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 350,
                }]
                .to_vec(),
            }]
            .to_vec(),
            outputs: [Balance {
                address: "account_recipient".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 450,
                }]
                .to_vec(),
            }]
            .to_vec(),
        };

        let result = calculate_balance_changes(original_balances, definitions, multi_send);
        println!("{:#?}", result.clone());
        // Resulting Output:
        assert_eq!(result, Err("Inputs do not match outputs".to_string()));
    }
}
