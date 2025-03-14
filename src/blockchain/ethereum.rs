use crate::models::asset::{Asset, NFT, TokenBalance, TokenPrice, TransactionVerification};
use ethers::prelude::*;
use ethers::types::{Address, BlockNumber, U256};
use ethers::providers::{Http, Provider};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::env;
use std::str::FromStr;
use std::sync::Arc;

// ERC20合约ABI
const ERC20_ABI: &str = r#"[
    {
        "constant": true,
        "inputs": [{"name": "_owner", "type": "address"}],
        "name": "balanceOf",
        "outputs": [{"name": "balance", "type": "uint256"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "decimals",
        "outputs": [{"name": "", "type": "uint8"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "symbol",
        "outputs": [{"name": "", "type": "string"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "name",
        "outputs": [{"name": "", "type": "string"}],
        "type": "function"
    }
]"#;

// NFT合约ABI
const ERC721_ABI: &str = r#"[
    {
        "constant": true,
        "inputs": [{"name": "_owner", "type": "address"}],
        "name": "balanceOf",
        "outputs": [{"name": "balance", "type": "uint256"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [{"name": "_tokenId", "type": "uint256"}],
        "name": "ownerOf",
        "outputs": [{"name": "owner", "type": "address"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [{"name": "_tokenId", "type": "uint256"}],
        "name": "tokenURI",
        "outputs": [{"name": "tokenURI", "type": "string"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "name",
        "outputs": [{"name": "", "type": "string"}],
        "type": "function"
    },
    {
        "constant": true,
        "inputs": [],
        "name": "symbol",
        "outputs": [{"name": "", "type": "string"}],
        "type": "function"
    }
]"#;

// 创建以太坊客户端
pub async fn get_eth_client() -> Result<Arc<Provider<Http>>, String> {
    let rpc_url = env::var("ETHEREUM_RPC_URL")
        .map_err(|_| "ETHEREUM_RPC_URL environment variable not set".to_string())?;
    
    let provider = Provider::<Http>::try_from(rpc_url)
        .map_err(|e| format!("Failed to create Ethereum provider: {}", e))?;
    
    Ok(Arc::new(provider))
}

// 获取ETH余额
pub async fn get_eth_balance(address: &str) -> Result<TokenBalance, String> {
    let client = get_eth_client().await?;
    
    let address = Address::from_str(address.trim_start_matches("0x"))
        .map_err(|e| format!("Invalid Ethereum address: {}", e))?;
    
    let balance = client.get_balance(address, None)
        .await
        .map_err(|e| format!("Failed to get ETH balance: {}", e))?;
    
    Ok(TokenBalance {
        chain: "ETH".to_string(),
        address: "".to_string(), // ETH没有合约地址
        raw_balance: balance.to_string(),
        decimals: 18,
    })
}

// 获取ERC20代币余额
pub async fn get_erc20_balance(token_address: &str, wallet_address: &str) -> Result<TokenBalance, String> {
    let client = get_eth_client().await?;
    
    let token_address = Address::from_str(token_address.trim_start_matches("0x"))
        .map_err(|e| format!("Invalid token address: {}", e))?;
    
    let wallet_address = Address::from_str(wallet_address.trim_start_matches("0x"))
        .map_err(|e| format!("Invalid wallet address: {}", e))?;
    
    // 创建合约实例
    let contract = Contract::new(
        token_address,
        serde_json::from_str::<ethers::abi::Abi>(ERC20_ABI).unwrap(),
        client.clone(),
    );
    
    // 调用balanceOf
    let balance: U256 = contract.method("balanceOf", wallet_address)
        .map_err(|e| format!("Failed to create balanceOf method call: {}", e))?
        .call()
        .await
        .map_err(|e| format!("Failed to call balanceOf: {}", e))?;
    
    // 获取代币精度
    let decimals: u8 = contract.method("decimals", ())
        .map_err(|e| format!("Failed to create decimals method call: {}", e))?
        .call()
        .await
        .map_err(|e| format!("Failed to call decimals: {}", e))?;
    
    Ok(TokenBalance {
        chain: "ETH".to_string(),
        address: format!("0x{}", hex::encode(token_address.as_bytes())),
        raw_balance: balance.to_string(),
        decimals,
    })
}

// 获取代币信息
pub async fn get_token_info(token_address: &str) -> Result<(String, String), String> {
    let client = get_eth_client().await?;
    
    let token_address = Address::from_str(token_address.trim_start_matches("0x"))
        .map_err(|e| format!("Invalid token address: {}", e))?;
    
    // 创建合约实例
    let contract = Contract::new(
        token_address,
        serde_json::from_str::<ethers::abi::Abi>(ERC20_ABI).unwrap(),
        client.clone(),
    );
    
    // 获取代币符号
    let symbol: String = contract.method("symbol", ())
        .map_err(|e| format!("Failed to create symbol method call: {}", e))?
        .call()
        .await
        .map_err(|e| format!("Failed to call symbol: {}", e))?;
    
    // 获取代币名称
    let name: String = contract.method("name", ())
        .map_err(|e| format!("Failed to create name method call: {}", e))?
        .call()
        .await
        .map_err(|e| format!("Failed to call name: {}", e))?;
    
    Ok((symbol, name))
}

// 验证交易
pub async fn verify_transaction(tx_hash: &str) -> Result<TransactionVerification, String> {
    let client = get_eth_client().await?;
    
    let tx_hash = H256::from_str(tx_hash.trim_start_matches("0x"))
        .map_err(|e| format!("Invalid transaction hash: {}", e))?;
    
    // 获取交易信息
    let tx = client.get_transaction(tx_hash)
        .await
        .map_err(|e| format!("Failed to get transaction: {}", e))?
        .ok_or_else(|| "Transaction not found".to_string())?;
    
    // 获取交易收据
    let receipt = client.get_transaction_receipt(tx_hash)
        .await
        .map_err(|e| format!("Failed to get transaction receipt: {}", e))?
        .ok_or_else(|| "Transaction receipt not found".to_string())?;
    
    // 获取区块时间戳
    let block = client.get_block(receipt.block_number.unwrap())
        .await
        .map_err(|e| format!("Failed to get block: {}", e))?
        .ok_or_else(|| "Block not found".to_string())?;
    
    // 构建交易验证结果
    let status = if receipt.status.unwrap_or_default() == U64::from(1) {
        "success"
    } else {
        "failed"
    };
    
    // 判断是否为ERC20交易
    let (token_address, token_symbol) = if tx.input.len() > 4 && &tx.input.to_vec()[0..4] == &[0xa9, 0x05, 0x9c, 0xbb] {
        // 这是ERC20 transfer方法的签名
        // transfer(address,uint256)
        
        let token_address = tx.to.map(|addr| format!("0x{}", hex::encode(addr.as_bytes())));
        
        let token_symbol = if let Some(addr) = token_address.clone() {
            match get_token_info(&addr).await {
                Ok((symbol, _)) => Some(symbol),
                Err(_) => None,
            }
        } else {
            None
        };
        
        (token_address, token_symbol)
    } else {
        (None, None)
    };
    
    Ok(TransactionVerification {
        is_valid: receipt.status.unwrap_or_default() == U64::from(1),
        transaction_hash: format!("0x{}", hex::encode(tx_hash.as_bytes())),
        from_address: format!("0x{}", hex::encode(tx.from.as_bytes())),
        to_address: tx.to.map_or("".to_string(), |addr| format!("0x{}", hex::encode(addr.as_bytes()))),
        value: tx.value.to_string(),
        token_address,
        token_symbol,
        timestamp: block.timestamp.as_u64() as i64,
        block_number: receipt.block_number.unwrap().as_u64(),
        status: status.to_string(),
        chain: "ETH".to_string(),
    })
} 