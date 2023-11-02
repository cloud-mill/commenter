use axum::{Extension, Json};
use chrono::Utc;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, instrument};
use uuid::Uuid;

use crate::{
    common::{
        errors::ServerError,
        utils::{append_uuid_to_materialized_path, uuid_list_to_materialized_path},
    },
    models::{Comment, CommentReaction, CommentReactor, CommentType, Commenter},
    persistent::PersistentLayer,
};

#[derive(Deserialize, Clone, Debug)]
pub struct CreateRootCommentRequest {
    pub resource_id: Uuid,
    pub commenter_account_id: Uuid,
    pub commenter_username: String,
    pub comment_text: String,
}

#[instrument(level = "trace")]
pub async fn create_root_comment(
    Extension(persistent_layer): Extension<Arc<PersistentLayer>>,
    Json(payload): Json<CreateRootCommentRequest>,
) -> Result<String, ServerError> {
    debug!(payload = ?payload);

    let comment_id = Uuid::new_v4();

    let comment = Comment {
        comment_id,
        comment_type: CommentType::Root,
        commenter: Commenter {
            account_id: payload.commenter_account_id,
            username: payload.commenter_username,
        },
        commented_timestamp: Utc::now(),
        comment_text: payload.comment_text,
        reactions: vec![],
        branch_comment_ids: vec![],
        materialized_path: uuid_list_to_materialized_path(&[payload.resource_id, comment_id]),
    };

    if (persistent_layer.insert_comment_mongo(comment).await).is_err() {
        return Err(ServerError::internal_server_error());
    }

    Ok(json!({ "comment_id": comment_id }).to_string())
}

#[derive(Deserialize, Clone, Debug)]
pub struct CreateBranchCommentRequest {
    pub branched_from: Uuid,
    pub commenter_account_id: Uuid,
    pub commenter_username: String,
    pub comment_text: String,
}

#[instrument(level = "trace")]
pub async fn create_branch_comment(
    Extension(persistent_layer): Extension<Arc<PersistentLayer>>,
    Json(payload): Json<CreateBranchCommentRequest>,
) -> Result<String, ServerError> {
    debug!(payload = ?payload);

    let branched_from_comment = persistent_layer
        .find_comment(payload.branched_from)
        .await
        .map_err(|_| ServerError::forbidden_error())?;

    let comment_id = Uuid::new_v4();

    let comment = Comment {
        comment_id,
        comment_type: CommentType::Branch,
        commenter: Commenter {
            account_id: payload.commenter_account_id,
            username: payload.commenter_username,
        },
        commented_timestamp: Utc::now(),
        comment_text: payload.comment_text,
        reactions: vec![],
        branch_comment_ids: vec![],
        materialized_path: append_uuid_to_materialized_path(
            &branched_from_comment.materialized_path,
            &comment_id,
        ),
    };

    if (persistent_layer.insert_comment_mongo(comment).await).is_err() {
        return Err(ServerError::internal_server_error());
    }

    Ok(json!({ "comment_id": comment_id }).to_string())
}

#[derive(Deserialize, Clone, Debug)]
pub struct ReactToCommentRequest {
    pub reactor_account_id: Uuid,
    pub reactor_username: String,
    pub emoji_unicode: String,
    pub reacted_comment_id: Uuid,
}

#[instrument(level = "trace")]
pub async fn react_to_comment(
    Extension(persistent_layer): Extension<Arc<PersistentLayer>>,
    Json(payload): Json<ReactToCommentRequest>,
) -> Result<(), ServerError> {
    debug!(payload = ?payload);

    let comment_reaction = CommentReaction {
        reactor: CommentReactor {
            account_id: payload.reactor_account_id,
            username: payload.reactor_username,
        },
        emoji_unified_code: payload.emoji_unicode,
    };

    if (persistent_layer
        .append_reaction_to_comment_mongo(payload.reacted_comment_id, comment_reaction)
        .await)
        .is_err()
    {
        return Err(ServerError::internal_server_error());
    }

    Ok(())
}

#[derive(Deserialize, Clone, Debug)]
pub struct UndoReactToCommentRequest {
    pub reactor_account_id: Uuid,
    pub reactor_username: String,
    pub emoji_unicode: String,
    pub reacted_comment_id: Uuid,
}

#[instrument(level = "trace")]
pub async fn undo_react_to_comment(
    Extension(persistent_layer): Extension<Arc<PersistentLayer>>,
    Json(payload): Json<UndoReactToCommentRequest>,
) -> Result<(), ServerError> {
    debug!(payload = ?payload);

    let comment_reaction = CommentReaction {
        reactor: CommentReactor {
            account_id: payload.reactor_account_id,
            username: payload.reactor_username,
        },
        emoji_unified_code: payload.emoji_unicode,
    };

    if (persistent_layer
        .remove_reaction_from_comment_mongo(payload.reacted_comment_id, comment_reaction)
        .await)
        .is_err()
    {
        return Err(ServerError::internal_server_error());
    }

    Ok(())
}

