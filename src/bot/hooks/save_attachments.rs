use crate::bot::commands::FEATURE_GALLERY;
use crate::database::{User, DB};
use image::EncodableLayout;
use mongodb::bson::doc;
use serenity::model::channel::Attachment;
use std::fmt::Display;
use std::fs;
use std::io::Cursor;
use std::num::ParseIntError;
use std::path::PathBuf;
use thiserror::Error;

pub async fn hook_save_attachments(
    attachments: &[serenity::model::channel::Attachment],
    db: &DB,
    guild_id: u64,
    channel_id: u64,
    msg_id: u64,
    author_id: u64,
    author_name: String,
    author_avatar: Option<String>,
    force: bool,
) -> Result<(), SaveAttachmentsError> {
    if !force {
        let channel = db
            .collection_allowed_channel
            .find_one(
                doc! { "id": channel_id.to_string(), "feature": FEATURE_GALLERY.to_string() },
                None,
            )
            .await?;
        if let None = channel {
            return Ok(());
        }
    }

    if attachments.len() > 0 {
        let user = save_user(&db, author_name, guild_id, author_id, author_avatar).await?;
        println!(
            "{}",
            match user {
                SaveUserResult::Updated(data) => format!("Updated user: {}", data),
                SaveUserResult::Created => format!("Created user"),
                SaveUserResult::None => format!("User is up to date"),
            }
        );

        for attachment in attachments {
            match save_attachment(&db, guild_id, channel_id, author_id, msg_id, attachment).await {
                Ok(file) => {
                    println!("File: {}", file);
                    Ok::<(), SaveAttachmentsError>(())
                }
                Err(err) => match err {
                    SaveAttachmentError::ImgTypeUnsupported(t) => {
                        println!("Error: img type unsuported: '{}'", t);
                        Ok(())
                    }
                    SaveAttachmentError::ImgTypeNotFound => {
                        println!("Error: img type not found: msg_id: '{}'", msg_id);
                        Ok(())
                    }
                    e => Err(SaveAttachmentsError::from(e)),
                },
            }?;
        }
    }

    Ok(())
}

pub struct ImgData {
    pub bytes: Vec<u8>,
    pub color: webp::PixelLayout,
    pub width: u32,
    pub height: u32,
}

impl ImgData {
    pub fn new(
        org_bytes: &[u8],
        _img_format: image::ImageFormat,
        new_height: u32,
    ) -> Result<ImgData, ImgDataNewError> {
        //let mut img = image::io::Reader::open(file)?.decode()?;
        let mut img: image::DynamicImage = image::io::Reader::new(Cursor::new(org_bytes))
            .with_guessed_format()?
            .decode()?;
        let width = img.width();
        let height = img.height();
        if height <= new_height {
            return Err(ImgDataNewError::ImgTooSmall {
                from: height,
                to: new_height,
            });
        }
        let ratio = width as f32 / height as f32;
        let new_width = (new_height as f32 * ratio) as u32;
        img = img.resize(new_width, new_height, image::imageops::FilterType::Nearest);

        let color = ImgData::webp_color_type(img.color());

        let bytes = if color == webp::PixelLayout::Rgba {
            let rgba = img.to_rgba8();
            rgba.into_raw()
        } else {
            let rgba = img.to_rgb8();
            rgba.into_raw()
        };

        Ok(ImgData {
            bytes,
            color,
            width: new_width,
            height: new_height,
        })
    }

    pub fn encode_webp(&self) -> Result<Vec<u8>, ImgDataEncodeWebpError> {
        let webp_encoder = webp::Encoder::new(&self.bytes, self.color, self.width, self.height);
        let r = webp_encoder
            .encode_simple(false, 10f32)
            .or_else(|e| Err(ImgDataEncodeWebpError::Encode(format!("{:?}", e))))?;
        let bytes: Vec<u8> = r.to_vec();

        Ok(bytes)
    }

    fn webp_color_type(t: image::ColorType) -> webp::PixelLayout {
        match t {
            image::ColorType::Rgba8 => webp::PixelLayout::Rgba,
            image::ColorType::Rgba16 => webp::PixelLayout::Rgba,
            image::ColorType::Rgba32F => webp::PixelLayout::Rgba,
            _ => webp::PixelLayout::Rgb,
        }
    }
}

