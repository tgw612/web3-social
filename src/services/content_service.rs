use crate::models::content::{Post, Comment, Like, Tag};
use crate::services::storage_service::StorageService;
use crate::utils::error::ServiceError;
use sqlx::{Pool, Postgres};
use std::sync::Arc;

/// 内容服务，处理发帖、评论、点赞等社交功能
pub struct ContentService {
    db: Pool<Postgres>,
    storage_service: Arc<StorageService>,
}

impl ContentService {
    pub fn new(db: Pool<Postgres>, storage_service: Arc<StorageService>) -> Self {
        Self { db, storage_service }
    }

    /// 创建新帖子
    pub async fn create_post(
        &self,
        user_id: i32,
        wallet_address: &str,
        content: &str,
        image_data: Option<Vec<u8>>,
        tags: Vec<String>,
        tx_hash: Option<String>,
    ) -> Result<Post, ServiceError> {
        // 如果有图片，先上传到IPFS
        let image_cid = match image_data {
            Some(data) => Some(self.storage_service.upload_to_ipfs(&data).await?),
            None => None,
        };

        // 将内容存储到Arweave
        let content_id = self.storage_service.upload_to_arweave(content.as_bytes()).await?;

        // 在数据库中创建帖子
        let post = sqlx::query_as!(
            Post,
            r#"
            INSERT INTO posts (
                user_id, wallet_address, content, content_arweave_id,
                image_ipfs_cid, tx_hash
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            user_id,
            wallet_address,
            content,
            content_id,
            image_cid,
            tx_hash
        )
        .fetch_one(&self.db)
        .await?;

        // 处理标签
        for tag in tags {
            // 创建或获取标签ID
            let tag_id = self.get_or_create_tag(&tag).await?;
            
            // 关联帖子和标签
            sqlx::query!(
                r#"
                INSERT INTO post_tags (post_id, tag_id)
                VALUES ($1, $2)
                "#,
                post.id,
                tag_id
            )
            .execute(&self.db)
            .await?;
        }

        Ok(post)
    }

    /// 获取或创建标签
    async fn get_or_create_tag(&self, tag_name: &str) -> Result<i32, ServiceError> {
        // 查询标签是否存在
        let tag = sqlx::query!(
            r#"
            SELECT id FROM tags 
            WHERE name = $1
            "#,
            tag_name
        )
        .fetch_optional(&self.db)
        .await?;

        // 如果标签不存在，创建新标签
        if let Some(tag) = tag {
            Ok(tag.id)
        } else {
            let tag = sqlx::query!(
                r#"
                INSERT INTO tags (name)
                VALUES ($1)
                RETURNING id
                "#,
                tag_name
            )
            .fetch_one(&self.db)
            .await?;

            Ok(tag.id)
        }
    }

    /// 按热度获取帖子列表
    pub async fn get_posts_by_hot(&self, page: i32, page_size: i32) -> Result<Vec<Post>, ServiceError> {
        let offset = (page - 1) * page_size;
        
        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT p.* FROM posts p
            LEFT JOIN (
                SELECT post_id, COUNT(*) as like_count 
                FROM likes 
                GROUP BY post_id
            ) l ON p.id = l.post_id
            LEFT JOIN (
                SELECT post_id, COUNT(*) as comment_count 
                FROM comments 
                GROUP BY post_id
            ) c ON p.id = c.post_id
            ORDER BY (COALESCE(l.like_count, 0) + COALESCE(c.comment_count, 0) * 2) DESC, p.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            page_size,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(posts)
    }

    /// 按时间获取帖子列表
    pub async fn get_posts_by_time(&self, page: i32, page_size: i32) -> Result<Vec<Post>, ServiceError> {
        let offset = (page - 1) * page_size;
        
        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT * FROM posts
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            page_size,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(posts)
    }

    /// 按标签获取帖子列表
    pub async fn get_posts_by_tag(&self, tag: &str, page: i32, page_size: i32) -> Result<Vec<Post>, ServiceError> {
        let offset = (page - 1) * page_size;
        
        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT p.* FROM posts p
            JOIN post_tags pt ON p.id = pt.post_id
            JOIN tags t ON pt.tag_id = t.id
            WHERE t.name = $1
            ORDER BY p.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            tag,
            page_size,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(posts)
    }

    /// 获取用户帖子列表
    pub async fn get_user_posts(&self, user_id: i32, page: i32, page_size: i32) -> Result<Vec<Post>, ServiceError> {
        let offset = (page - 1) * page_size;
        
        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT * FROM posts
            WHERE user_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            page_size,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(posts)
    }

    /// 获取帖子详情
    pub async fn get_post(&self, post_id: i32) -> Result<Post, ServiceError> {
        let post = sqlx::query_as!(
            Post,
            r#"
            SELECT * FROM posts 
            WHERE id = $1
            "#,
            post_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(ServiceError::NotFound("帖子不存在".into()))?;

        Ok(post)
    }

    /// 创建评论
    pub async fn create_comment(
        &self,
        user_id: i32,
        wallet_address: &str,
        post_id: i32,
        content: &str,
        parent_id: Option<i32>,
    ) -> Result<Comment, ServiceError> {
        // 验证帖子是否存在
        self.get_post(post_id).await?;
        
        // 如果是回复评论，验证父评论是否存在
        if let Some(parent_id) = parent_id {
            self.get_comment(parent_id).await?;
        }

        // 将内容存储到Arweave
        let content_id = self.storage_service.upload_to_arweave(content.as_bytes()).await?;

        // 创建评论
        let comment = sqlx::query_as!(
            Comment,
            r#"
            INSERT INTO comments (
                user_id, wallet_address, post_id, content, 
                content_arweave_id, parent_id
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING *
            "#,
            user_id,
            wallet_address,
            post_id,
            content,
            content_id,
            parent_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(comment)
    }

    /// 获取评论列表
    pub async fn get_comments(&self, post_id: i32, page: i32, page_size: i32) -> Result<Vec<Comment>, ServiceError> {
        let offset = (page - 1) * page_size;
        
        let comments = sqlx::query_as!(
            Comment,
            r#"
            SELECT * FROM comments
            WHERE post_id = $1 AND parent_id IS NULL
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#,
            post_id,
            page_size,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(comments)
    }

    /// 获取评论回复
    pub async fn get_comment_replies(&self, comment_id: i32, page: i32, page_size: i32) -> Result<Vec<Comment>, ServiceError> {
        let offset = (page - 1) * page_size;
        
        let replies = sqlx::query_as!(
            Comment,
            r#"
            SELECT * FROM comments
            WHERE parent_id = $1
            ORDER BY created_at ASC
            LIMIT $2 OFFSET $3
            "#,
            comment_id,
            page_size,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(replies)
    }

    /// 获取评论详情
    pub async fn get_comment(&self, comment_id: i32) -> Result<Comment, ServiceError> {
        let comment = sqlx::query_as!(
            Comment,
            r#"
            SELECT * FROM comments 
            WHERE id = $1
            "#,
            comment_id
        )
        .fetch_optional(&self.db)
        .await?
        .ok_or(ServiceError::NotFound("评论不存在".into()))?;

        Ok(comment)
    }

    /// 点赞帖子
    pub async fn like_post(&self, user_id: i32, post_id: i32) -> Result<(), ServiceError> {
        // 验证帖子是否存在
        self.get_post(post_id).await?;
        
        // 检查是否已点赞
        let exists = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM likes 
            WHERE user_id = $1 AND post_id = $2
            "#,
            user_id,
            post_id
        )
        .fetch_one(&self.db)
        .await?
        .count
        .unwrap_or(0) > 0;

        if exists {
            return Err(ServiceError::BadRequest("已经点赞过该帖子".into()));
        }

        // 创建点赞记录
        sqlx::query!(
            r#"
            INSERT INTO likes (user_id, post_id)
            VALUES ($1, $2)
            "#,
            user_id,
            post_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// 取消点赞
    pub async fn unlike_post(&self, user_id: i32, post_id: i32) -> Result<(), ServiceError> {
        sqlx::query!(
            r#"
            DELETE FROM likes 
            WHERE user_id = $1 AND post_id = $2
            "#,
            user_id,
            post_id
        )
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// 获取帖子点赞数
    pub async fn get_post_likes_count(&self, post_id: i32) -> Result<i64, ServiceError> {
        let count = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM likes 
            WHERE post_id = $1
            "#,
            post_id
        )
        .fetch_one(&self.db)
        .await?
        .count
        .unwrap_or(0);

        Ok(count)
    }

    /// 检查用户是否已点赞
    pub async fn has_user_liked(&self, user_id: i32, post_id: i32) -> Result<bool, ServiceError> {
        let exists = sqlx::query!(
            r#"
            SELECT COUNT(*) as count FROM likes 
            WHERE user_id = $1 AND post_id = $2
            "#,
            user_id,
            post_id
        )
        .fetch_one(&self.db)
        .await?
        .count
        .unwrap_or(0) > 0;

        Ok(exists)
    }

    /// 获取热门标签
    pub async fn get_hot_tags(&self, limit: i32) -> Result<Vec<Tag>, ServiceError> {
        let tags = sqlx::query_as!(
            Tag,
            r#"
            SELECT t.id, t.name, COUNT(pt.post_id) as post_count
            FROM tags t
            JOIN post_tags pt ON t.id = pt.tag_id
            GROUP BY t.id, t.name
            ORDER BY COUNT(pt.post_id) DESC
            LIMIT $1
            "#,
            limit
        )
        .fetch_all(&self.db)
        .await?;

        Ok(tags)
    }

    /// 搜索帖子
    pub async fn search_posts(&self, query: &str, page: i32, page_size: i32) -> Result<Vec<Post>, ServiceError> {
        let offset = (page - 1) * page_size;
        let search_term = format!("%{}%", query);
        
        let posts = sqlx::query_as!(
            Post,
            r#"
            SELECT p.* FROM posts p
            WHERE p.content ILIKE $1
            ORDER BY p.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            search_term,
            page_size,
            offset
        )
        .fetch_all(&self.db)
        .await?;

        Ok(posts)
    }
} 