# web3-social

#### 介绍
基于 web3 的社交匿名财富交流平台             

#### 软件架构
软件架构说明


#### 安装教程

1.  xxxx
2.  xxxx
3.  xxxx

#### 使用说明

1.  xxxx
2.  xxxx
3.  xxxx

#### 参与贡献

1.  Fork 本仓库
2.  新建 Feat_xxx 分支
3.  提交代码
4.  新建 Pull Request


#### 特性

一、核心需求

于 web3 的社交匿名财富交流平台  
​用户身份系统：用户通过钱包扫码登录，支持 okx，coinbase binance等钱包（支持 ETH、Solana 等多链）生成匿名身份
​资产展示：完全公开链上资产数据，无需隐私化处理。
​用户资料：支持自定义用户名、头像、昵称，并展示钱包地址与资产量。
社交功能：模仿雪球网站的社交系统 https://xueqiu.com/u/5073331412?md5__1038=n4mx2D0DcDnDyDjhxRxxBM8CQqqGqm5uqAphD
拥有发帖、评论、资产展示等基础功能，同时用户发言不可删除，永久存储。
二、功能模块设计

1. ​用户身份系统

​钱包登录：用户通过连接okx，coinbase binance等主流钱包（支持 ETH、Solana 等多链）生成匿名身份。
​用户名与资料：
用户可设置唯一用户名（如user123），与钱包地址绑定。
资料页展示：头像（支持上传至 IPFS）、昵称、钱包地址、链上资产总额（自动聚合多链资产）。
​去中心化存储：用户资料（头像、昵称）通过IPFS 存储，确保不可篡改。
2. ​资产展示模块
​多链资产聚合：
通过区块链节点 API（如 Alchemy、QuickNode）获取用户地址的代币、NFT 数据。
前端展示总资产价值（按实时价格换算）、持仓分布（饼图/柱状图）。
​公开透明度：
用户资产数据完全公开，其他用户可通过地址或用户名查询任意用户的资产详情。
3. ​内容交流系统

​发帖与评论：
发帖内容支持文本、图片（存储至 IPFS），关联钱包地址作为作者标识。
帖子按热度（点赞/评论数）或时间排序，支持标签分类（如#DeFi、#NFT）。
​投资信息关联：
用户发帖时可附加链上交易哈希，验证其投资操作的真实性（如"我在 Uniswap 买入 1 ETH"）。
​去中心化存储：帖子内容加密后存储至 Arweave，确保永久留存。
三、技术栈推荐

1. ​后端与区块链层（Rust 核心）​

​区块链交互：
使用 web3-rs 库连接 Ethereum、Solana 节点，获取用户资产数据。
通过 solana-client 库处理 Solana 链上交易查询。
​智能合约​（可选）：
用户资料存储合约：使用 ink! 编写，存储用户名与 IPFS 头像 CID（内容标识符）。
​API 服务：
框架：actix-web 提供 REST API，处理用户资料查询、帖子列表等逻辑。
数据缓存：使用 Redis 缓存高频访问的链上数据（如资产价格）。
2. ​前端与移动端（UniApp 核心）​
​钱包集成：
使用 @uni-helper/uni-web3 插件兼容多端钱包连接（H5/小程序/APP）。
通过 ethers.js 实现交易签名、地址验证。
​资产展示：
数据可视化：uCharts 插件绘制资产分布图表。
多链价格聚合：调用 CoinGecko API 获取实时代币价格。
​内容展示：
IPFS 图片加载：集成 @unipkg/ipfs-http-client 从 IPFS 网关获取图片。
Markdown 渲染：用户帖子支持富文本格式（如代码块、超链接）。
3. ​基础设施

​去中心化存储：
图片/帖子存储：使用 Crust Network 或 Arweave 实现永久存储。
​节点服务：
Ethereum 节点：使用 Infura。
Solana 节点：使用 QuickNode 的 RPC 服务。
四、部署与测试

​智能合约部署​（如适用）：
使用 cargo-contract 编译合约，部署至测试网（如 Ethereum Goerli、Solana Devnet）。
​后端服务：
通过 Docker 容器化部署 actix-web API 服务，配置 Nginx 反向代理。
​前端构建：
使用 npm run build:app-plus 生成多端应用，发布至 H5 服务器及应用商店。
五、安全与合规

​钱包签名验证：用户修改资料时需用钱包私钥签名，防止篡改。
​内容过滤：集成 AI 模型（如 TensorFlow.js）自动屏蔽敏感信息。
​合规提示：在资产展示页注明"数据仅供参考，不构成投资建议"。

## 后端开发指南

### 技术栈

- **核心语言**: Rust
- **Web框架**: actix-web
- **数据库**: PostgreSQL (通过sqlx操作)
- **缓存**: Redis
- **区块链交互**: 
  - Ethereum: ethers-rs, web3-rs
  - Solana: solana-client
- **存储**: IPFS, Arweave
- **认证**: JWT, 钱包签名验证

### 项目结构

```
web3-social-backend/
├── src/
│   ├── api/           # API路由和处理函数
│   ├── blockchain/    # 区块链交互逻辑
│   ├── config/        # 配置管理
│   ├── middlewares/   # 中间件（认证、速率限制等）
│   ├── models/        # 数据模型和结构
│   ├── services/      # 业务逻辑
│   ├── utils/         # 工具函数
│   └── main.rs        # 应用入口
├── migrations/        # 数据库迁移文件
├── Cargo.toml         # 项目依赖
└── .env.example       # 环境变量示例
```

### 功能模块

1. **用户身份系统**
   - 钱包登录（支持ETH、Solana等多链）
   - 用户资料管理（用户名、头像、昵称）
   - 钱包地址与用户身份绑定

2. **资产展示模块**
   - 多链资产聚合（代币、NFT）
   - 资产价值计算和展示
   - 交易验证

3. **内容交流系统**
   - 发帖与评论
   - 内容永久存储（IPFS、Arweave）
   - 点赞和热度排序

### 开发环境设置

1. 安装Rust和Cargo
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. 安装PostgreSQL和Redis

3. 克隆仓库并安装依赖
   ```bash
   git clone https://github.com/your-username/web3-social.git
   cd web3-social
   cp .env.example .env  # 复制并配置环境变量
   ```

4. 设置数据库
   ```bash
   psql -U postgres -c "CREATE DATABASE web3social;"
   psql -U postgres -d web3social -f migrations/20230901000000_initial_schema.sql
   ```

5. 运行开发服务器
   ```bash
   cargo run
   ```

### API文档

API文档将在服务启动后可通过 `http://localhost:8080/api/docs` 访问。

### 部署

1. 构建发布版本
   ```bash
   cargo build --release
   ```

2. 使用Docker部署
   ```bash
   docker-compose up -d
   ```
