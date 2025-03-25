// use crate::models::comment::Comment;
// use crate::models::post::Post;
// use crate::models::rbatis_entities::{CommentEntity, PostEntity, TagEntity, UserLikeEntity};
// use crate::services::storage_service::StorageService;
// use crate::utils::error::ServiceError;
// use rbatis::RBatis;
// use rbatis::rbdc::datetime::DateTime;
// use std::sync::Arc;

// /// 内容服务，处理发帖、评论、点赞等社交功能
// pub struct ContentService {
//     db: Arc<RBatis>,
//     storage_service: Arc<StorageService>,
// }

// impl ContentService {
//     pub fn new(db: Arc<RBatis>, storage_service: Arc<StorageService>) -> Self {
//         Self {
//             db,
//             storage_service,
//         }
//     }

//     /// 创建新帖子
//     pub async fn create_post(
//         &self,
//         user_id: uuid::Uuid,
//         wallet_address: &str,
//         content: &str,
//         image_data: Option<Vec<u8>>,
//         tags: Vec<String>,
//         tx_hash: Option<String>,
//     ) -> Result<Post, ServiceError> {
//         // 如果有图片，先上传到IPFS
//         let image_cid = match image_data {
//             Some(data) => Some(self.storage_service.upload_to_ipfs(&data).await?),
//             None => None,
//         };

//         // 将内容存储到Arweave
//         let content_id = self
//             .storage_service
//             .upload_to_arweave(content.as_bytes())
//             .await?;

//         // 创建帖子实体
//         let post_entity = PostEntity {
//             id: 0, // 数据库会自动生成ID
//             user_id,
//             content: content.to_string(),
//             images_ipfs_cids: if let Some(cid) = &image_cid {
//                 Some(vec![cid.clone()])
//             } else {
//                 None
//             },
//             arweave_tx_id: Some(content_id),
//             transaction_hash: tx_hash,
//             transaction_chain: None,
//             like_count: 0,
//             comment_count: 0,
//             tags: Some(tags.clone()),
//             created_at: rbatis::rbdc::datetime::DateTime::now(),
//             updated_at: rbatis::rbdc::datetime::DateTime::now(),
//         };

//         // 保存帖子
//         self.db
//             .save(&post_entity, &[])
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         // 将实体转换为Post模型
//         let post = Post {
//             id: uuid::Uuid::new_v4(), // 这里需要根据实际情况调整
//             user_id,
//             content: content.to_string(),
//             images_ipfs_cids: if let Some(cid) = image_cid {
//                 Some(vec![cid])
//             } else {
//                 None
//             },
//             arweave_tx_id: Some(content_id),
//             transaction_hash: tx_hash,
//             transaction_chain: None,
//             like_count: 0,
//             comment_count: 0,
//             tags: Some(tags),
//             created_at: chrono::Utc::now(),
//             updated_at: chrono::Utc::now(),
//         };

//         Ok(post)
//     }

//     /// 获取或创建标签
//     async fn get_or_create_tag(&self, tag_name: &str) -> Result<i32, ServiceError> {
//         // 使用rbatis查询标签
//         let wrapper = self.db.new_wrapper().eq("name", tag_name);

//         // 假设有TagEntity，如果没有，需要在rbatis_entities.rs中添加
//         let tag = self
//             .db
//             .fetch_by_wrapper::<TagEntity>(wrapper)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         if let Some(tag) = tag {
//             Ok(tag.id)
//         } else {
//             // 创建新标签
//             let new_tag = TagEntity {
//                 id: 0, // 数据库会自动生成ID
//                 name: tag_name.to_string(),
//             };

//             self.db
//                 .save(&new_tag, &[])
//                 .await
//                 .map_err(|_| ServiceError::InternalServerError)?;

//             // 获取新创建的标签ID
//             let wrapper = self.db.new_wrapper().eq("name", tag_name);

//             let new_tag = self
//                 .db
//                 .fetch_by_wrapper::<TagEntity>(wrapper)
//                 .await
//                 .map_err(|_| ServiceError::InternalServerError)?
//                 .ok_or(ServiceError::InternalServerError)?;

//             Ok(new_tag.id)
//         }
//     }

