use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use ::serenity::all::{
    Attachment, ButtonStyle, CacheHttp, ComponentInteraction, CreateAttachment, CreateButton, CreateInteractionResponse, CreateInteractionResponseFollowup, CreateInteractionResponseMessage, CreateMessage, CreateQuickModal, EditAttachments, EditInteractionResponse, InputTextStyle, Interaction, Message, ModalInteraction, QuickModalResponse, ReactionType
};
use toml::value::Datetime;
use std::{
    env, io::Cursor, sync::{Arc}, time::Duration
};
use anyhow::{bail, Result};
use reqwest;
use image::DynamicImage;

// Types used by all command functions
type SContext = serenity::Context;
use log::{info, warn};
use ttl_cache::TtlCache;
use tokio::{sync::{Mutex, RwLock}, time::Instant};
use lru_time_cache::LruCache;
use bytes::Bytes;
use std::collections::HashSet;
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
        let mut cache = self.cache.write();
        cache.await.insert(key, (image, info));
    }

    pub async fn get(&self, key: &str) -> Option<(DynamicImage, ImageInformation)> {
        let mut cache = self.cache.write();
        cache.await.get(key).map(|(image, info)| {
            (image.clone(), info.clone())
        })
    }
}