use crate::config::Config;
use crate::utils::error::ServiceError;
use std::sync::Arc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use rbatis::RBatis;

/// 存储服务，处理IPFS和Arweave去中心化存储
pub struct StorageService {
    config: Arc<Config>,
    http_client: Client,
    db: Option<Arc<RBatis>>,
}

impl StorageService {
    pub fn new(config: Arc<Config>, db: Option<Arc<RBatis>>) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            db,
        }
    }

    /// 上传内容到IPFS
    pub async fn upload_to_ipfs(&self, data: &[u8]) -> Result<String, ServiceError> {
        #[derive(Serialize)]
        struct IPFSUploadRequest<'a> {
            file: &'a [u8],
        }

        #[derive(Deserialize)]
        struct IPFSUploadResponse {
            cid: String,
        }

        // 使用IPFS HTTP API上传
        let response = self.http_client
            .post(&self.config.ipfs.api_url)
            .header("X-API-KEY", &self.config.ipfs.api_key)
            .json(&IPFSUploadRequest { file: data })
            .send()
            .await
            .map_err(|e| ServiceError::ExternalService(format!("IPFS服务错误: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".into());
            return Err(ServiceError::ExternalService(format!("IPFS上传失败: {}", error_text)));
        }

        let upload_response = response.json::<IPFSUploadResponse>().await
            .map_err(|e| ServiceError::ExternalService(format!("解析IPFS响应失败: {}", e)))?;

        Ok(upload_response.cid)
    }

    /// 上传内容到Arweave
    pub async fn upload_to_arweave(&self, data: &[u8]) -> Result<String, ServiceError> {
        #[derive(Serialize)]
        struct ArweaveUploadRequest<'a> {
            data: &'a [u8],
            tags: Vec<ArweaveTag>,
        }

        #[derive(Serialize)]
        struct ArweaveTag {
            name: String,
            value: String,
        }

        #[derive(Deserialize)]
        struct ArweaveUploadResponse {
            id: String,
        }

        // 创建标签，表明内容类型和应用来源
        let tags = vec![
            ArweaveTag {
                name: "Content-Type".to_string(),
                value: "text/plain".to_string(),
            },
            ArweaveTag {
                name: "App-Name".to_string(),
                value: "Web3Social".to_string(),
            },
        ];

        // 使用Arweave HTTP API或代理服务上传
        let response = self.http_client
            .post(&self.config.arweave.api_url)
            .header("X-API-KEY", &self.config.arweave.api_key)
            .json(&ArweaveUploadRequest { data, tags })
            .send()
            .await
            .map_err(|e| ServiceError::ExternalService(format!("Arweave服务错误: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".into());
            return Err(ServiceError::ExternalService(format!("Arweave上传失败: {}", error_text)));
        }

        let upload_response = response.json::<ArweaveUploadResponse>().await
            .map_err(|e| ServiceError::ExternalService(format!("解析Arweave响应失败: {}", e)))?;

        Ok(upload_response.id)
    }

    /// 从IPFS获取内容
    pub async fn get_from_ipfs(&self, cid: &str) -> Result<Vec<u8>, ServiceError> {
        // 构建IPFS网关URL
        let url = format!("{}/ipfs/{}", self.config.ipfs.gateway_url, cid);

        // 获取内容
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalService(format!("IPFS获取错误: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".into());
            return Err(ServiceError::ExternalService(format!("IPFS内容获取失败: {}", error_text)));
        }

        let bytes = response.bytes().await
            .map_err(|e| ServiceError::ExternalService(format!("读取IPFS响应失败: {}", e)))?;

        Ok(bytes.to_vec())
    }

    /// 从Arweave获取内容
    pub async fn get_from_arweave(&self, id: &str) -> Result<Vec<u8>, ServiceError> {
        // 构建Arweave网关URL
        let url = format!("{}/{}", self.config.arweave.gateway_url, id);

        // 获取内容
        let response = self.http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| ServiceError::ExternalService(format!("Arweave获取错误: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "未知错误".into());
            return Err(ServiceError::ExternalService(format!("Arweave内容获取失败: {}", error_text)));
        }

        let bytes = response.bytes().await
            .map_err(|e| ServiceError::ExternalService(format!("读取Arweave响应失败: {}", e)))?;

        Ok(bytes.to_vec())
    }
    
    /// 生成IPFS内容URL
    pub fn get_ipfs_url(&self, cid: &str) -> String {
        format!("{}/ipfs/{}", self.config.ipfs.gateway_url, cid)
    }
    
    /// 生成Arweave内容URL
    pub fn get_arweave_url(&self, id: &str) -> String {
        format!("{}/{}", self.config.arweave.gateway_url, id)
    }
}