//     /// 按热度获取帖子列表
//     pub async fn get_posts_by_hot(
//         &self,
//         page: i32,
//         page_size: i32,
//     ) -> Result<Vec<Post>, ServiceError> {
//         let offset = (page - 1) * page_size;

//         // 使用rbatis执行原生SQL查询
//         let sql = format!(
//             r#"
//             SELECT * FROM posts
//             ORDER BY (COALESCE(like_count, 0) + COALESCE(comment_count, 0) * 2) DESC, created_at DESC
//             LIMIT {} OFFSET {}
//         "#,
//             page_size, offset
//         );

//         let post_entities: Vec<PostEntity> = self
//             .db
//             .fetch_by_sql(&sql)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         // 将实体转换为Post模型
//         let posts = post_entities
//             .into_iter()
//             .map(|entity| {
//                 Post {
//                     id: uuid::Uuid::new_v4(), // 这里需要根据实际情况调整
//                     user_id: entity.user_id,
//                     content: entity.content,
//                     images_ipfs_cids: entity.images_ipfs_cids,
//                     arweave_tx_id: entity.arweave_tx_id,
//                     transaction_hash: entity.transaction_hash,
//                     transaction_chain: entity.transaction_chain,
//                     like_count: entity.like_count,
//                     comment_count: entity.comment_count,
//                     tags: entity.tags,
//                     created_at: entity.created_at.into(),
//                     updated_at: entity.updated_at.into(),
//                 }
//             })
//             .collect();

//         Ok(posts)
//     }

//     /// 按时间获取帖子列表
//     pub async fn get_posts_by_time(
//         &self,
//         page: i32,
//         page_size: i32,
//     ) -> Result<Vec<Post>, ServiceError> {
//         let offset = (page - 1) * page_size;

//         // 使用rbatis分页查询
//         let wrapper = self.db.new_wrapper().order_by(true, &["created_at"]);

//         let page_result = self
//             .db
//             .fetch_page_by_wrapper::<PostEntity>(wrapper, page as u64, page_size as u64)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         // 将实体转换为Post模型
//         let posts = page_result
//             .records
//             .into_iter()
//             .map(|entity| {
//                 Post {
//                     id: uuid::Uuid::new_v4(), // 这里需要根据实际情况调整
//                     user_id: entity.user_id,
//                     content: entity.content,
//                     images_ipfs_cids: entity.images_ipfs_cids,
//                     arweave_tx_id: entity.arweave_tx_id,
//                     transaction_hash: entity.transaction_hash,
//                     transaction_chain: entity.transaction_chain,
//                     like_count: entity.like_count,
//                     comment_count: entity.comment_count,
//                     tags: entity.tags,
//                     created_at: entity.created_at.into(),
//                     updated_at: entity.updated_at.into(),
//                 }
//             })
//             .collect();

//         Ok(posts)
//     }

//     /// 按标签获取帖子列表
//     pub async fn get_posts_by_tag(
//         &self,
//         tag: &str,
//         page: i32,
//         page_size: i32,
//     ) -> Result<Vec<Post>, ServiceError> {
//         let offset = (page - 1) * page_size;

//         // 使用rbatis执行原生SQL查询
//         let sql = format!(
//             r#"
//             SELECT p.* FROM posts p
//             WHERE p.tags @> ARRAY[$1]::text[]
//             ORDER BY p.created_at DESC
//             LIMIT {} OFFSET {}
//         "#,
//             page_size, offset
//         );

//         let params = vec![rbs::to_value(tag)];
//         let post_entities: Vec<PostEntity> = self
//             .db
//             .fetch_by_sql_with(&sql, params)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         // 将实体转换为Post模型
//         let posts = post_entities
//             .into_iter()
//             .map(|entity| {
//                 Post {
//                     id: uuid::Uuid::new_v4(), // 这里需要根据实际情况调整
//                     user_id: entity.user_id,
//                     content: entity.content,
//                     images_ipfs_cids: entity.images_ipfs_cids,
//                     arweave_tx_id: entity.arweave_tx_id,
//                     transaction_hash: entity.transaction_hash,
//                     transaction_chain: entity.transaction_chain,
//                     like_count: entity.like_count,
//                     comment_count: entity.comment_count,
//                     tags: entity.tags,
//                     created_at: entity.created_at.into(),
//                     updated_at: entity.updated_at.into(),
//                 }
//             })
//             .collect();