#[derive(Deserialize, Clone, Debug)]
pub struct UpdateCommentTextRequest {
    pub comment_id: Uuid,
    pub new_comment_text: String,
}

#[instrument(level = "trace")]
pub async fn update_comment_text(
    Extension(persistent_layer): Extension<Arc<PersistentLayer>>,
    Json(payload): Json<UpdateCommentTextRequest>,
) -> Result<(), ServerError> {
    debug!(payload = ?payload);

    // find the comment to be updated
    let mut comment = persistent_layer
        .find_comment(payload.comment_id)
        .await
        .map_err(|_| ServerError::forbidden_error())?;

    // new comment text
    comment.comment_text = payload.new_comment_text;

    // put it back
    if (persistent_layer
        .update_comment_mongo(payload.comment_id, comment)
        .await)
        .is_err()
    {
        return Err(ServerError::internal_server_error());
    }

    Ok(())
}

#[derive(Deserialize, Clone, Debug)]
pub struct DeleteCommentRequest {
    pub comment_id: Uuid,
}

#[instrument(level = "trace")]
pub async fn delete_comment(
    Extension(persistent_layer): Extension<Arc<PersistentLayer>>,
    Json(payload): Json<DeleteCommentRequest>,
) -> Result<(), ServerError> {
    debug!(payload = ?payload);

    // find the comment to be deleted
    let comment = persistent_layer
        .find_comment(payload.comment_id)
        .await
        .map_err(|_| ServerError::forbidden_error())?;

    // prune comments by the materialized path
    if (persistent_layer
        .prune_comments_mongo(comment.materialized_path)
        .await)
        .is_err()
    {
        return Err(ServerError::internal_server_error());
    }

    Ok(())
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetRootCommentsRequest {
    pub resource_id: Uuid,
}

#[instrument(level = "trace")]
pub async fn get_root_comments(
    Extension(persistent_layer): Extension<Arc<PersistentLayer>>,
    Json(payload): Json<GetRootCommentsRequest>,
) -> Result<String, ServerError> {
    debug!(payload = ?payload);

    let root_comments = persistent_layer
        .find_next_level_comments(payload.resource_id.to_string())
        .await
        .map_err(|_| ServerError::internal_server_error())?;

    Ok(json!({ "root_comments": root_comments }).to_string())
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetBranchCommentsNextRequest {
    pub branched_from: Uuid,
}

#[instrument(level = "trace")]
pub async fn get_branch_comments_next(
    Extension(persistent_layer): Extension<Arc<PersistentLayer>>,
    Json(payload): Json<GetBranchCommentsNextRequest>,
) -> Result<String, ServerError> {
    debug!(payload = ?payload);

    // find its root comment
    let root_comment = persistent_layer
        .find_comment(payload.branched_from)
        .await
        .map_err(|_| ServerError::forbidden_error())?;

    // find branch comments by materialized path
    let branch_comments = persistent_layer
        .find_next_level_comments(root_comment.materialized_path)
        .await
        .map_err(|_| ServerError::internal_server_error())?;

    Ok(json!({ "branch_comments": branch_comments }).to_string())
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetRestBranchCommentsRequest {
    pub branched_from: Uuid,
}

#[instrument(level = "trace")]
pub async fn get_branch_comments_rest(
    Extension(persistent_layer): Extension<Arc<PersistentLayer>>,
    Json(payload): Json<GetRestBranchCommentsRequest>,
) -> Result<String, ServerError> {
    debug!(payload = ?payload);

    // find its root comment
    let branched_from_comment = persistent_layer
        .find_comment(payload.branched_from)
        .await
        .map_err(|_| ServerError::forbidden_error())?;

    // find branch comments by materialized path
    let branch_comments = persistent_layer
        .find_all_comments(branched_from_comment.materialized_path)
        .await
        .map_err(|_| ServerError::internal_server_error())?;

    Ok(json!({ "branch_comments": branch_comments }).to_string())
}

#[derive(Deserialize, Clone, Debug)]
pub struct GetAllCommentsRequest {
    pub resource_id: Uuid,
}

#[instrument(level = "trace")]
pub async fn get_all_comments(
    Extension(persistent_layer): Extension<Arc<PersistentLayer>>,
    Json(payload): Json<GetAllCommentsRequest>,
) -> Result<String, ServerError> {
    debug!(payload = ?payload);

    let all_comments = persistent_layer
        .find_all_comments(payload.resource_id.to_string())
        .await
        .map_err(|_| ServerError::internal_server_error())?;

    Ok(json!({ "comments": all_comments }).to_string())
}
