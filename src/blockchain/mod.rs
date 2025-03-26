pub mod ethereum;
pub mod solana;

// 重新导出常用函数
pub use ethereum::{get_eth_balance, get_erc20_balance, verify_transaction as verify_eth_transaction};
pub use solana::{get_sol_balance, get_spl_balance }; 