pub fn save_org_img(_path: &str, bytes: &[u8]) -> Result<(), SaveImgError> {
    let path = PathBuf::from(_path);

    if path.exists() {
        return Err(SaveImgError::AlreadyExist(String::from(_path)));
    }

    fs::write(&path, bytes)?;

    Ok(())
}

pub fn save_webp(
    _path: &str,
    bytes: &[u8],
    img_format: image::ImageFormat,
    height: u32,
) -> Result<(), SaveWebpError> {
    let path = PathBuf::from(_path);
    if !path.exists() {
        let img = ImgData::new(&bytes, img_format, height)?;
        let bytes = img.encode_webp()?;
        fs::write(&path, bytes)?;

        Ok(())
    } else {
        Err(SaveWebpError::AlreadyExist(String::from(_path)))
    }
}

pub async fn save_user_pfp(
    user_id: u64,
    pfp_hash: Option<String>,
    mongo_user: &Option<User>,
) -> Result<SaveUserPfpResult, SaveUserPfpError> {
    let user_pfp_hash = pfp_hash.ok_or(SaveUserPfpError::NotFound(user_id))?;

    let org_pfp_hash = match mongo_user {
        Some(user) => &user.pfp_hash,
        None => &None,
    };

    // format!("{:x}", u128::from_be_bytes(file_hash_bytes))
    //let md5_bytes: [u8; 16] = u128::from_str_radix(&user_pfp_hash, 16)?.to_be_bytes();
    let pfp_img_path = format!("target/site/gallery/pfp_{}.webp", user_id);
    let pfp_url = format!(
        "https://cdn.discordapp.com/avatars/{}/{}.webp",
        user_id, user_pfp_hash
    );

    let pfp_file_exists = PathBuf::from(&pfp_img_path).exists();

    if let Some(org_pfp_hash) = org_pfp_hash {
        if *org_pfp_hash == user_pfp_hash && pfp_file_exists {
            return Ok(SaveUserPfpResult::AlreadyExists(user_pfp_hash));
        }
    }

    let pfp_img_response = reqwest::get(pfp_url).await?;
    let org_img_bytes = pfp_img_response.bytes().await?;

    if pfp_file_exists {
        fs::write(&pfp_img_path, &org_img_bytes)?;
        Ok(SaveUserPfpResult::Updated(user_pfp_hash))
    } else {
        fs::write(&pfp_img_path, &org_img_bytes)?;
        Ok(SaveUserPfpResult::Created(user_pfp_hash))
    }
}

pub async fn save_user(
    db: &DB,
    name: String,
    guild_id: u64,
    user_id: u64,
    pfp_hash: Option<String>,
) -> Result<SaveUserResult, SaveUserError> {
    let user = db
        .collection_user
        .find_one(doc! { "id": format!("{}", user_id) }, None)
        .await?;

    let pfp = save_user_pfp(user_id, pfp_hash, &user).await;
    let pfp_hash = match pfp {
        Ok(result) => Ok(Some(result.into_string())),
        Err(e) => match e {
            SaveUserPfpError::NotFound(_) => Ok(None),
            err => Err(err),
        },
    }?;

    return if let Some(user) = user {
        let mut update = doc! {};

        match pfp_hash {
            Some(bin) => match user.pfp_hash {
                Some(org_bin) => {
                    if bin != org_bin {
                        update.insert("pfp_hash", bin);
                    }
                }
                None => {
                    update.insert("pfp_hash", bin);
                }
            },
            None => match user.pfp_hash {
                Some(_) => {
                    update.insert("pfp_hash", None::<String>);
                }
                None => {}
            },
        }

        if name != user.name {
            update.insert("name", name);
        }

        if update.len() > 0 {
            update.insert("modified_at", mongodb::bson::DateTime::now());
            db.collection_img
                .update_one(
                    doc! { "id": format!("{}", user_id) },
                    doc! {
                        "$set": update.clone()
                    },
                    None,
                )
                .await?;
            Ok(SaveUserResult::Updated(format!("{}", update)))
        } else {
            Ok(SaveUserResult::None)
        }
    } else {
        let user = User {
            _id: mongodb::bson::oid::ObjectId::new(),
            guild_id: guild_id.to_string(),
            id: format!("{}", user_id),
            name,
            pfp_hash,
            modified_at: mongodb::bson::DateTime::now(),
            created_at: mongodb::bson::DateTime::now(),
        };

        db.collection_user.insert_one(&user, None).await?;

        Ok(SaveUserResult::Created)
    };
}

