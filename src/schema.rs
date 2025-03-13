// @generated automatically by Diesel CLI.

// 添加 Diesel 导入
use diesel::prelude::*;
use diesel::sql_types::*;

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Text,
        nickname -> Nullable<Text>,
        wallet_address -> Text,
        wallet_chain -> Text,
        avatar_ipfs_cid -> Nullable<Text>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    posts (id) {
        id -> Uuid,
        user_id -> Uuid,
        content -> Text,
        images_ipfs_cids -> Nullable<Array<Text>>,
        arweave_tx_id -> Nullable<Text>,
        transaction_hash -> Nullable<Text>,
        transaction_chain -> Nullable<Text>,
        like_count -> Int4,
        comment_count -> Int4,
        tags -> Nullable<Array<Text>>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    comments (id) {
        id -> Uuid,
        post_id -> Uuid,
        user_id -> Uuid,
        parent_id -> Nullable<Uuid>,
        content -> Text,
        arweave_tx_id -> Nullable<Text>,
        like_count -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    user_likes (id) {
        id -> Uuid,
        user_id -> Uuid,
        post_id -> Nullable<Uuid>,
        comment_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    auth_challenges (id) {
        id -> Uuid,
        wallet_address -> Text,
        wallet_chain -> Text,
        nonce -> Text,
        created_at -> Timestamptz,
        expires_at -> Timestamptz,
    }
}

diesel::joinable!(posts -> users (user_id));
diesel::joinable!(comments -> users (user_id));
diesel::joinable!(comments -> posts (post_id));
diesel::joinable!(user_likes -> users (user_id));
diesel::joinable!(user_likes -> posts (post_id));
diesel::joinable!(user_likes -> comments (comment_id));

diesel::allow_tables_to_appear_in_same_query!(
    users,
    posts,
    comments,
    user_likes,
    auth_challenges,
); 