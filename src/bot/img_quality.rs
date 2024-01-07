#[derive(Clone, PartialEq, Debug)]
pub enum ImgQuality {
    Low,
    Medium,
    High,
    Org,
}

impl ImgQuality {
    pub fn gen_link_preview(&self, hex: &str, format: &str) -> String {
        match self {
            ImgQuality::Low => format!("/assets/gallery/low_{}.webp", hex),
            ImgQuality::Medium => format!("/assets/gallery/medium_{}.webp", hex),
            ImgQuality::High => format!("/assets/gallery/high_{}.webp", hex),
            ImgQuality::Org => format!("/assets/gallery/org_{}.{}", hex, format),
        }
    }

    pub fn gen_link_org(hex: &str, format: &str) -> String {
        format!("/assets/gallery/org_{}.{}", hex, format)
    }

    pub fn gen_img_path_org(root: &str, hex: &str, format: &str) -> String {
        // format!("target/site/gallery/org_{}.{}", hex, format)
        format!("target/site/gallery/org_{}.{}", hex, format)
    }

    pub fn gen_img_path_high(root: &str, hex: &str) -> String {
        format!("target/site/gallery/high_{}.webp", hex)
    }

    pub fn gen_img_path_medium(root: &str, hex: &str) -> String {
        format!("target/site/gallery/medium_{}.webp", hex)
    }

    pub fn gen_img_path_low(root: &str, hex: &str) -> String {
        format!("target/site/gallery/low_{}.webp", hex)
    }

    pub fn sizes() -> [u32; 3] {
        [360, 720, 1080]
    }
}