pub async fn save_attachment(
    db: &DB,
    guild_id: u64,
    channel_id: u64,
    user_id: u64,
    msg_id: u64,
    attachment: &Attachment,
) -> Result<SaveAttachmentResult, SaveAttachmentError> {
    let content_type = attachment
        .content_type
        .as_ref()
        .ok_or(SaveAttachmentError::ImgTypeNotFound)?;

    let format = match content_type.as_str() {
        "image/png" => Ok("png"),
        "image/jpeg" => Ok("jpeg"),
        t => Err(SaveAttachmentError::ImgTypeUnsupported(t.to_string())),
    }?;

    let org_img_response = reqwest::get(&attachment.url).await?;
    let org_img_bytes = org_img_response.bytes().await?;

    let file_hash_bytes: [u8; 16] = hashes::md5::hash(org_img_bytes.as_bytes()).into_bytes();
    let file_hash_hex = format!("{:x}", u128::from_be_bytes(file_hash_bytes));

    let file_name = PathBuf::from(&attachment.filename);
    //let file_stem = file_name.file_stem().unwrap().to_str().unwrap();
    let file_ext = file_name.extension().unwrap().to_str().unwrap();

    let org_img_path = format!("target/site/gallery/org_{}.{}", file_hash_hex, file_ext);
    let low_img_path = format!("target/site/gallery/low_{}.webp", file_hash_hex);
    let medium_img_path = format!("target/site/gallery/medium_{}.webp", file_hash_hex);
    let high_img_path = format!("target/site/gallery/high_{}.webp", file_hash_hex);

    let paths = [low_img_path, medium_img_path, high_img_path];
    let mut paths_state = [false, false, false];
    let img_heights = [360, 720, 1080];

    let save_org_img_result = save_org_img(&org_img_path, &org_img_bytes);
    if let Err(save_org_img_result) = save_org_img_result {
        match save_org_img_result {
            SaveImgError::AlreadyExist(_) => Ok(()),
            err => Err(err),
        }?
    }

    'path_loop: for (i, path) in paths.iter().enumerate() {
        println!("{}", path);
        paths_state[i] = match save_webp(
            path,
            &org_img_bytes,
            image::ImageFormat::Png,
            img_heights[i],
        ) {
            Ok(_) => Ok(true),
            Err(e) => match e {
                SaveWebpError::AlreadyExist(_p) => Ok(true),
                SaveWebpError::ImgDecoding(decoding_err) => match decoding_err {
                    ImgDataNewError::ImgTooSmall { from: _, to: _ } => break 'path_loop,
                    err => Err(SaveWebpError::from(err)),
                },
                err => Err(err),
            },
        }?;
    }

    let found_img = db
        .collection_img
        .find_one(
            doc! {
                "org_hash": file_hash_hex.clone()
            },
            None,
        )
        .await?;

    if let Some(found_img) = found_img {
        let db_img_names = ["has_low", "has_medium", "has_high"];
        let db_img_states = [found_img.has_low, found_img.has_medium, found_img.has_high];

        let mut update = doc! {};

        let msg_id = msg_id.to_string();
        if found_img.id != msg_id {
            update.insert("id", msg_id);
        }

        if !found_img.show {
            update.insert("show", true);
        }

        if found_img.org_url != attachment.url {
            update.insert("org_url", attachment.url.clone());
        }

        for (i, path_state) in paths_state.into_iter().enumerate() {
            if db_img_states[i] != path_state {
                update.insert(db_img_names[i], path_state);
            }
        }

        if update.len() > 0 {
            update.insert("modified_at", mongodb::bson::DateTime::now());
            let update_msg = format!("{}: {:#?}", file_hash_hex, &update);
            let _update_status = db
                .collection_img
                .update_one(
                    doc! { "org_hash": file_hash_hex.clone() },
                    doc! {
                        "$set": update
                    },
                    None,
                )
                .await?;
            Ok(SaveAttachmentResult::Updated(update_msg))
        } else {
            Ok(SaveAttachmentResult::None(file_hash_hex))
        }
    } else {
        let org_img: image::DynamicImage = image::io::Reader::new(Cursor::new(&org_img_bytes))
            .with_guessed_format()?
            .decode()?;

        let img = crate::database::Img {
            _id: mongodb::bson::oid::ObjectId::new(),
            show: true,
            guild_id: guild_id.to_string(),
            channel_id: channel_id.to_string(),
            user_id: format!("{}", user_id),
            id: format!("{}", msg_id),
            org_url: attachment.url.clone(),
            org_hash: file_hash_hex.clone(),
            format: file_ext.to_string(),
            width: org_img.width(),
            height: org_img.height(),
            has_high: paths_state[2],
            has_medium: paths_state[1],
            has_low: paths_state[0],
            modified_at: mongodb::bson::DateTime::now(),
            created_at: mongodb::bson::DateTime::now(),
        };

        db.collection_img.insert_one(&img, None).await?;
        Ok(SaveAttachmentResult::Created(file_hash_hex))
    }
}

