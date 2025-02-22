use crate::database::{COLLECTION_USER_NAME, DB};
use artcord_state::global::{AggImg, AggImgFieldName, DbImgFieldName, DbUserFieldName};
use bson::doc;
use futures::TryStreamExt;
use field_types::FieldName;

impl DB {
    pub async fn img_aggregate_gallery(
        &self,
        amount: u32,
        from: i64,
    ) -> Result<Vec<AggImg>, mongodb::error::Error> {
        let pipeline = vec![
            doc! { "$sort": doc! { DbImgFieldName::CreatedAt.name(): -1 } },
            doc! { "$match": doc! { DbImgFieldName::CreatedAt.name(): { "$lt": from }, DbImgFieldName::Show.name(): true } },
            doc! { "$limit": Some( amount.clamp(25, 10000) as i64) },
            doc! { "$lookup": doc! { "from": COLLECTION_USER_NAME, "localField": DbImgFieldName::UserId.name(), "foreignField": DbUserFieldName::AuthorId.name(), "as": AggImgFieldName::User.name()} },
            doc! { "$unwind": format!("${}", AggImgFieldName::User.name()) },
        ];

        let mut imgs = self.collection_img.aggregate(pipeline, None).await?;

        let mut send_this: Vec<AggImg> = Vec::new();

        while let Some(result) = imgs.try_next().await? {
            let doc: AggImg = mongodb::bson::from_document(result)?;
            send_this.push(doc);
        }

        Ok(send_this)
    }
}