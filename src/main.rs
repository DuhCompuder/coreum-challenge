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
    let mut input_sum: HashMap<String, u128> = HashMap::new();
    let mut output_sum: HashMap<String, u128> = HashMap::new();
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
                let update_input_sum = input_sum.get(&coin.denom);
                if update_input_sum.is_some() {
                    input_sum.insert(
                        coin.denom.clone(),
                        coin.amount as u128 + update_input_sum.unwrap(),
                    );
                } else {
                    input_sum.insert(coin.denom.clone(), coin.amount as u128);
                }
                let update_denom = non_issuer_input_sum.get(&coin.denom);
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
            let update_non_issuer_denom = non_issuer_output_sum.get(&coin.denom);
            println!(
                "check if is issuer: {}",
                output.address.contains("issuer_account")
            );
            println!(
                "check if is issuer coin info before: {:?}",
                non_issuer_output_sum.get(&coin.denom)
            );
            if update_non_issuer_denom.is_some() {
                if !output.address.contains("issuer_account") {
                    non_issuer_output_sum.insert(
                        coin.denom.clone(),
                        coin.amount as u128 + update_non_issuer_denom.unwrap(),
                    );
                }
            } else {
                if !output.address.contains("issuer_account") {
                    non_issuer_output_sum.insert(coin.denom.clone(), coin.amount as u128);
                } else {
                    non_issuer_output_sum.insert(coin.denom.clone(), 0);
                }
            }
            let update_denom = output_sum.get(&coin.denom);

            if update_denom.is_some() {
                output_sum.insert(
                    coin.denom.clone(),
                    coin.amount as u128 + update_denom.unwrap(),
                );
            } else {
                output_sum.insert(coin.denom.clone(), coin.amount as u128);
            }
            println!(
                "check if is issuer coin info: {:?}",
                non_issuer_output_sum.get(&coin.denom)
            );
        })
    });
    println!("result_balances: {:#?}", result_balances.clone());
    for (_index, value) in denoms_to_update.into_iter().enumerate() {
        println!(
            "input_sum.get(&value) vs. output_sum.get(&value): {:#?}, {:#?}",
            input_sum.get(&value),
            output_sum.get(&value)
        );
        if input_sum.get(&value) != output_sum.get(&value) {
            return Err("Inputs do not match outputs".to_string());
        }
    }

    for (_index, input) in multi_send_tx.inputs.into_iter().enumerate() {
        // finsih output
        let mut balance_new = Balance::new(input.address.clone());
        let existing_balance = result_balances
            .iter()
            .position(|b| b.address == input.address);
        if let Some(balance_index) = existing_balance {
            balance_new = result_balances[balance_index].clone();
            result_balances.remove(balance_index);
        }
        input.coins.into_iter().for_each(|coin| {
            let coin_definition: DenomDefinition = definitions
                .iter()
                .find(|&d| d.denom == coin.denom.clone())
                .unwrap()
                .clone();
            // total_burn = min(non_issuer_input_sum, non_issuer_output_sum) //total burn amount
            let denom = coin.denom.clone();
            let amount = coin.amount as u128;
            let total_burn_commission_min = min(
                non_issuer_input_sum.get(&denom),
                non_issuer_output_sum.get(&denom),
            )
            .unwrap();

            let commission =
                (*total_burn_commission_min as f64 * coin_definition.commission_rate).ceil();

            // account_share = roundup(total_burn * input_from_account / non_issuer_input_sum)
            let burn = *total_burn_commission_min as f64 * &coin_definition.burn_rate;
            println!(
                "burn: {}, amount: {}, non_issuer_input_sum: {}",
                burn.clone(),
                amount.clone(),
                non_issuer_input_sum.get(&denom).unwrap().clone()
            );
            let burn_account_share = (burn * amount as f64
                / *non_issuer_input_sum.get(&denom).unwrap() as f64)
                .ceil() as i128;
            let commission_account_share = (commission * amount as f64
                / *non_issuer_input_sum.get(&denom).unwrap() as f64)
                .ceil() as i128;
            // total_burn_amount = sum (account_shares)

            let mut updated_coin_details = Coin::new(denom);
            let existing_coin = balance_new.coins.iter().position(|c| c.denom == coin.denom);
            if let Some(existing_coin_index) = existing_coin {
                updated_coin_details = balance_new.coins[existing_coin_index].clone();
            }
            println!(
                "amount: {}, burn_account_share: {}, commit_rate: {}",
                coin.amount.clone(),
                burn_account_share.clone(),
                commission_account_share.clone()
            );
            updated_coin_details.amount -=
                coin.amount + burn_account_share as i128 + commission_account_share;

            if let Some(existing_coin_index) = existing_coin {
                balance_new.coins[existing_coin_index] = updated_coin_details;
            } else {
                balance_new.coins.push(updated_coin_details);
            }

            //check if there is an issuer for this denom in result_balances
            //if issuer exist, update issuer balance, if issuer does not exist, push a new balance of issuer with updated balance
            let has_issuer = result_balances
                .iter()
                .find(|b| b.address == coin_definition.issuer);

            if commission_account_share > 0 {
                match has_issuer {
                    Some(b) => {
                        println!("updated_coin_details amount: {:?}", b);
                        let coin_index = b
                            .coins
                            .clone()
                            .into_iter()
                            .position(|c| c.denom == coin_definition.denom)
                            .unwrap();

                        println!("updated_coin_details amount after update: {:?}", b);
                        let balance_index: usize = result_balances
                            .iter()
                            .position(|b| b.address == coin_definition.issuer)
                            .unwrap();
                        result_balances[balance_index].coins[coin_index].amount +=
                            commission_account_share;
                    }
                    None => {
                        let mut new_issuer_balance = Balance::new(coin_definition.issuer);
                        let mut coin_in_balance = Coin::new(coin_definition.denom);
                        coin_in_balance.amount += commission_account_share;
                        new_issuer_balance.coins.push(coin_in_balance);
                        result_balances.push(new_issuer_balance);
                    }
                };
            }
        });
        // let balance_exists = result_balances
        //     .iter()
        //     .position(|b| b.address == balance_new.address);
        // match balance_exists {
        //     Some(index) => {
        //         let coin_index = result_balances[index].coins.iter().position(|c| c.denom ==)
        //         result_balances[index].coins = balance_new;
        //     },
        //     None => result_balances.push(balance_new)
        // }
        result_balances.push(balance_new);
        // result_balances.push(balance_new);
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
                    denom: "denom1".to_string(),
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
    #[test]
    fn example_5() {
        //Test input values
        let original_balances: Vec<Balance> = [
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
                    denom: "denom1".to_string(),
                    amount: 1000,
                }]
                .to_vec(),
            },
        ]
        .to_vec();
        let definitions: Vec<DenomDefinition> = [DenomDefinition {
            denom: "denom1".to_string(),
            issuer: "issuer_account_A".to_string(),
            burn_rate: 0.01,
            commission_rate: 0.01,
        }]
        .to_vec();
        let multi_send: MultiSend = MultiSend {
            inputs: [
                Balance {
                    address: "account1".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 1,
                    }]
                    .to_vec(),
                },
                Balance {
                    address: "account2".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 1,
                    }]
                    .to_vec(),
                },
            ]
            .to_vec(),
            outputs: [Balance {
                address: "account_recipient".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 2,
                }]
                .to_vec(),
            }]
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
                    amount: 2,
                }]
                .to_vec(),
            },
            Balance {
                address: "issuer_account_A".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 2,
                }]
                .to_vec(),
            },
            Balance {
                address: "account1".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: -3, // 1 sent, 1 burnt , 1 send to issuer as commission (1 = roundup(2 * 0.01))
                }]
                .to_vec(),
            },
            Balance {
                address: "account2".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: -3, // 1 sent, 1 burnt , 1 send to issuer as commission
                }]
                .to_vec(),
            },
        ]
        .to_vec();

        assert_eq!(result, Ok(balance_changes));
    }
    #[test]
    fn example_6_sender_is_also_a_receiver() {
        //Test input values
        let original_balances: Vec<Balance> = [
            Balance {
                address: "account1".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 10_000,
                }]
                .to_vec(),
            },
            Balance {
                address: "account2".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 10_000,
                }]
                .to_vec(),
            },
            Balance {
                address: "account3".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 10_000,
                }]
                .to_vec(),
            },
        ]
        .to_vec();
        let definitions: Vec<DenomDefinition> = [DenomDefinition {
            denom: "denom1".to_string(),
            issuer: "issuer_account_A".to_string(),
            burn_rate: 0.5,
            commission_rate: 0.5,
        }]
        .to_vec();
        let multi_send: MultiSend = MultiSend {
            inputs: [
                Balance {
                    address: "account1".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 1600,
                    }]
                    .to_vec(),
                },
                Balance {
                    address: "account2".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 1000,
                    }]
                    .to_vec(),
                },
            ]
            .to_vec(),
            outputs: [
                Balance {
                    address: "account3".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 2000,
                    }]
                    .to_vec(),
                },
                Balance {
                    address: "account2".to_string(),
                    coins: [Coin {
                        denom: "denom1".to_string(),
                        amount: 600,
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
                address: "account3".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 2000,
                }]
                .to_vec(),
            },
            Balance {
                address: "issuer_account_A".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 1300,
                }]
                .to_vec(),
            },
            Balance {
                address: "account1".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: -3200, // 1600 sent, 800 burnt , 800 send to issuer as commission
                }]
                .to_vec(),
            },
            Balance {
                address: "account2".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: -1400, // 1000 sent, 500 burnt , 500 send to issuer as commission and 600 received from account1
                }]
                .to_vec(),
            },
        ]
        .to_vec();

        assert_eq!(result, Ok(balance_changes));
    }
    #[test]
    fn example_7_issuer_and_a_sender_both_as_receiver() {
        //Test input values
        let original_balances: Vec<Balance> = [
            Balance {
                address: "account1".to_string(),
                coins: [
                    Coin {
                        denom: "denom1".to_string(),
                        amount: 10_000,
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: 10_000,
                    },
                ]
                .to_vec(),
            },
            Balance {
                address: "account2".to_string(),
                coins: [
                    Coin {
                        denom: "denom1".to_string(),
                        amount: 10_000,
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: 10_000,
                    },
                ]
                .to_vec(),
            },
        ]
        .to_vec();
        let definitions: Vec<DenomDefinition> = [
            DenomDefinition {
                denom: "denom1".to_string(),
                issuer: "issuer_account_A".to_string(),
                burn_rate: 0.1,
                commission_rate: 0.05,
            },
            DenomDefinition {
                denom: "denom2".to_string(),
                issuer: "issuer_account_B".to_string(),
                burn_rate: 0.15,
                commission_rate: 0.1,
            },
        ]
        .to_vec();
        let multi_send: MultiSend = MultiSend {
            inputs: [
                Balance {
                    address: "account1".to_string(),
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
                    address: "account2".to_string(),
                    coins: [
                        Coin {
                            denom: "denom1".to_string(),
                            amount: 1000,
                        },
                        Coin {
                            denom: "denom2".to_string(),
                            amount: 2000,
                        },
                    ]
                    .to_vec(),
                },
            ]
            .to_vec(),
            outputs: [
                Balance {
                    address: "account_recipient".to_string(),
                    coins: [
                        Coin {
                            denom: "denom1".to_string(),
                            amount: 2000,
                        },
                        Coin {
                            denom: "denom2".to_string(),
                            amount: 2000,
                        },
                    ]
                    .to_vec(),
                },
                Balance {
                    address: "account_recipient_2".to_string(),
                    coins: [Coin {
                        denom: "denom2".to_string(),
                        amount: 250,
                    }]
                    .to_vec(),
                },
                Balance {
                    address: "issuer_account_B".to_string(),
                    coins: [Coin {
                        denom: "denom2".to_string(),
                        amount: 500,
                    }]
                    .to_vec(),
                },
                Balance {
                    address: "account2".to_string(),
                    coins: [Coin {
                        denom: "denom2".to_string(),
                        amount: 250,
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
                coins: [
                    Coin {
                        denom: "denom1".to_string(),
                        amount: 2000,
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: 2000,
                    },
                ]
                .to_vec(),
            },
            Balance {
                address: "account_recipient_2".to_string(),
                coins: [Coin {
                    denom: "denom2".to_string(),
                    amount: 250,
                }]
                .to_vec(),
            },
            Balance {
                address: "issuer_account_B".to_string(),
                coins: [Coin {
                    denom: "denom2".to_string(),
                    amount: 751, // 500 sent, 84 commission from account1 and 167 commission from account2
                }]
                .to_vec(),
            },
            Balance {
                address: "issuer_account_A".to_string(),
                coins: [Coin {
                    denom: "denom1".to_string(),
                    amount: 100,
                }]
                .to_vec(),
            },
            Balance {
                address: "account1".to_string(),
                coins: [
                    Coin {
                        denom: "denom1".to_string(),
                        amount: -1150, // 1000 sent, 100 burnt , 50 send to issuer as commission
                    },
                    Coin {
                        denom: "denom2".to_string(),
                        amount: -1209, // 1000 sent, 125 burnt , 84 send to issuer as commission
                    },
                ]
                .to_vec(),
            },
            Balance {
                address: "account2".to_string(),
                coins: [
                    Coin {
                        denom: "denom2".to_string(),
                        amount: -2167, // 2000 sent, 250 burnt, 167 send to issuer as commission and 250 received from account1
                    },
                    Coin {
                        denom: "denom1".to_string(),
                        amount: -1150, // 1000 sent, 100 burnt , 50 send to issuer as commission
                    },
                ]
                .to_vec(),
            },
        ]
        .to_vec();

        assert_eq!(result, Ok(balance_changes));
    }
}
