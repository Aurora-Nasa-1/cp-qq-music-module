//! 对外统一客户端入口与 typed builder 导出。
//!
//! # Overview
//!
//! 本模块聚合了所有公开请求构建器，并通过 [`MusicClient`] 暴露统一入口。
//! 每个业务域都使用类型态（`*Kind`）表达“当前请求阶段”，避免在编译期可规避的错误组合。
//!
//! # Quick Start
//!
//! ```no_run
//! # async fn run() -> Result<(), music_bridge_rs::MusicClientError> {
//! let client = music_bridge_rs::MusicClient::new();
//! let lyric = client.playback().lyric().id("108914").send().await?;
//! println!("lyric len: {}", lyric.lyric.len());
//! # Ok(())
//! # }
//! ```
//!
//! # Core Concepts
//!
//! - 域入口方法：`search/detail/playback/discover/playlist/login`。
//! - `*Request<'a, K>`：链式请求构建器。
//! - `*Kind`：类型态标记，约束不同阶段可调用的方法集合。
//! - [`LoginTokenRef`][`crate::client::LoginTokenRef`]：跨平台 token 借用包装。
//!
//! # Errors and Panics
//!
//! - 请求失败通过 [`MusicClientError`][`crate::error::MusicClientError`] 返回，不以 panic
//!   作为业务控制流。
//! - 参数缺失、平台不匹配与网络错误都可通过错误变体区分。
//!
//! # See Also
//!
//! - [`models`][`crate::models`]：请求返回的统一数据模型。
//! - [`error`][`crate::error`]：错误定义与结果类型。

use crate::platform::{NeteaseClient, TencentClient};

mod detail;
mod discover;
mod login;
mod playback;
mod playlist;
mod search;
mod utils;

/// 详情域请求构建器及其类型态标记。
pub use detail::{
    DetailAlbumKind, DetailArtistKind, DetailKind, DetailPlaylistKind, DetailRequest,
    DetailToplistKind,
};
/// 发现域请求构建器及其类型态标记。
pub use discover::{
    DiscoverHotkeyKind, DiscoverKind, DiscoverPlaylistCategoriesKind, DiscoverPlaylistListKind,
    DiscoverRecommendPlaylistKind, DiscoverRequest, DiscoverToplistListKind,
};
/// 登录域请求构建器、会话对象与类型态标记。
pub use login::{LoginKind, LoginRefreshKind, LoginRequest, LoginSession, LoginSessionKind};
/// 播放域请求构建器及其类型态标记。
pub use playback::{PlaybackKind, PlaybackLyricKind, PlaybackRequest, PlaybackUrlKind};
/// 歌单域请求构建器及其类型态标记。
pub use playlist::{
    PlaylistCategoriesKind, PlaylistDetailKind, PlaylistKind, PlaylistListKind, PlaylistRequest,
};
/// 搜索域请求构建器及其类型态标记。
pub use search::{
    SearchAlbumKind, SearchArtistKind, SearchKind, SearchPlaylistKind, SearchRequest,
    SearchSongKind,
};
/// 统一 token 借用类型，用于跨平台鉴权参数传递。
pub use utils::LoginTokenRef;

#[cfg(test)]
mod tests;

/// 统一音乐客户端入口。
///
/// 该类型本身不携带业务参数，只负责创建各业务域请求构建器。
/// 默认平台为 [`Platform::Netease`][`crate::models::Platform::Netease`]，你可以在构建器上通过
/// `.platform(...)` 切换。
///
/// # Examples
///
/// ```no_run
/// # async fn run() -> Result<(), music_bridge_rs::MusicClientError> {
/// let client = music_bridge_rs::MusicClient::new();
/// let result = client.search().song().keyword("林俊杰").send().await?;
/// println!("found: {}", result.songs.len());
/// # Ok(())
/// # }
/// ```
pub struct MusicClient {
    netease: NeteaseClient,
    tencent: TencentClient,
}

impl Default for MusicClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicClient {
    /// 创建新的客户端实例。
    ///
    /// 返回值可复用，用于发起多个独立请求。
    pub fn new() -> Self {
        Self { netease: NeteaseClient::new(), tencent: TencentClient::new() }
    }

    /// 创建搜索域请求构建器。
    pub fn search(&self) -> SearchRequest<'_, SearchKind> {
        SearchRequest::new(self)
    }

    /// 创建详情域请求构建器。
    pub fn detail(&self) -> DetailRequest<'_, DetailKind> {
        DetailRequest::new(self)
    }

    /// 创建播放域请求构建器。
    pub fn playback(&self) -> PlaybackRequest<'_, PlaybackKind> {
        PlaybackRequest::new(self)
    }

    /// 创建发现域请求构建器。
    pub fn discover(&self) -> DiscoverRequest<'_, DiscoverKind> {
        DiscoverRequest::new(self)
    }

    /// 创建歌单域请求构建器。
    pub fn playlist(&self) -> PlaylistRequest<'_, PlaylistKind> {
        PlaylistRequest::new(self)
    }

    /// 创建登录域请求构建器。
    pub fn login(&self) -> LoginRequest<'_, LoginKind> {
        LoginRequest::new(self)
    }
}
