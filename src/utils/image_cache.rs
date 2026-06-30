use std::collections::{HashMap, HashSet};
use egui::TextureHandle;

pub struct ImageCache {
    textures: HashMap<String, TextureHandle>,
    pending: HashSet<String>,
}

impl ImageCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            pending: HashSet::new(),
        }
    }

    pub fn get(&self, url: &str) -> Option<&TextureHandle> {
        self.textures.get(url)
    }

    pub fn is_pending(&self, url: &str) -> bool {
        self.pending.contains(url)
    }

    pub fn mark_pending(&mut self, url: String) {
        self.pending.insert(url);
    }

    pub fn insert(&mut self, url: String, texture: TextureHandle) {
        self.pending.remove(&url);
        self.textures.insert(url, texture);
    }

    pub fn contains(&self, url: &str) -> bool {
        self.textures.contains_key(url)
    }

    pub fn mark_failed(&mut self, url: &str) {
        self.pending.remove(url);
    }
}

pub async fn fetch_image(http: &reqwest::Client, url: &str) -> anyhow::Result<egui::ColorImage> {
    let bytes = http.get(url).send().await?.bytes().await?;
    let img = image::load_from_memory(&bytes)?;
    let size = [img.width() as usize, img.height() as usize];
    let rgba = img.to_rgba8();
    Ok(egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()))
}
