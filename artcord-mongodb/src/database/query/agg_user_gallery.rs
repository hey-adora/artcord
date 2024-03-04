use artcord_state::{aggregation::server_msg_img::{AggImg, AggImgFieldName}, model::{img::ImgFieldName, user::UserFieldName}};
use bson::doc;
use futures::TryStreamExt;
use crate::database::query::user::COLLECTION_USER_NAME;

use crate::database::DB;

impl DB {
    pub async fn img_aggregate_user_gallery(
        &self,
        amount: u32,
        from: i64,
        user_id: &str,
    ) -> Result<Option<Vec<AggImg>>, mongodb::error::Error> {
        let user = self
            .collection_user
            .find_one(doc! {UserFieldName::Id.name(): user_id}, None)
            .await?;
        if let None = user {
            return Ok(None);
        }

        let pipeline = vec![
            doc! { "$sort": doc! { ImgFieldName::CreatedAt.name(): -1 } },
            doc! { "$match": doc! { ImgFieldName::CreatedAt.name(): { "$lt": from }, ImgFieldName::Show.name(): true, ImgFieldName::UserId.name(): user_id } },
            doc! { "$limit": Some( amount.clamp(25, 10000) as i64) },
            doc! { "$lookup": doc! { "from": COLLECTION_USER_NAME, "localField": ImgFieldName::UserId.name(), "foreignField": UserFieldName::Id.name(), "as": AggImgFieldName::User.name()} },
            doc! { "$unwind": format!("${}", AggImgFieldName::User.name()) },
        ];
        // println!("{:#?}", pipeline);

        let imgs = self.collection_img.aggregate(pipeline, None).await?;
        let imgs = imgs.try_collect().await.unwrap_or_else(|_| vec![]);

        let mut send_this: Vec<AggImg> = Vec::new();

        for img in imgs {
            let doc: AggImg = mongodb::bson::from_document(img)?;
            send_this.push(doc);
        }

        // while let Some(result) = imgs.try_next().await? {
        //     let doc: ServerMsgImg = mongodb::bson::from_document(result)?;
        //     send_this.push(doc);
        // }

        //println!("Len: {}", send_this.len());

        Ok(Some(send_this))
    }
}