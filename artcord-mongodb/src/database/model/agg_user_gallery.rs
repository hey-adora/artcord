use crate::database::{COLLECTION_USER_NAME, DB};
use artcord_state::global::{AggImg, AggImgFieldName, DbImgFieldName, DbUserFieldName};
use bson::doc;
use chrono::Utc;
use field_types::FieldName;
use futures::TryStreamExt;

impl DB {
    pub async fn img_aggregate_user_gallery(
        &self,
        amount: u32,
        from: i64,
        user_id: &str,
    ) -> Result<Option<Vec<AggImg>>, mongodb::error::Error> {
        let user = self
            .collection_user
            .find_one(doc! {DbUserFieldName::AuthorId.name(): user_id}, None)
            .await?;
        if let None = user {
            return Ok(None);
        }

        let pipeline = vec![
            doc! { "$sort": doc! { DbImgFieldName::CreatedAt.name(): -1 } },
            doc! { "$match": doc! { DbImgFieldName::CreatedAt.name(): { "$lt": from }, DbImgFieldName::Show.name(): true, DbImgFieldName::UserId.name(): user_id } },
            doc! { "$limit": Some( amount.clamp(25, 10000) as i64) },
            doc! { "$lookup": doc! { "from": COLLECTION_USER_NAME, "localField": DbImgFieldName::UserId.name(), "foreignField": DbUserFieldName::AuthorId.name(), "as": AggImgFieldName::User.name()} },
            doc! { "$unwind": format!("${}", AggImgFieldName::User.name()) },
        ];

        let imgs = self.collection_img.aggregate(pipeline, None).await?;
        let imgs = imgs.try_collect().await.unwrap_or_else(|_| vec![]);

        let mut send_this: Vec<AggImg> = Vec::new();

        for img in imgs {
            let doc: AggImg = mongodb::bson::from_document(img)?;
            send_this.push(doc);
        }
        
        Ok(Some(send_this))
    }
}
