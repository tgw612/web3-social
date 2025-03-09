use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

// Arweave API响应结构
#[derive(Debug, Serialize, Deserialize)]
struct ArweavePostResponse {
    id: String,
}

// 获取Arweave节点URL
pub fn get_arweave_node_url() -> String {
    env::var("ARWEAVE_NODE_URL").unwrap_or_else(|_| "https://arweave.net".to_string())
}

// 上传内容到Arweave（简化版）
// 注意：实际操作需要钱包私钥来签名交易，这里是简化实现
// 在生产环境应该使用arweave-js的Rust移植版或通过HTTP接口与钱包集成
pub async fn upload_to_arweave(data: &str, tags: Option<Vec<(String, String)>>) -> Result<String, String> {
    let client = Client::new();
    let arweave_url = get_arweave_node_url();
    
    // 构建上传数据
    // 注意：这是简化示例，实际应该使用arweave-js等库进行交易构建和签名
    // 这段代码在实际环境中无法工作，只作为代码结构示例
    let response = client
        .post(format!("{}/tx", arweave_url))
        .json(&serde_json::json!({
            "data": data,
            "tags": tags.unwrap_or_default()
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to upload to Arweave: {}", e))?;
    
    // 解析响应
    let arweave_response = response
        .json::<ArweavePostResponse>()
        .await
        .map_err(|e| format!("Failed to parse Arweave response: {}", e))?;
    
    Ok(arweave_response.id)
}

// 从Arweave获取内容
pub async fn get_from_arweave(tx_id: &str) -> Result<String, String> {
    let client = Client::new();
    let arweave_url = get_arweave_node_url();
    
    let response = client
        .get(format!("{}/{}", arweave_url, tx_id))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch from Arweave: {}", e))?;
    
    let content = response
        .text()
        .await
        .map_err(|e| format!("Failed to parse Arweave content: {}", e))?;
    
    Ok(content)
} 