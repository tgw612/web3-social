use crate::models::asset::{Asset, TokenBalance, TokenPrice, TransactionVerification, NFT};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature, transaction::Transaction};
use std::env;
use std::str::FromStr;

// 原生SOL精度
const SOL_DECIMALS: u8 = 9;

// 获取Solana RPC客户端
pub fn get_solana_client() -> Result<RpcClient, String> {
    let rpc_url = env::var("SOLANA_RPC_URL")
        .map_err(|_| "SOLANA_RPC_URL environment variable not set".to_string())?;

    Ok(RpcClient::new(rpc_url))
}

// 获取SOL余额
pub async fn get_sol_balance(address: &str) -> Result<TokenBalance, String> {
    let client = get_solana_client()?;

    let pubkey = Pubkey::from_str(address).map_err(|e| format!("Invalid Solana address: {}", e))?;

    let balance = client
        .get_balance(&pubkey)
        .map_err(|e| format!("Failed to get SOL balance: {}", e))?;

    Ok(TokenBalance {
        chain: "SOL".to_string(),
        address: "".to_string(), // SOL没有合约地址
        raw_balance: balance.to_string(),
        decimals: SOL_DECIMALS,
    })
}

// 获取SPL代币余额（简化版，实际应使用SPL token库）
pub async fn get_spl_balance(
    token_address: &str,
    wallet_address: &str,
) -> Result<TokenBalance, String> {
    // 在实际实现中，应该使用spl_token库查询代币余额
    // 这里作为简化示例，返回一个空结果

    Err("Not implemented".to_string())
}

// 验证Solana交易
// pub async fn verify_transaction(tx_signature: &str) -> Result<TransactionVerification, String> {
//     let client = get_solana_client()?;

//     let signature = Signature::from_str(tx_signature)
//         .map_err(|e| format!("Invalid transaction signature: {}", e))?;

//     // 获取交易信息
//     let tx_info = client
//         .get_transaction(
//             &signature,
//             solana_client::rpc_config::RpcTransactionConfig {
//                 encoding: solana_client::rpc_config::RpcTransactionConfig::Base64,
//                 commitment: None,
//                 max_supported_transaction_version: None,
//             },
//         )
//         .map_err(|e| format!("Failed to get transaction: {}", e))?;

//     // 获取交易状态和时间戳
//     let status = if tx_info.meta.as_ref().unwrap().status.is_ok() {
//         "success"
//     } else {
//         "failed"
//     };

//     // 获取交易发送方和接收方（简化，实际上需要解析交易指令）
//     let tx = tx_info.transaction;
//     let from_address = tx.account_keys[0].to_string();
//     let to_address = if tx.account_keys.len() > 1 {
//         tx.account_keys[1].to_string()
//     } else {
//         "".to_string()
//     };

//     // 构建交易验证结果
//     Ok(TransactionVerification {
//         is_valid: tx_info.meta.as_ref().unwrap().status.is_ok(),
//         transaction_hash: tx_signature.to_string(),
//         from_address,
//         to_address,
//         value: tx_info.meta.as_ref().unwrap().fee.to_string(), // 实际值需要从交易指令中提取
//         token_address: None,                                   // 简化，实际需要解析交易指令
//         token_symbol: None,                                    // 简化，实际需要查询token信息
//         timestamp: tx_info.block_time.unwrap_or(0) as i64,
//         block_number: tx_info.slot,
//         status: status.to_string(),
//         chain: "SOL".to_string(),
//     })
// }
