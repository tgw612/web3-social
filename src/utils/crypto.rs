use ethers::prelude::*;
use rand::Rng;
use sha2::{Sha256, Digest};
use std::convert::TryFrom;
use std::str::FromStr;

// 生成随机的挑战码
pub fn generate_nonce() -> String {
    let random_bytes: [u8; 32] = rand::thread_rng().gen();
    hex::encode(&random_bytes)
}

// 验证以太坊签名
pub async fn verify_eth_signature(message: &str, signature: &str, wallet_address: &str) -> bool {
    // 创建前缀消息
    let prefixed_message = format!("\x19Ethereum Signed Message:\n{}{}", message.len(), message);
    
    // 计算前缀消息的哈希
    let mut hasher = Sha256::new();
    hasher.update(prefixed_message.as_bytes());
    let message_hash = hasher.finalize();
    
    // 解析签名
    let signature_bytes = match hex::decode(signature.trim_start_matches("0x")) {
        Ok(bytes) => bytes,
        Err(_) => return false,
    };
    
    let signature = match Signature::try_from(signature_bytes.as_slice()) {
        Ok(sig) => sig,
        Err(_) => return false,
    };
    
    // 从签名恢复地址
    let recovered_address = match signature.recover(H256::from_slice(&message_hash)) {
        Ok(addr) => addr,
        Err(_) => return false,
    };
    
    // 比较恢复的地址和提供的地址
    let provided_address = match Address::from_str(wallet_address.trim_start_matches("0x")) {
        Ok(addr) => addr,
        Err(_) => return false,
    };
    
    recovered_address == provided_address
}

// 验证Solana签名 (简化版，实际应使用solana库进行验证)
pub async fn verify_sol_signature(message: &str, signature: &str, wallet_address: &str) -> bool {
    // 在实际实现中，应该使用solana-sdk进行正确的签名验证
    // 这里仅作为示例占位符
    
    // 1. 应该使用bs58解码签名和公钥
    // 2. 使用ed25519-dalek验证签名
    // 3. 返回验证结果
    
    false // 暂时返回false，需要实际实现
} 