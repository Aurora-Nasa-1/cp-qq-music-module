//! 详情域 typed builder。
//!
//! # Overview
//!
//! 该模块提供歌手、专辑、歌单和榜单详情查询。
//! 不同子能力的 ID 类型不同：歌手/专辑使用字符串 ID，歌单/榜单使用数值 ID。
//!
//! # Quick Start
//!
//! ```no_run
//! # async fn run() -> Result<(), music_bridge_rs::MusicClientError> {
//! let client = music_bridge_rs::MusicClient::new();
//! let artist = client.detail().artist().id("3684").limit(10).send().await?;
//! println!("artist: {}", artist.name);
//! # Ok(())
//! # }
//! ```

use std::marker::PhantomData;

use super::utils::{netease_token, tencent_token, validate_auth_platform};
use super::{LoginTokenRef, MusicClient};
use crate::error::{MusicClientError, MusicClientResult};
use crate::models::{AlbumDetailResult, ArtistDetailResult, Platform, PlaylistDetailResult};

/// 详情域根类型态标记。
#[derive(Clone, Copy, Debug)]
pub struct DetailKind;

/// 歌手详情类型态标记。
#[derive(Clone, Copy, Debug)]
pub struct DetailArtistKind;

/// 专辑详情类型态标记。
#[derive(Clone, Copy, Debug)]
pub struct DetailAlbumKind;

/// 歌单详情类型态标记。
#[derive(Clone, Copy, Debug)]
pub struct DetailPlaylistKind;

/// 榜单详情类型态标记。
#[derive(Clone, Copy, Debug)]
pub struct DetailToplistKind;

/// 详情请求构建器。
///
/// `K` 为类型态参数，用于限制可调用的方法组合。
///
/// 当目标能力支持歌曲分页时，默认 `offset = 0`、`limit = 20`。
pub struct DetailRequest<'a, K> {
    client: &'a MusicClient,
    id: Option<String>,
    playlist_id: Option<u64>,
    offset: u64,
    limit: u64,
    platform: Platform,
    token: Option<LoginTokenRef<'a>>,
    _kind: PhantomData<K>,
}

impl<'a, K> DetailRequest<'a, K> {
    const DEFAULT_LIMIT: u64 = 20;
    const DEFAULT_OFFSET: u64 = 0;

    fn into_kind<T>(self) -> DetailRequest<'a, T> {
        DetailRequest {
            client: self.client,
            id: self.id,
            playlist_id: self.playlist_id,
            offset: self.offset,
            limit: self.limit,
            platform: self.platform,
            token: self.token,
            _kind: PhantomData,
        }
    }

    /// 设置请求平台，默认 `Platform::Netease`。
    pub fn platform(mut self, platform: Platform) -> Self {
        self.platform = platform;
        self
    }

    /// 注入登录 token。
    ///
    /// 若 token 所属平台与 `.platform(...)` 不一致，`send()` 会返回
    /// `MusicClientError::AuthTokenPlatformMismatch`。
    pub fn login(mut self, token: impl Into<LoginTokenRef<'a>>) -> Self {
        self.token = Some(token.into());
        self
    }
}

impl<'a> DetailRequest<'a, DetailKind> {
    pub(super) fn new(client: &'a MusicClient) -> Self {
        Self {
            client,
            id: None,
            playlist_id: None,
            offset: DetailRequest::<DetailKind>::DEFAULT_OFFSET,
            limit: DetailRequest::<DetailKind>::DEFAULT_LIMIT,
            platform: Platform::Netease,
            token: None,
            _kind: PhantomData,
        }
    }

    /// 切换为歌手详情请求。
    pub fn artist(self) -> DetailRequest<'a, DetailArtistKind> {
        self.into_kind()
    }

    /// 切换为专辑详情请求。
    pub fn album(self) -> DetailRequest<'a, DetailAlbumKind> {
        self.into_kind()
    }

    /// 切换为歌单详情请求。
    pub fn playlist(self) -> DetailRequest<'a, DetailPlaylistKind> {
        self.into_kind()
    }

    /// 切换为榜单详情请求。
    pub fn toplist(self) -> DetailRequest<'a, DetailToplistKind> {
        self.into_kind()
    }
}

