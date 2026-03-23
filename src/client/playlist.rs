//! 歌单域 typed builder。
//!
//! # Overview
//!
//! 该模块提供歌单详情、歌单分类和分类歌单列表三类能力。
//! 若需要鉴权能力，可在请求上通过 `.login(...)` 注入对应平台 token。
//!
//! # Quick Start
//!
//! ```no_run
//! # async fn run() -> Result<(), music_bridge_rs::MusicClientError> {
//! let client = music_bridge_rs::MusicClient::new();
//! let detail = client.playlist().detail().id(8903867087).send().await?;
//! println!("playlist: {}", detail.name);
//! # Ok(())
//! # }
//! ```

use std::marker::PhantomData;

use super::utils::{netease_token, require_category, tencent_token, validate_auth_platform};
use super::{LoginTokenRef, MusicClient};
use crate::error::{MusicClientError, MusicClientResult};
use crate::models::{Platform, PlaylistCategoriesResult, PlaylistDetailResult, PlaylistListResult};

/// 歌单域根类型态标记。
#[derive(Clone, Copy, Debug)]
pub struct PlaylistKind;

/// 歌单详情类型态标记。
#[derive(Clone, Copy, Debug)]
pub struct PlaylistDetailKind;

/// 歌单分类类型态标记。
#[derive(Clone, Copy, Debug)]
pub struct PlaylistCategoriesKind;

/// 歌单列表类型态标记。
#[derive(Clone, Copy, Debug)]
pub struct PlaylistListKind;

/// 歌单请求构建器。
///
/// `K` 为类型态参数，用于限制当前子能力。
///
/// 歌单列表子能力默认分页参数为 `offset = 0`、`limit = 20`。
pub struct PlaylistRequest<'a, K> {
    client: &'a MusicClient,
    keyword: Option<String>,
    id: Option<u64>,
    category: Option<String>,
    offset: u64,
    limit: u64,
    platform: Platform,
    token: Option<LoginTokenRef<'a>>,
    _kind: PhantomData<K>,
}

impl<'a, K> PlaylistRequest<'a, K> {
    const DEFAULT_LIMIT: u64 = 20;
    const DEFAULT_OFFSET: u64 = 0;

    fn into_kind<T>(self) -> PlaylistRequest<'a, T> {
        PlaylistRequest {
            client: self.client,
            keyword: self.keyword,
            id: self.id,
            category: self.category,
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

impl<'a> PlaylistRequest<'a, PlaylistKind> {
    pub(super) fn new(client: &'a MusicClient) -> Self {
        Self {
            client,
            keyword: None,
            id: None,
            category: None,
            offset: PlaylistRequest::<PlaylistKind>::DEFAULT_OFFSET,
            limit: PlaylistRequest::<PlaylistKind>::DEFAULT_LIMIT,
            platform: Platform::Netease,
            token: None,
            _kind: PhantomData,
        }
    }

    /// 切换为歌单详情请求。
    pub fn detail(self) -> PlaylistRequest<'a, PlaylistDetailKind> {
        self.into_kind()
    }

    /// 切换为歌单分类请求。
    pub fn categories(self) -> PlaylistRequest<'a, PlaylistCategoriesKind> {
        self.into_kind()
    }

    /// 切换为歌单列表请求。
    pub fn list(self) -> PlaylistRequest<'a, PlaylistListKind> {
        self.into_kind()
    }
}

impl<'a> PlaylistRequest<'a, PlaylistDetailKind> {
    /// 设置歌单 ID（数值，必填）。
    pub fn id(mut self, id: u64) -> Self {
        self.id = Some(id);
        self
    }

    /// 发送歌单详情请求。
    ///
    /// # 前置条件
    ///
    /// - 需要先通过 [`Self::id`] 设置歌单 ID。
    /// - 若调用过 [`PlaylistRequest::login`]，其 token 所属平台必须与 [`PlaylistRequest::platform`]
    ///   一致。
    ///
    /// # Errors
    ///
    /// - [`MusicClientError::MissingId`] - 未设置歌单 `id`
    /// - [`MusicClientError::AuthTokenPlatformMismatch`] - token 所属平台与请求平台不匹配
    /// - [`MusicClientError::NetworkError`] - 网络请求失败
    pub async fn send(self) -> MusicClientResult<PlaylistDetailResult> {
        let id = self.id.ok_or(MusicClientError::MissingId)?;
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

impl<'a> PlaylistRequest<'a, PlaylistCategoriesKind> {
    /// 发送歌单分类请求。
    ///
    /// # 前置条件
    ///
    /// - 若调用过 [`PlaylistRequest::login`]，其 token 所属平台必须与 [`PlaylistRequest::platform`]
    ///   一致。
    ///
    /// # Errors
    ///
    /// - [`MusicClientError::AuthTokenPlatformMismatch`] - token 所属平台与请求平台不匹配
    /// - [`MusicClientError::NetworkError`] - 网络请求失败
    pub async fn send(self) -> MusicClientResult<PlaylistCategoriesResult> {
        validate_auth_platform(self.platform, self.token)?;
        match self.platform {
            Platform::Netease => {
                self.client.netease.get_playlist_categories(netease_token(self.token)).await
            }
            Platform::Tencent => {
                self.client.tencent.get_playlist_categories(tencent_token(self.token)).await
            }
        }
    }
}

impl<'a> PlaylistRequest<'a, PlaylistListKind> {
    /// 设置歌单分类（必填且不能为空字符串）。
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// 设置分页偏移量，默认 `0`。
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = offset;
        self
    }

    /// 设置分页大小，默认 `20`。
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = limit;
        self
    }

    /// 发送歌单列表请求。
    ///
    /// # 前置条件
    ///
    /// - 需要先通过 [`Self::category`] 设置非空分类名称或分类
    ///   ID（字符串形式，去除首尾空白后不能为空）。
    /// - 若调用过 [`PlaylistRequest::login`]，其 token 所属平台必须与 [`PlaylistRequest::platform`]
    ///   一致。
    ///
    /// # Errors
    ///
    /// - [`MusicClientError::MissingCategory`] - 未设置 `category` 或 `category` 为空字符串
    /// - [`MusicClientError::AuthTokenPlatformMismatch`] - token 所属平台与请求平台不匹配
    /// - [`MusicClientError::NetworkError`] - 网络请求失败
    pub async fn send(self) -> MusicClientResult<PlaylistListResult> {
        let category = require_category(self.category.as_deref())?;
        validate_auth_platform(self.platform, self.token)?;
        match self.platform {
            Platform::Netease => {
                self.client
                    .netease
                    .get_playlist_list(category, self.limit, self.offset, netease_token(self.token))
                    .await
            }
            Platform::Tencent => {
                self.client
                    .tencent
                    .get_playlist_list(category, self.limit, self.offset, tencent_token(self.token))
                    .await
            }
        }
    }
}
