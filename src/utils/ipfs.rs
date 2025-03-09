use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use std::env;
use std::io::Cursor;

// IPFS客户端单例
pub fn get_ipfs_client() -> IpfsClient {
    let ipfs_api_url = env::var("IPFS_API_URL").unwrap_or_else(|_| "http://localhost:5001".to_string());
    let url_parts: Vec<&str> = ipfs_api_url.split("://").collect();
    
    if url_parts.len() != 2 {
        panic!("Invalid IPFS API URL format");
    }
    
    let (protocol, rest) = (url_parts[0], url_parts[1]);
    let host_parts: Vec<&str> = rest.split(':').collect();
    
    if host_parts.len() != 2 {
        panic!("Invalid IPFS API URL format");
    }
    
    let (host, port_str) = (host_parts[0], host_parts[1]);
    let port = port_str.parse::<u16>().expect("Invalid IPFS API port");
    
    match protocol {
        "http" => IpfsClient::new(host, port),
        "https" => IpfsClient::new_with_https(host, port),
        _ => panic!("Unsupported IPFS API protocol: {}", protocol),
    }
}

// 获取IPFS网关URL
pub fn get_ipfs_gateway_url() -> String {
    env::var("IPFS_GATEWAY_URL").unwrap_or_else(|_| "http://localhost:8080".to_string())
}

// 上传文件到IPFS
pub async fn upload_to_ipfs(data: Vec<u8>) -> Result<String, String> {
    let client = get_ipfs_client();
    let cursor = Cursor::new(data);
    
    match client.add(cursor).await {
        Ok(res) => Ok(res.hash),
        Err(e) => Err(format!("Failed to upload to IPFS: {}", e)),
    }
}

// 从IPFS获取内容
pub async fn get_from_ipfs(cid: &str) -> Result<Vec<u8>, String> {
    let client = get_ipfs_client();
    
    match client.cat(cid).await {
        Ok(bytes) => Ok(bytes.to_vec()),
        Err(e) => Err(format!("Failed to get from IPFS: {}", e)),
    }
} 