impl<'a> DetailRequest<'a, DetailArtistKind> {
    /// 设置歌手 ID（字符串，必填）。
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 设置歌曲列表分页偏移量，默认 `0`。
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }

    /// 设置歌曲列表分页大小，默认 `20`。
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = limit;
        self
    }

    /// 发送歌手详情请求。
    ///
    /// # 前置条件
    ///
    /// - 需要先通过 [`Self::id`] 设置歌手 ID。
    /// - 若调用过 [`DetailRequest::login`]，其 token 所属平台必须与 [`DetailRequest::platform`]
    ///   一致。
    ///
    /// # Errors
    ///
    /// - [`MusicClientError::MissingId`] - 未设置歌手 `id`
    /// - [`MusicClientError::AuthTokenPlatformMismatch`] - token 所属平台与请求平台不匹配
    /// - [`MusicClientError::NetworkError`] - 网络请求失败
    pub async fn send(self) -> MusicClientResult<ArtistDetailResult> {
        let id = self.id.as_deref().ok_or(MusicClientError::MissingId)?;
        validate_auth_platform(self.platform, self.token)?;
        match self.platform {
            Platform::Netease => {
                self.client
                    .netease
                    .artist_detail(id, self.limit, self.offset, netease_token(self.token))
                    .await
            }
            Platform::Tencent => {
                self.client
                    .tencent
                    .artist_detail(id, self.limit, self.offset, tencent_token(self.token))
                    .await
            }
        }
    }
}

impl<'a> DetailRequest<'a, DetailAlbumKind> {
    /// 设置专辑 ID（字符串，必填）。
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// 发送专辑详情请求。
    ///
    /// # 前置条件
    ///
    /// - 需要先通过 [`Self::id`] 设置专辑 ID。
    /// - 若调用过 [`DetailRequest::login`]，其 token 所属平台必须与 [`DetailRequest::platform`]
    ///   一致。
    ///
    /// # Errors
    ///
    /// - [`MusicClientError::MissingId`] - 未设置专辑 `id`
    /// - [`MusicClientError::AuthTokenPlatformMismatch`] - token 所属平台与请求平台不匹配
    /// - [`MusicClientError::NetworkError`] - 网络请求失败
    pub async fn send(self) -> MusicClientResult<AlbumDetailResult> {
        let id = self.id.as_deref().ok_or(MusicClientError::MissingId)?;
        validate_auth_platform(self.platform, self.token)?;
        match self.platform {
            Platform::Netease => {
                self.client.netease.album_detail(id, netease_token(self.token)).await
            }
            Platform::Tencent => {
                self.client.tencent.album_detail(id, tencent_token(self.token)).await
            }
        }
    }
}

impl<'a> DetailRequest<'a, DetailPlaylistKind> {
    /// 设置歌单 ID（数值，必填）。
    pub fn id(mut self, id: u64) -> Self {
        self.playlist_id = Some(id);
        self
    }

    /// 发送歌单详情请求。
    ///
    /// # 前置条件
    ///
    /// - 需要先通过 [`Self::id`] 设置歌单 ID。
    /// - 若调用过 [`DetailRequest::login`]，其 token 所属平台必须与 [`DetailRequest::platform`]
    ///   一致。
    ///
    /// # Errors
    ///
    /// - [`MusicClientError::MissingId`] - 未设置歌单 `id`
    /// - [`MusicClientError::AuthTokenPlatformMismatch`] - token 所属平台与请求平台不匹配
    /// - [`MusicClientError::NetworkError`] - 网络请求失败
    pub async fn send(self) -> MusicClientResult<PlaylistDetailResult> {
        let id = self.playlist_id.ok_or(MusicClientError::MissingId)?;
        validate_auth_platform(self.platform, self.token)?;
        match self.platform {
            Platform::Netease => {
                self.client.netease.playlist_detail(id, netease_token(self.token)).await
            }
            Platform::Tencent => {
                self.client.tencent.playlist_detail(id, tencent_token(self.token)).await
            }
        }
    }
}

impl<'a> DetailRequest<'a, DetailToplistKind> {
    /// 设置榜单 ID（数值，必填）。
    pub fn id(mut self, id: u64) -> Self {
        self.playlist_id = Some(id);
        self
    }

    /// 发送榜单详情请求。
    ///
    /// 在 `Netease` 平台会复用歌单详情接口，在 `Tencent` 平台会调用独立榜单详情接口。
    ///
    /// # 前置条件
    ///
    /// - 需要先通过 [`Self::id`] 设置榜单 ID。
    /// - 若调用过 [`DetailRequest::login`]，其 token 所属平台必须与 [`DetailRequest::platform`]
    ///   一致。
    ///
    /// # Errors
    ///
    /// - [`MusicClientError::MissingId`] - 未设置榜单 `id`
    /// - [`MusicClientError::AuthTokenPlatformMismatch`] - token 所属平台与请求平台不匹配
    /// - [`MusicClientError::NetworkError`] - 网络请求失败
    pub async fn send(self) -> MusicClientResult<PlaylistDetailResult> {
        let id = self.playlist_id.ok_or(MusicClientError::MissingId)?;
        validate_auth_platform(self.platform, self.token)?;
        match self.platform {
            Platform::Netease => {
                self.client.netease.playlist_detail(id, netease_token(self.token)).await
            }
            Platform::Tencent => {
                self.client.tencent.toplist_detail(id, tencent_token(self.token)).await
            }
        }
    }
}
