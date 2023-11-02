use anyhow::Result;
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{
        self, doc,
        Bson::{self, Null},
        Document,
    },
    options::FindOptions,
    Collection,
};
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{
    models::{Comment, CommentReaction},
    persistent::PersistentLayer,
};

impl PersistentLayer {
    #[instrument(level = "trace", skip_all)]
    pub async fn insert_comment_mongo(&self, comment: Comment) -> Result<()> {
        let db = self.mongo_client.database(&self.mongo_config.mongo_db_name);
        let comments_collection: Collection<Document> = db.collection("comments");

        let bson_comment = bson::to_bson(&comment)?
            .as_document()
            .ok_or(anyhow::anyhow!(
                "bson_resource: failed to convert BSON to document"
            ))?
            .clone();

        let insert_result = comments_collection.insert_one(bson_comment, None).await?;

        if let Null = insert_result.inserted_id {
            return Err(anyhow::anyhow!("error inserting the document"));
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn append_reaction_to_comment_mongo(
        &self,
        comment_id: Uuid,
        comment_reaction_to_append: CommentReaction,
    ) -> Result<()> {
        let db = self.mongo_client.database(&self.mongo_config.mongo_db_name);
        let comments_collection: Collection<Comment> = db.collection("comments");

        let update_filter = doc! {
            "comment_id": bson::to_bson(&comment_id)?
        };

        let update = doc! {
            "$push": {
                "reactions": bson::to_bson(&comment_reaction_to_append)?,
            }
        };

        let update_result = comments_collection
            .update_one(update_filter, update, None)
            .await?;

        if update_result.modified_count == 0 {
            return Err(anyhow::anyhow!(
                "error updating (appending reaction to) comment documents"
            ));
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn remove_reaction_from_comment_mongo(
        &self,
        comment_id: Uuid,
        comment_reaction_to_remove: CommentReaction,
    ) -> Result<()> {
        let db = self.mongo_client.database(&self.mongo_config.mongo_db_name);
        let comments_collection: Collection<Comment> = db.collection("comments");

        let update_filter = doc! {
            "comment_id": bson::to_bson(&comment_id)?
        };

        let update = doc! {
            "$pull": {
                "reactions": bson::to_bson(&comment_reaction_to_remove)?,
            }
        };

        let update_result = comments_collection
            .update_one(update_filter, update, None)
            .await?;

        if update_result.modified_count == 0 {
            return Err(anyhow::anyhow!(
                "error updating (removing reaction from) comment documents"
            ));
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn update_comment_mongo<T>(&self, comment_id: Uuid, new_comment: T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let db = self.mongo_client.database(&self.mongo_config.mongo_db_name);
        let comments_collection: Collection<Comment> = db.collection("comments");

        let update_filter = doc! {
            "comment_id": bson::to_bson(&comment_id)?
        };

        let update = doc! {
            "$set": bson::to_bson(&new_comment)?
        };

        let update_result = comments_collection
            .update_one(update_filter, update, None)
            .await?;

        if update_result.modified_count == 0 {
            info!("no comment documentations updated")
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn prune_comments_mongo(&self, comment_materialized_path: String) -> Result<()> {
        let db = self.mongo_client.database(&self.mongo_config.mongo_db_name);
        let comments_collection: Collection<Comment> = db.collection("comments");

        let regex_pattern = format!("^{}", comment_materialized_path); // starts with the given path
        let filter = doc! {
            "materialized_path": {
                "$regex": regex_pattern
            }
        };

        let delete_result = comments_collection.delete_many(filter, None).await?;

        if delete_result.deleted_count == 0 {
            info!("no comment documentations deleted");
        }

        Ok(())
    }

    pub async fn find_comment(&self, comment_id: Uuid) -> Result<Comment> {
        let db = self.mongo_client.database(&self.mongo_config.mongo_db_name);
        let comments_collection: Collection<Document> = db.collection("comments");

        let filter = doc! {
            "comment_id": bson::to_bson(&comment_id)?
        };

        if let Some(document) = comments_collection.find_one(filter, None).await? {
            let comment: Comment = bson::from_bson(Bson::Document(document))?;
            Ok(comment)
        } else {
            Err(anyhow::Error::msg("comment not found"))
        }
    }

    pub async fn find_next_level_comments(&self, current_path: String) -> Result<Vec<Comment>> {
        let db = self.mongo_client.database(&self.mongo_config.mongo_db_name);
        let comments_collection: Collection<Document> = db.collection("comments");

        let regex_pattern = format!(
            r"^{}->[0-9a-fA-F]{{8}}-[0-9a-fA-F]{{4}}-[0-9a-fA-F]{{4}}-[0-9a-fA-F]{{4}}-[0-9a-fA-F]{{12}}$", // don't be scared, it's just uuid's regex :)
            current_path
        );

        let filter = doc! {
            "materialized_path": {
                "$regex": regex_pattern
            }
        };

        let find_options = FindOptions::builder()
            .sort(doc! {
                "commented_timestamp": -1  // indicates descending order, latest first
            })
            .build();

        let mut cursor = comments_collection.find(filter, Some(find_options)).await?;

        let mut results: Vec<Comment> = Vec::new();
        while let Some(document) = cursor.try_next().await? {
            let comment: Comment = bson::from_bson(Bson::Document(document))?;
            results.push(comment);
        }

        Ok(results)
    }

    pub async fn find_all_comments(&self, current_path: String) -> Result<Vec<Comment>> {
        let db = self.mongo_client.database(&self.mongo_config.mongo_db_name);
        let comments_collection: Collection<Document> = db.collection("comments");

        let regex_pattern = format!("^{}", current_path);

        let pipeline = vec![
            doc! {
                "$match": {
                    "materialized_path": {
                        "$regex": regex_pattern
                    }
                }
            },
            doc! {
                "$addFields": {
                    "path_length": {
                        "$strLenCP": "$materialized_path"
                    }
                }
            },
            doc! {
                "$sort": {
                    "path_length": 1,  // ascending order
                    "commented_timestamp": -1  // descending order
                }
            },
        ];

        let mut cursor = comments_collection.aggregate(pipeline, None).await?;

        let mut results: Vec<Comment> = Vec::new();
        while let Some(document) = cursor.try_next().await? {
            let comment: Comment = bson::from_bson(Bson::Document(document))?;
            results.push(comment);
        }

        Ok(results)
    }
}
