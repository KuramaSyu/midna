use std::time::Duration;
use image::DynamicImage;
use tokio::sync::RwLock;
use lru_time_cache::LruCache;

// use colors.rs
use crate::utils::colors::ImageInformation;



pub struct ImageCache {
    pub cache: RwLock<LruCache<String, (DynamicImage, ImageInformation)>>,
}

impl ImageCache {
    pub fn new(capacity: usize, ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(LruCache::with_expiry_duration_and_capacity(ttl, capacity)),
        }
    }

    pub async fn insert(&self, key: String, image: DynamicImage, info: ImageInformation) {
        let cache = self.cache.write();
        cache.await.insert(key, (image, info));
    }

    pub async fn get(&self, key: &str) -> Option<(DynamicImage, ImageInformation)> {
        let cache = self.cache.write();
        cache.await.get(key).map(|(image, info)| {
            (image.clone(), info.clone())
        })
    }
}