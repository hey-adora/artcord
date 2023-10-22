use mongodb::bson::doc;
use mongodb::options::{DeleteOptions, FindOptions};
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Img {
    url: String,
}

#[derive(Clone)]
pub struct DB {
    pub client: mongodb::Client,
    pub database: mongodb::Database,
    pub collection_img: mongodb::Collection<Img>
}

// pub struct Item<T> {
//     collection: mongodb::Collection<T>
// }
//
// impl <T> Item<T> {
//     pub async fn add(&self, item: &T) {
//         self.collection.insert_one(&item, None).await.unwrap();
//     }
// }
//
//
// pub struct DB {
//     database: mongodb::Database,
//     item_img: Item<Img>
// }
//
// impl DB {
//     pub async fn new() -> Self {
//         let mut client_options = ClientOptions::parse("mongodb://root:example@localhost:27017")
//             .await
//             .unwrap();
//         client_options.app_name = Some("My App".to_string());
//         let client = Client::with_options(client_options).unwrap();
//         let database = client.database("duck");
//         let collection_img = database.collection::<Img>("duck");
//         let item_img = Item {
//             collection: collection_img
//         };
//         // let img = Img::default();
//         //
//         // use std::time::Instant;
//         // let now = Instant::now();
//         //
//         // for i in 0..1000 {
//         //     collection.insert_one(&img, None).await.unwrap();
//         // }
//         //
//         // let elapsed = now.elapsed();
//         // println!("Elapsed: {:.2?}", elapsed);
//         //
//         // let filter = doc! {};
//         // let opts: DeleteOptions = DeleteOptions::builder().build();
//         // collection.delete_many(filter, opts).await.unwrap();
//
//         Self {
//             database,
//             item_img
//         }
//     }
//
//     fn collection
// }



impl Default for Img {
    fn default() -> Self {
        Img {
            url: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        }
    }
}

pub async fn create_database() -> DB {
    let mut client_options = ClientOptions::parse("mongodb://root:example@localhost:27017")
        .await
        .unwrap();
    client_options.app_name = Some("My App".to_string());
    let client = Client::with_options(client_options).unwrap();
    let database = client.database("artcord");
    let collection_img = database.collection::<Img>("img");

    DB {
        database,
        client,
        collection_img
    }
}