//         Ok(posts)
//     }

//     /// 获取用户帖子列表
//     pub async fn get_user_posts(
//         &self,
//         user_id: uuid::Uuid,
//         page: i32,
//         page_size: i32,
//     ) -> Result<Vec<Post>, ServiceError> {
//         let offset = (page - 1) * page_size;

//         // 使用rbatis查询用户帖子
//         let wrapper = self
//             .db
//             .new_wrapper()
//             .eq("user_id", user_id)
//             .order_by(false, &["created_at"]);

//         let page_result = self
//             .db
//             .fetch_page_by_wrapper::<PostEntity>(wrapper, page as u64, page_size as u64)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         // 将实体转换为Post模型
//         let posts = page_result
//             .records
//             .into_iter()
//             .map(|entity| Post {
//                 id: entity.id,
//                 user_id: entity.user_id,
//                 content: entity.content,
//                 images_ipfs_cids: entity.images_ipfs_cids,
//                 arweave_tx_id: entity.arweave_tx_id,
//                 transaction_hash: entity.transaction_hash,
//                 transaction_chain: entity.transaction_chain,
//                 like_count: entity.like_count,
//                 comment_count: entity.comment_count,
//                 tags: entity.tags,
//                 created_at: entity.created_at.into(),
//                 updated_at: entity.updated_at.into(),
//             })
//             .collect();

//         Ok(posts)
//     }

//     /// 获取帖子详情
//     pub async fn get_post(&self, post_id: i32) -> Result<Post, ServiceError> {
//         // 使用rbatis查询帖子
//         let wrapper = self.db.new_wrapper().eq("id", post_id);

//         let entity = self
//             .db
//             .fetch_by_wrapper::<PostEntity>(wrapper)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?
//             .ok_or(ServiceError::NotFound("帖子不存在".into()))?;

//         // 将实体转换为Post模型
//         let post = Post {
//             id: entity.id,
//             user_id: entity.user_id,
//             content: entity.content,
//             images_ipfs_cids: entity.images_ipfs_cids,
//             arweave_tx_id: entity.arweave_tx_id,
//             transaction_hash: entity.transaction_hash,
//             transaction_chain: entity.transaction_chain,
//             like_count: entity.like_count,
//             comment_count: entity.comment_count,
//             tags: entity.tags,
//             created_at: entity.created_at.into(),
//             updated_at: entity.updated_at.into(),
//         };

//         Ok(post)
//     }

//     /// 创建评论
//     pub async fn create_comment(
//         &self,
//         user_id: String,
//         post_id: String,
//         content: &str,
//         parent_id: Option<String>,
//     ) -> Result<Comment, ServiceError> {
//         // 验证帖子是否存在
//         let post_wrapper = self.db.new_wrapper().eq("id", post_id);

//         let post_exists = self
//             .db
//             .fetch_count_by_wrapper::<PostEntity>(post_wrapper)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?
//             > 0;

//         if !post_exists {
//             return Err(ServiceError::NotFound("帖子不存在".into()));
//         }

//         // 如果是回复评论，验证父评论是否存在
//         if let Some(parent_id_val) = parent_id {
//             let comment_wrapper = self.db.new_wrapper().eq("id", parent_id_val);

//             let comment_exists = self
//                 .db
//                 .fetch_count_by_wrapper::<CommentEntity>(comment_wrapper)
//                 .await
//                 .map_err(|_| ServiceError::InternalServerError)?
//                 > 0;

//             if !comment_exists {
//                 return Err(ServiceError::NotFound("父评论不存在".into()));
//             }
//         }

//         // 将内容存储到Arweave
//         let content_id = self
//             .storage_service
//             .upload_to_arweave(content.as_bytes())
//             .await?;

//         // 创建评论实体
//         let comment_id = uuid::Uuid::new_v4();
//         let comment_entity = CommentEntity {
//             id: comment_id,
//             post_id,
//             user_id,
//             parent_id,
//             content: content.to_string(),
//             arweave_tx_id: Some(content_id),
//             like_count: 0,
//             created_at: DateTime::now(),
//             updated_at: DateTime::now(),
//         };

