use std::env;
use bitcoincore_rpc::{Auth, Client, RpcApi, Error as RpcError};
use serde_json::{Value,json};
use bitcoin::hash_types::Txid;
use std::str::FromStr;
use bitcoin::psbt::PartiallySignedTransaction as Psbt;
use bitcoin::blockdata::transaction::TxIn as TransactionInput;
use bitcoin::blockdata::transaction::TxOut;
use bitcoin::OutPoint;
use bitcoin::Sequence;
use bitcoin::blockdata::script::Builder;
use bitcoin::TxIn;
use bitcoin::Amount;
use bitcoin::Script;
use bitcoin::Address as BitcoinAddress;
use bitcoin::network::constants::Network;
use bitcoin::address::NetworkChecked;
use bitcoin::address::NetworkUnchecked;



#[tokio::main]
async fn main() {
    let base_url = "http://127.0.0.1:18443";
    let auth = Auth::UserPass(String::from("newuser"), String::from("newpass"));
    let client: Client = Client::new(&base_url, auth.clone()).unwrap();

    // Specify the name of the wallet you want to load or create
    let wallet_name = "mynewwallet"; 

    // Create a new client object for wallet-specific RPC calls
    let wallet_url = format!("{}/wallet/{}", base_url, wallet_name);
    let wallet_client: Client = Client::new(&wallet_url, auth.clone()).unwrap();

    // Try to get a new address, if this fails, the wallet is likely not loaded
    let new_address_result: Result<String, _> = wallet_client.call("getnewaddress", &[]);
    match new_address_result {
        Ok(_) => println!("Wallet '{}' loaded.", wallet_name),
        Err(_) => {
            // Wallet could not be found, create a new one
            match client.create_wallet(wallet_name, None, None, None, None) {
                Ok(_) => println!("Wallet '{}' created.", wallet_name),
                Err(err) => {
                    println!("Error creating wallet: {}", err);
                    return;
                },
            }
        },
    }


    // After loading or creating the wallet, get a new address and its balance
    let new_address: String = wallet_client.call("getnewaddress", &[]).unwrap();

    match new_address_result {
        Ok(new_address) => {
            println!("New address: {}", new_address);
            
            let _ = wallet_client.call::<Value>("generatetoaddress", &[120.into(), new_address.clone().into()]);

            // Print the balance of the wallet
            let balance = wallet_client.get_balance(Some(0), None).unwrap();
            println!("Balance of the wallet: {}", balance);

            // Estimate the transaction fee
            let estimate: Result<Value, _> = wallet_client.call("estimatesmartfee", &[Value::from(6)]);
            let fee_per_kb = match estimate {
                Ok(estimate) => {
                    if let Some(fee) = estimate["feerate"].as_f64() {
                        fee
                    } else {
                        println!("Fee rate could not be estimated. Defaulting to 0.001 BTC/kB.");
                        0.001
                    }
                },
                Err(err) => {
                    println!("Error while estimating fee: {}", err);
                    return;
                }
            };


println!("Estimated fee per kB: {}", fee_per_kb);

// Convert fee from BTC/kB to Satoshis/Byte
let fee_per_byte = (fee_per_kb * 1e8 / 1000.0).round() as u64;
println!("Converted fee per Byte (in Satoshis): {}", fee_per_byte);


            // Set the transaction fee to 0.0001 BTC/kB
            //let _: Result<(), _> = wallet_client.call("settxfee", &[0.0001.into()]);
            let _: Result<(), _> = wallet_client.call("settxfee", &[fee_per_kb.into()]);

            let txid_result: Result<String, _> = wallet_client.call("sendtoaddress", &[new_address.into(), 1.0.into()]);
            match txid_result {
                Ok(txid) => {
                    println!("Transaction ID: {}", txid);
                    //let _ = get_transaction_details(&wallet_client, &txid).await;
        
                    // Retrieve UTXOs
                    let unspent_result: Result<Vec<serde_json::Value>, _> = wallet_client.call("listunspent", &[0.into(), 9999999.into()]);
                    if let Ok(unspent) = unspent_result {
                        if unspent.is_empty() {
                            println!("No UTXOs found.");
                            return;
                        }
        
                        // Use the first UTXO
                        let utxo = &unspent[0];
                        let utxo_amount = Amount::from_btc(utxo["amount"].as_f64().unwrap()).unwrap();
                        println!("UTXO Amount: {}", utxo_amount.to_btc());
                        let txid: Txid = Txid::from_str(utxo["txid"].as_str().unwrap()).unwrap();
                        let vout: u32 = utxo["vout"].as_u64().unwrap() as u32;
                        let utxos: Result<Vec<serde_json::Value>, _> = wallet_client.call("listunspent", &[0.into(), 9999999.into()]);

                        if let Ok(utxos) = utxos {
                            let mut utxo_exists = false;

                            for utxo in utxos {
                                if utxo["txid"].as_str().unwrap() == txid.to_string() && utxo["vout"].as_u64().unwrap() as u32 == vout {
                                    utxo_exists = true;
                                    break;
                                }
                            }

                            if utxo_exists {
                                println!("The UTXO is still unspent.");
                            } else {
                                println!("The UTXO has been spent.");
                                return;
                            }
                        } else {
                            println!("Error while checking if the UTXO is unspent: {:?}", utxos.err());
                            return;
                        }

                        let sequence: Sequence = bitcoin::Sequence(u32::from(Sequence::MAX) - 1);
                        let newnew_address: String = wallet_client.call("getnewaddress", &[]).unwrap();
                        // Specify the amount to send
                        let estimated_size = 1000; // This is an estimate. The actual size might be different.
                        let fee = (fee_per_byte as f64 * estimated_size as f64).round() as u64;
                        let fee_amount = Amount::from_sat(fee);
                        let amount_to_send = utxo_amount - fee_amount;


                        //let amount = Amount::from_btc(9.0).unwrap();
                        let amount = amount_to_send;

                        let input_obj = json!({
                            "txid": txid.to_string(),
                            "vout": vout,
                            "sequence": sequence
                        });
                        
                        let output_obj = json!({
                            newnew_address.clone(): amount.to_btc()
                        });
                        
                        // Step 1: Create and sign the transaction
let raw_transaction_result: Result<String, _> = wallet_client.call(
    "createrawtransaction", 
    &[json!([input_obj.clone()]), json!([output_obj.clone()])]
);
let signed_transaction = match raw_transaction_result {
    Ok(raw_transaction) => {
        // Sign the transaction
        let signed_transaction_result: Result<Value, _> = wallet_client.call("signrawtransactionwithwallet", &[Value::String(raw_transaction)]);
        match signed_transaction_result {
            Ok(signed_transaction) => Some(signed_transaction),
            Err(err) => {
                println!("Error while signing raw transaction: {}", err);
                return;
            }
        }
    },
    Err(err) => {
        println!("Error while creating raw transaction: {}", err);
        return;
    }
};

// Step 2: Calculate the actual size of the signed transaction
let signed_transaction_value = signed_transaction.unwrap();
let signed_hex = signed_transaction_value["hex"].as_str().unwrap();
let transaction_size = signed_hex.len() / 2; // 1 byte is 2 hex characters


// Step 3: Use this actual size to calculate the fee
let fee = (fee_per_byte as f64 * transaction_size as f64).round() as u64;
let fee_amount = Amount::from_sat(fee);
let amount_to_send = utxo_amount - fee_amount;

// Step 4: Recreate and resign the transaction with the correct fee
let input_obj = json!({
    "txid": txid.to_string(),
    "vout": vout,
    "sequence": sequence
});
let output_obj = json!({
    newnew_address.clone(): amount_to_send.to_btc()
});
let raw_transaction_result: Result<String, _> = wallet_client.call(
    "createrawtransaction", 
    &[json!([input_obj]), json!([output_obj])]
);
match raw_transaction_result {
    Ok(raw_transaction) => {
        // Sign the transaction
        let signed_transaction_result: Result<Value, _> = wallet_client.call("signrawtransactionwithwallet", &[Value::String(raw_transaction)]);
        match signed_transaction_result {
            Ok(signed_transaction) => {
                let signed_hex = signed_transaction["hex"].as_str().unwrap();
                println!("Signed Transaction: {}", signed_hex);

                // Broadcast the transaction
                let send_result: Result<String, _> = wallet_client.call("sendrawtransaction", &[Value::String(signed_hex.into())]);
                match send_result {
                    Ok(txid) => {
                        println!("Transaction broadcasted with TXID: {}", txid);
                    },
                    Err(err) => println!("Error while sending raw transaction: {}", err),
                }
            },
            Err(err) => println!("Error while signing raw transaction: {}", err),
        }
    },
    Err(err) => println!("Error while creating raw transaction: {}", err),
}

                        
                        

                    } else {
                        println!("Error while listing unspent transactions: {:?}", unspent_result.err());
                    }
                },
                Err(err) => println!("Error while sending to address: {}", err),
            }
 
        },
        Err(err) => println!("Error while getting new address: {}", err),
    }




}


async fn get_transaction_details(client: &Client, txid_str: &str) {
    // Convert the string to a bitcoin::Txid object
    let txid = bitcoin::Txid::from_str(txid_str).unwrap();

    // Get the best block hash
    let best_block_hash = client.get_best_block_hash().unwrap();
    println!("Best block hash: {}", best_block_hash);

    // Get the block
    let block = client.get_block(&best_block_hash).unwrap();
    println!("Block: {:?}", block);

    // Get the transaction
    let transaction = client.get_transaction(&txid, None).unwrap();
    println!("Transaction: {:?}", transaction);
}

