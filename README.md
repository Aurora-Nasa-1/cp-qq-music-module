# music-bridge-rs

`music-bridge-rs` 是一个 Rust 异步库，提供统一的音乐客户端接口，封装网易云音乐（Netease）与 QQ 音乐（Tencent）的常见能力。

目标是让上层业务通过一致的调用方式完成搜索、详情、播放、登录、歌单和发现等操作，而不必分别处理两个平台的接口细节。

> [!IMPORTANT]
> 请遵守当地法律法规与各音乐平台服务条款，尊重版权并支持正版。  
> 本项目不内置、不分发任何音频文件或付费资源。

## 功能概览

- 统一客户端入口：`MusicClient`
- 统一平台枚举：`Platform::{Netease, Tencent}`
- 链式请求构建器（typed builder）
- 已支持能力：
  - `search`：歌曲/歌手/专辑/歌单搜索
  - `detail`：歌手/专辑/歌单/榜单详情
  - `playback`：歌词、播放链接
  - `discover`：热词、推荐歌单、榜单、歌单分类、歌单列表
  - `playlist`：歌单详情、分类、列表
  - `login`：二维码登录会话、登录状态轮询、Token 刷新

## 安装

### 1) 本地路径依赖

```toml
[dependencies]
music-bridge-rs = { path = "/path/to/music-bridge-rs" }
tokio = { version = "1", features = ["macros", "rt-multi-thread", "time"] }
```

### 2) 运行环境

- Rust edition: `2024`
- 异步运行时：`tokio`

## 快速开始

```rust
use music_bridge_rs::client::MusicClient;
use music_bridge_rs::models::Platform;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MusicClient::new();

    let result = client
        .search()
        .song()
        .keyword("江南")
        .platform(Platform::Netease)
        .limit(10)
        .send()
        .await?;

    for song in result.songs {
        println!("{} - {}", song.id, song.name);
    }

    Ok(())
}
```

## 统一调用模型

调用形式统一为：

```text
client.<domain>().<kind>().<params>().send().await
```

说明：

- 默认平台是 `Platform::Netease`
- 通用分页默认值：`offset = 0`, `limit = 20`（适用于支持分页的请求）
- 需要登录态的请求可通过 `.login(&token)` 注入平台 Token
- 若平台与 Token 类型不一致，会返回 `MusicClientError::AuthTokenPlatformMismatch`

## API 能力与返回类型

| 域 | 子能力 | 必填参数 | 返回类型 |
| --- | --- | --- | --- |
| `search` | `song` / `artist` / `album` / `playlist` | `keyword` | `SearchSongResult` / `SearchArtistResult` / `SearchAlbumResult` / `SearchPlaylistResult` |
| `detail` | `artist` / `album` | `id: String` | `ArtistDetailResult` / `AlbumDetailResult` |
| `detail` | `playlist` / `toplist` | `id: u64` | `PlaylistDetailResult` |
| `playback` | `lyric` | `id: String` | `LyricResult` |
| `playback` | `url` | `id: String` | `UrlResult` |
| `discover` | `hotkey` / `recommend_playlist` / `toplist_list` / `playlist_categories` | 无 | 对应统一模型 |
| `discover` | `playlist_list` | `category` | `PlaylistListResult` |
| `playlist` | `detail` | `id: u64` | `PlaylistDetailResult` |
| `playlist` | `categories` | 无 | `PlaylistCategoriesResult` |
| `playlist` | `list` | `category` | `PlaylistListResult` |
| `login` | `session` | 无 | `LoginSession` |
| `login` | `refresh` | `token` | `LoginToken` |

## 常见调用示例

### 1) 搜索

```rust
use music_bridge_rs::client::MusicClient;
use music_bridge_rs::models::Platform;

async fn demo(client: &MusicClient) -> Result<(), music_bridge_rs::error::MusicClientError> {
    let songs = client
        .search()
        .song()
        .keyword("林俊杰")
        .platform(Platform::Netease)
        .limit(10)
        .offset(0)
        .send()
        .await?;

    println!("songs = {}", songs.songs.len());
    Ok(())
}
```

### 2) 详情与播放

```rust
use music_bridge_rs::client::MusicClient;
use music_bridge_rs::models::{Platform, SongQuality};

async fn demo(client: &MusicClient) -> Result<(), music_bridge_rs::error::MusicClientError> {
    let artist = client
        .detail()
        .artist()
        .id("3684")
        .platform(Platform::Netease)
        .send()
        .await?;
    println!("artist = {}", artist.name);

    let lyric = client
        .playback()
        .lyric()
        .id("108914")
        .platform(Platform::Netease)
        .send()
        .await?;
    println!("lyric length = {}", lyric.lyric.len());

    let url = client
        .playback()
        .url()
        .id("108914")
        .level(SongQuality::Standard)
        .platform(Platform::Netease)
        .send()
        .await?;
    println!("play url = {}", url.url);

    Ok(())
}
```

### 3) 发现与歌单

