use crate::database::query::user::COLLECTION_USER_NAME;
use crate::database::DB;
use artcord_state::{
    aggregation::server_msg_img::{AggImg, AggImgFieldName},
    model::{img::ImgFieldName, user::UserFieldName},
};
use bson::doc;
use futures::TryStreamExt;

impl DB {
    pub async fn img_aggregate_gallery(
        &self,
        amount: u32,
        from: i64,
    ) -> Result<Vec<AggImg>, mongodb::error::Error> {
        let pipeline = vec![
            doc! { "$sort": doc! { ImgFieldName::CreatedAt.name(): -1 } },
            doc! { "$match": doc! { ImgFieldName::CreatedAt.name(): { "$lt": from }, ImgFieldName::Show.name(): true } },
            doc! { "$limit": Some( amount.clamp(25, 10000) as i64) },
            doc! { "$lookup": doc! { "from": COLLECTION_USER_NAME, "localField": ImgFieldName::UserId.name(), "foreignField": UserFieldName::AuthorId.name(), "as": AggImgFieldName::User.name()} },
            doc! { "$unwind": format!("${}", AggImgFieldName::User.name()) },
        ];
        // println!("{:#?}", pipeline);

        let mut imgs = self.collection_img.aggregate(pipeline, None).await?;

        let mut send_this: Vec<AggImg> = Vec::new();

        while let Some(result) = imgs.try_next().await? {
            let doc: AggImg = mongodb::bson::from_document(result)?;
            //let a = doc.f
            send_this.push(doc);
            // println!("hh");
        }

        Ok(send_this)
    }
}

