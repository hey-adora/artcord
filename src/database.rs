use mongodb::bson::{Binary, doc};
use mongodb::options::{DeleteOptions, FindOptions};
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use serenity::prelude::TypeMapKey;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Img {
    pub user_id: u64,
    pub org_hash: Binary,
    pub format: u8,
    pub has_high: bool,
    pub has_medium: bool,
    pub has_low: bool,
    pub modified_at: mongodb::bson::DateTime,
    pub created_at: mongodb::bson::DateTime,
}

pub enum ImgFormat {
    PNG,
    JPG,
}

impl ImgFormat {
    fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(ImgFormat::PNG),
            1 => Some(ImgFormat::JPG),
            _ => None
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        match value {
            "png" => Some(ImgFormat::PNG),
            "jpg" => Some(ImgFormat::JPG),
            _ => None
        }
    }

    fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(ImgFormat::PNG),
            1 => Some(ImgFormat::JPG),
            _ => None
        }
    }
}


impl Into<u8> for ImgFormat {
    fn into(self) -> u8 {
        match self {
            ImgFormat::PNG => 0,
            ImgFormat::JPG => 1,
        }
    }
}

impl Into<&str> for &ImgFormat {
    fn into(self) -> &'static str {
        match self {
            ImgFormat::PNG => "png",
            ImgFormat::JPG => "jpg",
        }
    }
}

impl Display for ImgFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let a: &str = self.into();
        write!(f, "{}", a)
    }
}


#[derive(Clone, Debug)]
pub struct DB {
    pub client: mongodb::Client,
    pub database: mongodb::Database,
    pub collection_img: mongodb::Collection<Img>
}

impl TypeMapKey for DB {
    type Value = Self;
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


// 
// impl Default for Img {
//     fn default() -> Self {
//         Img {
//             url: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
//         }
//     }
// }

pub async fn create_database() -> DB {
    let mut client_options = ClientOptions::parse("mongodb://root:example@localhost:27017")
        .await
        .unwrap();
    client_options.app_name = Some("My App".to_string());
    let client = Client::with_options(client_options).unwrap();

    let database = client.database("artcord");
    let collection_img = database.collection::<Img>("img");

    //let test
    //collection_img.insert_one()

    println!("Connecting to database...");
    let db_list = client.list_database_names(doc! {}, None).await.unwrap();
    println!("Databases: {:?}", db_list);


    DB {
        database,
        client,
        collection_img
    }
}