#[derive(Error, Debug)]
pub enum SaveImgError {
    #[error("Img already exists at {0}.")]
    AlreadyExist(String),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum SaveWebpError {
    #[error("{0}")]
    ImgDecoding(#[from] ImgDataNewError),

    #[error("{0}")]
    ImgEncoding(#[from] ImgDataEncodeWebpError),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Img already exists at {0}.")]
    AlreadyExist(String),
}

#[derive(Error, Debug)]
pub enum ImgDataNewError {
    #[error("Img invalid format: {0}.")]
    Format(#[from] std::io::Error),

    #[error("Failed to decode img: {0}.")]
    Decode(#[from] image::ImageError),

    #[error("Img too small to covert from {from:?} to {to:?}.")]
    ImgTooSmall { from: u32, to: u32 },
}

#[derive(Error, Debug)]
pub enum ImgDataEncodeWebpError {
    #[error("Webp encoding error: {0}")]
    Encode(String),
}

#[derive(Error, Debug)]
pub enum SaveAttachmentError {
    #[error("Msg content type not found.")]
    ImgTypeNotFound,

    #[error("Msg content type not found {0}.")]
    ImgTypeUnsupported(String),

    #[error("Failed downloading img {0}.")]
    Request(#[from] reqwest::Error),

    #[error("Failed to save org img {0}.")]
    ImgSave(#[from] SaveImgError),

    #[error("Failed to save webp img {0}.")]
    ImgSaveWebp(#[from] SaveWebpError),

    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Failed to decode img: {0}.")]
    Decode(#[from] image::ImageError),
}

#[derive(Error, Debug)]
pub enum SaveAttachmentsError {
    // #[error("Skip event.")]
    // Skip(())
    //
    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),

    // #[error("User is not authorized.")]
    // Unauthorized(String),
    #[error("Failed to save user: {0}")]
    SaveUserError(#[from] SaveUserError),

    #[error("Failed to save attachment: {0}")]
    SaveAttachmentError(#[from] SaveAttachmentError),
}

#[derive(Error, Debug)]
pub enum SaveUserError {
    #[error("Failed to save pfp: {0}")]
    SavingPfp(#[from] SaveUserPfpError),

    #[error("Mongodb: {0}.")]
    Mongo(#[from] mongodb::error::Error),
}

#[derive(Error, Debug)]
pub enum SaveUserPfpError {
    #[error("User '{0}' pfp not found.")]
    NotFound(u64),

    #[error("Failed downloading pfp img {0}.")]
    Request(#[from] reqwest::Error),

    #[error("Failed to convert hex to decimal {0}.")]
    HexToDec(#[from] ParseIntError),

    #[error("Failed to save pfp: {0}")]
    IO(#[from] std::io::Error),
}

pub enum SaveAttachmentResult {
    Created(String),
    Updated(String),
    None(String),
}

impl Display for SaveAttachmentResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                SaveAttachmentResult::Updated(str) => format!("updated: {}", str),
                SaveAttachmentResult::Created(str) => format!("created: {}", str),
                SaveAttachmentResult::None(str) => format!("good: {}", str),
            }
        )
    }
}

pub enum SaveUserPfpResult {
    AlreadyExists(String),
    Updated(String),
    Created(String),
}

impl SaveUserPfpResult {
    pub fn into_string(self) -> String {
        match self {
            SaveUserPfpResult::Created(str) => str,
            SaveUserPfpResult::Updated(str) => str,
            SaveUserPfpResult::AlreadyExists(str) => str,
        }
    }
}

pub enum SaveUserResult {
    Updated(String),
    Created,
    None,
}