//         // 保存评论
//         self.db
//             .save(&comment_entity, &[])
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         // 将实体转换为Comment模型
//         let comment = Comment {
//             id: comment_id,
//             post_id,
//             user_id,
//             parent_id,
//             content: content.to_string(),
//             arweave_tx_id: Some(content_id),
//             like_count: 0,
//             created_at: chrono::Utc::now(),
//             updated_at: chrono::Utc::now(),
//         };

//         Ok(comment)
//     }

//     /// 获取评论列表
//     pub async fn get_comments(
//         &self,
//         post_id: uuid::Uuid,
//         page: i32,
//         page_size: i32,
//     ) -> Result<Vec<Comment>, ServiceError> {
//         // 使用rbatis查询评论
//         let wrapper = self
//             .db
//             .new_wrapper()
//             .eq("post_id", post_id)
//             .is_null("parent_id")
//             .order_by(true, &["created_at"]);

//         let page_result = self
//             .db
//             .fetch_page_by_wrapper::<CommentEntity>(wrapper, page as u64, page_size as u64)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         // 将实体转换为Comment模型
//         let comments = page_result
//             .records
//             .into_iter()
//             .map(|entity| Comment {
//                 id: entity.id,
//                 post_id: entity.post_id,
//                 user_id: entity.user_id,
//                 parent_id: entity.parent_id,
//                 content: entity.content,
//                 arweave_tx_id: entity.arweave_tx_id,
//                 like_count: entity.like_count,
//                 created_at: entity.created_at.into(),
//                 updated_at: entity.updated_at.into(),
//             })
//             .collect();

//         Ok(comments)
//     }

//     /// 获取评论回复
//     pub async fn get_comment_replies(
//         &self,
//         comment_id: uuid::Uuid,
//         page: i32,
//         page_size: i32,
//     ) -> Result<Vec<Comment>, ServiceError> {
//         // 使用rbatis查询评论回复
//         let wrapper = self
//             .db
//             .new_wrapper()
//             .eq("parent_id", comment_id)
//             .order_by(true, &["created_at"]);

//         let page_result = self
//             .db
//             .fetch_page_by_wrapper::<CommentEntity>(wrapper, page as u64, page_size as u64)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         // 将实体转换为Comment模型
//         let replies = page_result
//             .records
//             .into_iter()
//             .map(|entity| Comment {
//                 id: entity.id,
//                 post_id: entity.post_id,
//                 user_id: entity.user_id,
//                 parent_id: entity.parent_id,
//                 content: entity.content,
//                 arweave_tx_id: entity.arweave_tx_id,
//                 like_count: entity.like_count,
//                 created_at: entity.created_at.into(),
//                 updated_at: entity.updated_at.into(),
//             })
//             .collect();

//         Ok(replies)
//     }

//     /// 获取评论详情
//     pub async fn get_comment(&self, comment_id: uuid::Uuid) -> Result<Comment, ServiceError> {
//         // 使用rbatis查询评论
//         let wrapper = self.db.new_wrapper().eq("id", comment_id);

//         let entity = self
//             .db
//             .fetch_by_wrapper::<CommentEntity>(wrapper)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?
//             .ok_or(ServiceError::NotFound("评论不存在".into()))?;

//         // 将实体转换为Comment模型
//         let comment = Comment {
//             id: entity.id,
//             post_id: entity.post_id,
//             user_id: entity.user_id,
//             parent_id: entity.parent_id,
//             content: entity.content,
//             arweave_tx_id: entity.arweave_tx_id,
//             like_count: entity.like_count,
//             created_at: entity.created_at.into(),
//             updated_at: entity.updated_at.into(),
//         };

//         Ok(comment)
//     }

//     /// 点赞帖子
//     pub async fn like_post(
//         &self,
//         user_id: uuid::Uuid,
//         post_id: uuid::Uuid,
//     ) -> Result<(), ServiceError> {
//         // 验证帖子是否存在
//         let post_wrapper: bool = self.db.new_wrapper().eq("id", post_id);

//         let post_exists: bool = self
//             .db
//             .fetch_count_by_wrapper::<PostEntity>(post_wrapper)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?
//             > 0;

//         if !post_exists {
//             return Err(ServiceError::NotFound("帖子不存在".into()));
//         }

//         // 检查是否已点赞
//         let like_wrapper = self
//             .db
//             .new_wrapper()
//             .eq("user_id", user_id);

