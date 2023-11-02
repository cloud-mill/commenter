use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommentType {
    Root,
    Branch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub comment_id: Uuid,
    pub comment_type: CommentType,
    pub commenter: Commenter,
    pub commented_timestamp: DateTime<Utc>,
    pub comment_text: String,
    pub reactions: Vec<CommentReaction>,
    pub branch_comment_ids: Vec<Uuid>,
    pub materialized_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commenter {
    pub account_id: Uuid,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentReactor {
    pub account_id: Uuid,
    pub username: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct CommentReaction {
    pub reactor: CommentReactor,
    pub emoji_unified_code: String,
}