```rust
use music_bridge_rs::client::MusicClient;
use music_bridge_rs::models::Platform;

async fn demo(client: &MusicClient) -> Result<(), music_bridge_rs::error::MusicClientError> {
    let toplists = client
        .discover()
        .toplist_list()
        .platform(Platform::Netease)
        .send()
        .await?;
    println!("toplists = {}", toplists.toplists.len());

    let playlists = client
        .playlist()
        .list()
        .platform(Platform::Netease)
        .category("流行")
        .limit(20)
        .send()
        .await?;
    println!("playlist list = {}", playlists.playlists.len());

    Ok(())
}
```

### 4) 二维码登录（QQ 音乐）+ 登录态请求

```rust
use music_bridge_rs::client::MusicClient;
use music_bridge_rs::models::{LoginStatus, LoginToken, Platform};

async fn demo(client: &MusicClient) -> Result<(), music_bridge_rs::error::MusicClientError> {
    let session = client
        .login()
        .session()
        .platform(Platform::Tencent)
        .send()
        .await?;

    // qr_code 为 data:image/png;base64,...，可渲染后扫码
    println!("qr_key: {}", session.qr_key());
    println!("qr_code: {}", session.qr_code());

    loop {
        match session.status().await? {
            LoginStatus::WaitingScan => println!("waiting scan..."),
            LoginStatus::WaitingConfirm => println!("waiting confirm..."),
            LoginStatus::QrCodeExpired => break,
            LoginStatus::Success(LoginToken::Tencent(token)) => {
                let recommend = client
                    .discover()
                    .recommend_playlist()
                    .platform(Platform::Tencent)
                    .login(&token)
                    .send()
                    .await?;
                println!("recommend playlists = {}", recommend.playlists.len());
                break;
            }
            LoginStatus::Success(LoginToken::Netease(_)) => {}
        }
    }

    Ok(())
}
```

### 5) Token 刷新

```rust
use music_bridge_rs::client::MusicClient;
use music_bridge_rs::models::{LoginToken, Platform, TencentLoginToken};

async fn refresh_qq_token(
    client: &MusicClient,
    token: &TencentLoginToken,
) -> Result<LoginToken, music_bridge_rs::error::MusicClientError> {
    client.login().refresh().platform(Platform::Tencent).token(token).send().await
}
```

## 错误处理

统一错误类型：`music_bridge_rs::error::MusicClientError`。

常见错误：

- `MissingKeyword`：搜索关键词缺失
- `MissingId`：详情/播放等请求缺少 ID
- `MissingCategory`：歌单列表缺少分类
- `MissingRefreshToken`：刷新登录态时缺少 token
- `InvalidCategoryId`：QQ 音乐歌单分类参数无法解析为合法分类 ID
- `AuthTokenPlatformMismatch`：平台与传入 token 不匹配
- `NeteaseUnexpectedLoginStatus`：网易二维码登录状态码不在已知范围
- `NeteaseLoginTokenInvalid`：网易登录/刷新返回成功但无法解析出有效 token cookie
- `InvalidTencentLoginTokenField`：QQ 登录/刷新响应字段缺失或非法
- `TencentLoginCanceled`：QQ 扫码登录被用户取消
- `TencentLoginFailed`：QQ 扫码登录失败
- `TencentMqttLogin`：QQ MQTT 登录链路异常

示例：

```rust
use music_bridge_rs::client::MusicClient;
use music_bridge_rs::error::MusicClientError;

async fn demo(client: &MusicClient) {
    let result = client.search().send().await;
    if let Err(MusicClientError::MissingKeyword) = result {
        println!("keyword is required");
    }
}
```

## 测试与检查

```bash
# 编译检查
cargo check

# 格式化
cargo +nightly fmt

# lint
cargo clippy --all-targets --all-features -- -D warnings

# 默认测试（不含 ignore 的在线测试）
cargo test

# 包含在线依赖/人工交互流程测试
cargo test -- --ignored
```

## 项目结构

```text
src/
  client/      # 对外链式 API 与请求构建器
  platform/    # 平台内部实现（netease / tencent）
  models.rs    # 跨平台统一数据模型
  error.rs     # 统一错误定义
  lib.rs       # 模块导出
```

## 版权与合规

- 本库仅封装客户端调用流程与数据模型，不内置、不分发任何音频文件、曲库资源或版权内容。
- 不提供破解、绕过付费/会员限制、绕过 DRM、批量抓取或去除版权标识等能力。
- 涉及账号登录与接口调用时，使用者应确保账号、Token 与访问行为合法合规，并获得必要授权。
- 请勿将本库用于未获授权的传播、下载、转存或再分发音乐内容。

## 许可证

本项目基于 [MIT License](./LICENSE) 许可证发布。

## 免责声明

因使用（或无法使用）本项目导致的任何直接或间接损失、数据问题、账号风险、服务封禁或法律责任，均由使用者自行承担。

## 注意事项

- 该库不对外暴露 `platform` 实现细节，请通过 `client` 与 `models` 使用能力。
- 登录 token 含敏感信息，请勿提交到仓库或日志系统。
- 若要手动构造 token，请使用 `NeteaseLoginToken::new(...)` / `TencentLoginToken::new(...)`，不要依赖内部字段。