//         let exists = self
//             .db
//             .fetch_count_by_wrapper::<UserLikeEntity>(like_wrapper)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?
//             > 0;

//         if exists {
//             return Err(ServiceError::BadRequest("已经点赞过该帖子".into()));
//         }

//         // 创建点赞记录
//         let like_entity = UserLikeEntity {
//             id: uuid::Uuid::new_v4(),
//             user_id,
//             post_id: Some(post_id),
//             comment_id: None,
//             created_at: DateTime::now(),
//         };

//         self.db
//             .save(&like_entity, &[])
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         Ok(())
//     }

//     /// 取消点赞
//     pub async fn unlike_post(
//         &self,
//         user_id: uuid::Uuid,
//         post_id: uuid::Uuid,
//     ) -> Result<(), ServiceError> {
//         // 使用rbatis删除点赞记录
//         let wrapper = self
//             .db
//             .new_wrapper()
//             .eq("user_id", user_id);

//         self.db
//             .remove_by_wrapper::<UserLikeEntity>(wrapper)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         Ok(())
//     }

//     /// 获取帖子点赞数
//     pub async fn get_post_likes_count(&self, post_id: uuid::Uuid) -> Result<i64, ServiceError> {
//         // 使用rbatis查询点赞数
//         let wrapper = self.db.new_wrapper().eq("post_id", post_id);

//         let count = self
//             .db
//             .fetch_count_by_wrapper::<UserLikeEntity>(wrapper)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         Ok(count)
//     }

//     /// 检查用户是否已点赞
//     pub async fn has_user_liked(
//         &self,
//         user_id: uuid::Uuid,
//         post_id: uuid::Uuid,
//     ) -> Result<bool, ServiceError> {
//         // 使用rbatis查询是否已点赞
//         let wrapper = self
//             .db
//             .new_wrapper()
//             .eq("user_id", user_id);

//         let count = self
//             .db
//             .fetch_count_by_wrapper::<UserLikeEntity>(wrapper)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         Ok(count > 0)
//     }

//     /// 获取热门标签
//     pub async fn get_hot_tags(&self, limit: i32) -> Result<Vec<TagEntity>, ServiceError> {
//         // 使用rbatis执行原生SQL查询获取热门标签
//         let sql = format!(
//             r#"
//             SELECT t.id, t.name, COUNT(pt.post_id) as post_count
//             FROM tags t
//             JOIN post_tags pt ON t.id = pt.tag_id
//             GROUP BY t.id, t.name
//             ORDER BY COUNT(pt.post_id) DESC
//             LIMIT {}
//         "#,
//             limit
//         );

//         let tags: Vec<TagEntity> = self
//             .db
//             .fetch_by_sql(&sql)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         Ok(tags)
//     }

//     /// 搜索帖子
//     pub async fn search_posts(
//         &self,
//         query: &str,
//         page: i32,
//         page_size: i32,
//     ) -> Result<Vec<Post>, ServiceError> {
//         let offset = (page - 1) * page_size;
//         let search_term = format!("%{}%", query);

//         // 使用rbatis执行原生SQL查询搜索帖子
//         let sql = format!(
//             r#"
//             SELECT * FROM posts
//             WHERE content ILIKE '{}'
//             ORDER BY created_at DESC
//             LIMIT {} OFFSET {}
//         "#,
//             search_term, page_size, offset
//         );

//         let post_entities: Vec<PostEntity> = self
//             .db
//             .fetch_by_sql(&sql)
//             .await
//             .map_err(|_| ServiceError::InternalServerError)?;

//         // 将实体转换为Post模型
//         let posts = post_entities
//             .into_iter()
//             .map(|entity| Post {
//                 id: entity.id,
//                 user_id: entity.user_id,
//                 content: entity.content,
//                 images_ipfs_cids: entity.images_ipfs_cids,
//                 arweave_tx_id: entity.arweave_tx_id,
//                 transaction_hash: entity.transaction_hash,
//                 transaction_chain: entity.transaction_chain,
//                 like_count: entity.like_count,
//                 comment_count: entity.comment_count,
//                 tags: entity.tags,
//                 created_at: entity.created_at.into(),
//                 updated_at: entity.updated_at.into(),
//             })
//             .collect();

//         Ok(posts)
//     }
// }
