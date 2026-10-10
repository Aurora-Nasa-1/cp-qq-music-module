//! CPPlayer 音源模块（JNI）入口 —— **QQ 音乐**。
//!
//! # 它是什么
//!
//! 把本库的统一 [`MusicClient`] 封装成 CPPlayer 宿主认识的 JNI 音源模块
//! （`manifest.json` 的 `type` 为 `"jni"`）。宿主侧实现见
//! `CPPlayer-KMP/core/src/jvmMain/kotlin/cp/player/core/provider/JniProvider.kt`，
//! 契约文档见 `CPPlayer-KMP/docs/dev/PROVIDER_DEV_GUIDE.md` §3.3。
//!
//! # ⚠️ 导出符号名与宿主 Java 类的全限定名硬绑定
//!
//! JNI 按 `Java_<包名下划线化>_<类名>_<方法名>` 查找符号。宿主类当前是
//! `cp.player.core.provider.JniProvider`，所以下面三个函数的前缀必须是
//! `Java_cp_player_core_provider_JniProvider_`。
//!
//! 历史沿革：`cp.player.provider` → `cp.player.kmp.provider` → `cp.player.core.provider`。
//! **宿主换包名时这里必须同步改并重新编译模块**，否则 `.so`/`.dll` 仍能被
//! `System.load()` 加载成功，但首次方法调用会抛 `UnsatisfiedLinkError`
//! （症状是「模块显示已加载，一调用就崩」）。
//!
//! # 调用链
//!
//! ```text
//! 宿主 MusicApiService
//!   └─ ProviderManager.callApi(标准方法名, params)   // 按 manifest.apiMap 映射
//!        └─ JniProvider.nativeCallApi(method, paramsJson)   // JNI
//!             └─ 本模块 dispatch()：方法名 → MusicClient(Platform::Tencent)
//!                  └─ 统一模型 → 宿主期望的 JSON（网易云风格 code/data/songs/ar/al/lrc）
//! ```
//!
//! # 平台
//!
//! 本模块**固定** [`Platform::Tencent`]：模块包即「QQ 音乐音源」。
//! 网易云音源请用单独编译/打包的模块，不要在本模块里切平台。
//!
//! # 登录
//!
//! 宿主把账号凭据放在 `params["cookie"]`（形如
//! `uin=...; qqmusic_key=...; qm_keyst=...; tmeLoginType=...`）。
//! 这里解析成 [`TencentLoginToken`] 后注入构建器；解析失败则按未登录处理。
//! 扫码登录走 `login/qr/key` → `login/qr/create` → `login/qr/check` 三步，
//! 会话状态保存在模块内的全局槽位。

use std::collections::HashMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::OnceLock;

use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jint, jstring};
use serde_json::{Map, Value, json};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;

use crate::client::LoginSession;
use crate::error::MusicClientError;
use crate::models::{
    Album, AlbumDetailResult, Artist, ArtistDetailResult, LoginStatus, LoginToken, LyricResult,
    Platform, Playlist, PlaylistCategoriesResult, PlaylistDetailResult, PlaylistListResult,
    SearchAlbumResult, SearchArtistResult, SearchPlaylistResult, SearchSongResult, Song,
    SongQuality, SongsDetailResult, TencentLoginToken, ToplistListResult, UrlResult,
};
use crate::MusicClient;

/// 本模块固定服务的平台（QQ 音乐）。
const PLATFORM: Platform = Platform::Tencent;

/// 全局 tokio 运行时（JNI 调用是同步的，需要 `block_on`）。
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// 全局客户端单例：复用 reqwest 连接池，避免每次调用都重建 TLS。
static CLIENT: OnceLock<MusicClient> = OnceLock::new();

/// 扫码登录会话槽位。
static LOGIN: OnceLock<Mutex<LoginState>> = OnceLock::new();

fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(4)
            .thread_name("cp-qq-module")
            .build()
            .expect("failed to create tokio runtime")
    })
}

fn client() -> &'static MusicClient {
    CLIENT.get_or_init(MusicClient::new)
}

/// 扫码登录会话的全局槽位。
///
/// `LoginSession` 内部含 MQTT 会话（QQ 扫码登录需要），未标注 `Send`。
/// 这里显式断言其可跨线程持有：宿主对同一 Provider 的调用是串行的
/// （`ProviderManager.callApi` 跑在 `Dispatchers.IO`），且槽位全程由
/// [`tokio::sync::Mutex`] 保护，不存在并发访问。
struct LoginState(Option<Box<LoginSession<'static>>>);

#[allow(unsafe_code)]
unsafe impl Send for LoginState {}

fn login_slot() -> &'static Mutex<LoginState> {
    LOGIN.get_or_init(|| Mutex::new(LoginState(None)))
}

// ===========================================================================
// JNI 导出
// ===========================================================================

/// 启动本地服务。
///
/// 本模块不启动独立 HTTP 服务（宿主直接通过 [`nativeCallApi`] 调用），
/// 这里只做一次性初始化（创建 tokio 运行时）。保留该导出是为了满足宿主契约。
///
/// # Safety
///
/// 由 JVM 通过 JNI 调用；参数由 JVM 保证有效。
#[unsafe(no_mangle)]
#[allow(unsafe_code, non_snake_case)]
pub unsafe extern "system" fn Java_cp_player_core_provider_JniProvider_startNativeServer(
    _env: JNIEnv,
    _this: JClass,
    _host: JString,
    _port: jint,
) {
    let _ = runtime();
}

/// 分发一次标准化 API 调用，返回 JSON 字符串。
///
/// `method` 是宿主 `manifest.apiMap` 映射后的方法名（见 [`dispatch`]）；
/// `params_json` 是 `{"k":"v"}` 形式的字符串参数表。
///
/// # Safety
///
/// 由 JVM 通过 JNI 调用；参数由 JVM 保证有效。
#[unsafe(no_mangle)]
#[allow(unsafe_code, non_snake_case)]
pub unsafe extern "system" fn Java_cp_player_core_provider_JniProvider_nativeCallApi(
    mut env: JNIEnv,
    _this: JClass,
    method: JString,
    params_json: JString,
) -> jstring {
    let method: String = env.get_string(&method).map(Into::into).unwrap_or_default();
    let params_json: String = env
        .get_string(&params_json)
        .map(Into::into)
        .unwrap_or_else(|_| "{}".to_string());

    // JNI 边界必须拦住 panic：库代码里的 panic 若跨 FFI 展开是未定义行为，
    // 在宿主进程里表现为「调用即崩」。这里统一转成错误 JSON。
    let result = catch_unwind(AssertUnwindSafe(|| {
        let params = parse_params(&params_json);
        runtime().block_on(dispatch(&method, &params))
    }))
    .unwrap_or_else(|_| error_json("模块内部错误（已捕获 panic）"));

    match env.new_string(result) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

/// 音频分析（本模块不提供）。
///
/// 宿主把它当作可选能力：返回错误 JSON 即可，不影响播放。
///
/// # Safety
///
/// 由 JVM 通过 JNI 调用；参数由 JVM 保证有效。
#[unsafe(no_mangle)]
#[allow(unsafe_code, non_snake_case)]
pub unsafe extern "system" fn Java_cp_player_core_provider_JniProvider_analyzeAudioFile(
    env: JNIEnv,
    _this: JClass,
    _path: JString,
) -> jstring {
    let payload = json!({ "error": "audio analysis is not supported by this module" }).to_string();
    match env.new_string(payload) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

// ===========================================================================
// 分发
// ===========================================================================

/// 把宿主标准方法名翻译成 [`MusicClient`] 调用，并返回宿主期望的 JSON。
///
/// 支持的方法集见 [`handle`]；未列出的方法返回 `code: 501`。
/// 强烈建议在 `manifest.json` 的 `apiMap` 里把不支持的方法标为 `"unsupported"`，
/// 让宿主**直接短路**（记为 WARNING 而非 ERROR），而不是发到这里再报错。
async fn dispatch(method: &str, params: &HashMap<String, String>) -> String {
    let token = tencent_token_from_params(params);
    match handle(method, params, token.as_ref()).await {
        Ok(value) => value.to_string(),
        Err(err) => error_json(&err.to_string()),
    }
}

async fn handle(
    method: &str,
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    match method {
        // ---- 搜索 ----
        "cloudsearch" => search(params, token).await,
        "search/hot/detail" => hotkey(token).await,
        "search/suggest" => suggests(params, token).await,
        // ---- 详情 ----
        "song/detail" => song_detail(params, token).await,
        "album" => album_detail(params, token).await,
        "artist/detail" | "artist/songs" | "artist/top/song" => artist_detail(params, token).await,
        // ---- 播放 ----
        "song/url/v1" | "song/url/v1/302" | "song/download/url/v1" => song_url(params, token).await,
        "lyric/new" => lyric(params, token).await,
        // ---- 歌单 ----
        "playlist/detail" => playlist_detail(params, token).await,
        "playlist/track/all" => playlist_tracks(params, token).await,
        "playlist/catlist" => playlist_categories(token).await,
        "top_playlist" | "playlist/list" => playlist_list(params, token).await,
        // ---- 发现 ----
        "toplist" | "toplist_detail" => toplist(token).await,
        "personalized" => recommend_playlist(token).await,
        // ---- 登录 ----
        "login/qr/key" | "login/qr/create" => qr_create().await,
        "login/qr/check" => qr_check().await,
        other => Ok(json!({
            "code": 501,
            "msg": format!("QQ 音源模块不支持该方法: {other}"),
        })),
    }
}

// ===========================================================================
// 各方法实现
// ===========================================================================

async fn search(
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    let keyword = first_param(params, &["keywords", "keyword"]).unwrap_or_default();
    let limit = param_u64(params, "limit", 30);
    let offset = param_u64(params, "offset", 0);
    let kind = param_u64(params, "type", 1);
    let c: &MusicClient = client();

    match kind {
        // 100 = 歌手
        100 => {
            let mut req =
                c.search().artist().keyword(keyword).limit(limit).offset(offset).platform(PLATFORM);
            if let Some(t) = token {
                req = req.login(t);
            }
            let res: SearchArtistResult = req.send().await?;
            let artists: Vec<Value> = res.artists.iter().map(host_artist).collect();
            Ok(json!({"code": 200, "result": {
                "artists": artists,
                "artistCount": res.artists.len(),
            }}))
        }
        // 10 = 专辑
        10 => {
            let mut req =
                c.search().album().keyword(keyword).limit(limit).offset(offset).platform(PLATFORM);
            if let Some(t) = token {
                req = req.login(t);
            }
            let res: SearchAlbumResult = req.send().await?;
            let albums: Vec<Value> = res.albums.iter().map(host_album).collect();
            Ok(json!({"code": 200, "result": {
                "albums": albums,
                "albumCount": res.albums.len(),
            }}))
        }
        // 1000 = 歌单
        1000 => {
            let mut req = c
                .search()
                .playlist()
                .keyword(keyword)
                .limit(limit)
                .offset(offset)
                .platform(PLATFORM);
            if let Some(t) = token {
                req = req.login(t);
            }
            let res: SearchPlaylistResult = req.send().await?;
            let playlists: Vec<Value> = res.playlists.iter().map(host_playlist).collect();
            Ok(json!({"code": 200, "result": {
                "playlists": playlists,
                "playlistCount": res.playlists.len(),
            }}))
        }
        // 其余（含 1 = 单曲）
        _ => {
            let mut req =
                c.search().song().keyword(keyword).limit(limit).offset(offset).platform(PLATFORM);
            if let Some(t) = token {
                req = req.login(t);
            }
            let res: SearchSongResult = req.send().await?;
            let songs: Vec<Value> = res.songs.iter().map(host_song).collect();
            Ok(json!({"code": 200, "result": {
                "songs": songs,
                "songCount": res.songs.len(),
            }}))
        }
    }
}

async fn song_detail(
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    let ids = split_ids(first_param(params, &["ids", "id"]).unwrap_or_default());
    let mut req = client().detail().song().ids(ids).platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res: SongsDetailResult = req.send().await?;
    let songs: Vec<Value> = res.songs.iter().map(host_song).collect();
    Ok(json!({"code": 200, "songs": songs}))
}

async fn song_url(
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    let id = first_param(params, &["id"]).unwrap_or_default();
    let level_raw = params.get("level").map(String::as_str).unwrap_or("standard");
    let level = map_level(level_raw);
    let mut req = client().playback().url().id(id.clone()).level(level).platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res: UrlResult = req.send().await?;
    // 空 URL = 该曲目在当前账号/版权下不可播放：按宿主约定返回 `url: null`（不算错误），
    // 避免健康监控把「无版权」记成 ERROR。
    let url = if res.url.trim().is_empty() { Value::Null } else { Value::String(res.url) };
    Ok(json!({"code": 200, "data": [{
        "id": id,
        "url": url,
        "br": 0,
        "size": 0,
        "type": "mp3",
        "level": level_raw,
    }]}))
}

async fn lyric(
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    let id = first_param(params, &["id"]).unwrap_or_default();
    let mut req = client().playback().lyric().id(id).platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res: LyricResult = req.send().await?;
    Ok(json!({"code": 200, "lrc": {"lyric": res.lyric}, "tlyric": {"lyric": res.trans_lyric.unwrap_or_default()}}))
}

async fn playlist_detail(
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    let id = first_param(params, &["id"]).unwrap_or_default();
    let res = fetch_playlist(&id, token).await?;
    let songs: Vec<Value> = res.songs.iter().map(host_song).collect();
    Ok(json!({"code": 200, "playlist": {
        "id": res.id,
        "name": res.name,
        "coverImgUrl": res.pic_url,
        "trackCount": res.songs.len(),
        "description": res.description,
        "creator": {"nickname": ""},
        "tracks": songs,
    }}))
}

async fn playlist_tracks(
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    let id = first_param(params, &["id"]).unwrap_or_default();
    let res = fetch_playlist(&id, token).await?;
    let songs: Vec<Value> = res.songs.iter().map(host_song).collect();
    Ok(json!({"code": 200, "songs": songs}))
}

async fn fetch_playlist(
    id: &str,
    token: Option<&TencentLoginToken>,
) -> Result<PlaylistDetailResult, MusicClientError> {
    let mut req = client().playlist().detail().id(id.to_string()).platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    req.send().await
}

async fn album_detail(
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    let id = first_param(params, &["id"]).unwrap_or_default();
    let mut req = client().detail().album().id(id).platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res: AlbumDetailResult = req.send().await?;
    let artist = res
        .songs
        .first()
        .and_then(|s| s.artists.first())
        .map(|a| json!({"id": a.id, "name": a.name}))
        .unwrap_or_else(|| json!({"id": "", "name": ""}));
    let songs: Vec<Value> = res.songs.iter().map(host_song).collect();
    Ok(json!({"code": 200,
        "album": {
            "id": res.id,
            "name": res.name,
            "picUrl": res.pic_url,
            "description": res.description,
            "artist": artist,
        },
        "songs": songs,
    }))
}

async fn artist_detail(
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    let id = first_param(params, &["id"]).unwrap_or_default();
    let limit = param_u64(params, "limit", 50);
    let mut req = client().detail().artist().id(id).limit(limit).platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res: ArtistDetailResult = req.send().await?;
    let songs: Vec<Value> = res.songs.iter().map(host_song).collect();
    Ok(json!({"code": 200,
        "data": {"artist": {
            "id": res.id,
            "name": res.name,
            "picUrl": res.pic_url,
            "briefDesc": res.description,
            "albumSize": 0,
            "musicSize": res.songs.len(),
        }},
        "songs": songs,
    }))
}

async fn playlist_categories(token: Option<&TencentLoginToken>) -> Result<Value, MusicClientError> {
    let mut req = client().playlist().categories().platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res: PlaylistCategoriesResult = req.send().await?;
    let mut categories = Map::new();
    for (id, name) in &res.group {
        categories.insert(id.to_string(), json!(name));
    }
    let sub: Vec<Value> = res
        .categories
        .iter()
        .map(|c| json!({"id": c.id, "name": c.name, "category": c.category}))
        .collect();
    Ok(json!({"code": 200, "categories": categories, "sub": sub}))
}

async fn playlist_list(
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    let category = first_param(params, &["cat", "category"]).unwrap_or_default();
    let limit = param_u64(params, "limit", 30);
    let offset = param_u64(params, "offset", 0);
    let mut req = client()
        .playlist()
        .list()
        .category(category)
        .limit(limit)
        .offset(offset)
        .platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res: PlaylistListResult = req.send().await?;
    let playlists: Vec<Value> = res.playlists.iter().map(host_playlist).collect();
    Ok(json!({"code": 200, "playlists": playlists, "more": res.more}))
}

async fn toplist(token: Option<&TencentLoginToken>) -> Result<Value, MusicClientError> {
    let mut req = client().discover().toplist_list().platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res: ToplistListResult = req.send().await?;
    let list: Vec<Value> = res
        .toplists
        .iter()
        .map(|t| {
            let tracks: Vec<Value> = t
                .tracks
                .as_ref()
                .map(|tr| tr.iter().map(|x| json!({"first": x.title, "second": x.artist})).collect())
                .unwrap_or_default();
            json!({
                "id": t.id,
                "name": t.name,
                "coverImgUrl": t.pic_url.clone().unwrap_or_default(),
                "tracks": tracks,
            })
        })
        .collect();
    Ok(json!({"code": 200, "list": list}))
}

async fn recommend_playlist(token: Option<&TencentLoginToken>) -> Result<Value, MusicClientError> {
    let mut req = client().discover().recommend_playlist().platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res = req.send().await?;
    let playlists: Vec<Value> = res.playlists.iter().map(host_playlist).collect();
    Ok(json!({"code": 200, "result": playlists}))
}

async fn hotkey(token: Option<&TencentLoginToken>) -> Result<Value, MusicClientError> {
    let mut req = client().discover().hotkey().platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res = req.send().await?;
    let data: Vec<Value> = res.hotkey.iter().map(|w| json!({"searchWord": w})).collect();
    Ok(json!({"code": 200, "data": data}))
}

async fn suggests(
    params: &HashMap<String, String>,
    token: Option<&TencentLoginToken>,
) -> Result<Value, MusicClientError> {
    let keyword = first_param(params, &["keywords", "keyword"]).unwrap_or_default();
    let mut req = client().discover().suggests().keyword(keyword).platform(PLATFORM);
    if let Some(t) = token {
        req = req.login(t);
    }
    let res = req.send().await?;
    let all: Vec<Value> = res.suggests.iter().map(|s| json!({"keyword": s})).collect();
    Ok(json!({"code": 200, "result": {"allMatch": all}}))
}

// ---- 登录 ----

/// 创建（或复用）扫码会话，返回二维码。
///
/// 同时服务 `login/qr/key` 与 `login/qr/create`：宿主两步都会拿到同一张二维码。
async fn qr_create() -> Result<Value, MusicClientError> {
    let mut slot = login_slot().lock().await;
    if slot.0.is_none() {
        let session: LoginSession<'static> =
            client().login().session().platform(PLATFORM).send().await?;
        slot.0 = Some(Box::new(session));
    }
    let qrimg = slot.0.as_ref().map(|s| s.qr_code().to_string()).unwrap_or_default();
    Ok(json!({"code": 200, "data": {"unikey": "cp-qq", "qrimg": qrimg, "qrurl": ""}}))
}

/// 轮询扫码状态，映射到宿主约定状态码。
async fn qr_check() -> Result<Value, MusicClientError> {
    let mut slot = login_slot().lock().await;
    let Some(session) = slot.0.as_ref() else {
        return Ok(json!({"code": 800, "msg": "二维码已过期，请重新获取"}));
    };
    let status = session.status().await?;
    let value = match status {
        LoginStatus::Success(token) => {
            let cookie = match &token {
                LoginToken::Tencent(t) => t.to_cookie(),
                LoginToken::Netease(t) => t.to_cookie(),
            };
            // 登录成功即清空会话，避免下次扫码复用旧会话。
            slot.0 = None;
            json!({"code": 803, "cookie": cookie})
        }
        LoginStatus::WaitingScan => json!({"code": 801}),
        LoginStatus::WaitingConfirm => json!({"code": 802}),
        LoginStatus::QrCodeExpired => {
            slot.0 = None;
            json!({"code": 800, "msg": "二维码已过期，请重新获取"})
        }
    };
    Ok(value)
}

// ===========================================================================
// 宿主 JSON 形状转换
// ===========================================================================

/// 统一 [`Song`] → 宿主曲目 JSON（网易云风格 `ar` / `al` / `dt`）。
///
/// 宿主解析见 `CPPlayer-KMP/.../music/TrackJsonMapper.kt`：
/// 歌手取 `ar`/`artists`，专辑取 `al`/`album`，封面取 `al.picUrl`，时长取 `dt`/`duration`。
fn host_song(song: &Song) -> Value {
    json!({
        "id": song.id,
        "name": song.name,
        "dt": song.duration_ms,
        "ar": song.artists.iter().map(|a| json!({"id": a.id, "name": a.name})).collect::<Vec<_>>(),
        "al": {"id": song.album.id, "name": song.album.name, "picUrl": song.pic_url},
    })
}

fn host_artist(artist: &Artist) -> Value {
    json!({"id": artist.id, "name": artist.name, "picUrl": artist.pic_url})
}

fn host_album(album: &Album) -> Value {
    json!({
        "id": album.id,
        "name": album.name,
        "picUrl": album.pic_url,
        "artist": {"id": album.artist.id, "name": album.artist.name},
    })
}

fn host_playlist(playlist: &Playlist) -> Value {
    json!({
        "id": playlist.id,
        "name": playlist.name,
        "coverImgUrl": playlist.pic_url,
        "picUrl": playlist.pic_url,
        "trackCount": 0,
        "creator": {"nickname": ""},
    })
}

// ===========================================================================
// 参数解析
// ===========================================================================

fn parse_params(raw: &str) -> HashMap<String, String> {
    serde_json::from_str::<HashMap<String, String>>(raw).unwrap_or_default()
}

fn first_param(params: &HashMap<String, String>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|k| params.get(*k)).filter(|v| !v.trim().is_empty()).cloned()
}

fn param_u64(params: &HashMap<String, String>, key: &str, default: u64) -> u64 {
    params.get(key).and_then(|v| v.trim().parse::<u64>().ok()).unwrap_or(default)
}

fn split_ids(raw: String) -> Vec<String> {
    raw.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// 宿主音质串 → 本库 [`SongQuality`]。
fn map_level(level: &str) -> SongQuality {
    match level.to_ascii_lowercase().as_str() {
        "lossless" => SongQuality::Lossless,
        "hires" => SongQuality::Hires,
        "jymaster" | "master" => SongQuality::Master,
        "sky" | "dolby" => SongQuality::Surround,
        "stereo" => SongQuality::Stereo,
        "exhigh" => SongQuality::Exhigh,
        _ => SongQuality::Standard,
    }
}

/// 从宿主 `cookie` 参数解析 QQ 音乐登录态。
///
/// 形如 `uin=123; qqmusic_key=xxx; qm_keyst=xxx; tmeLoginType=1`。
/// 缺少 `uin` 或 `qqmusic_key` 时返回 [`None`]（按未登录处理）。
fn tencent_token_from_params(params: &HashMap<String, String>) -> Option<TencentLoginToken> {
    let cookie = params.get("cookie")?;
    if cookie.trim().is_empty() {
        return None;
    }
    let mut uin: Option<u64> = None;
    let mut music_key: Option<String> = None;
    let mut login_type: u64 = 1;
    for part in cookie.split(';') {
        let Some((key, value)) = part.trim().split_once('=') else {
            continue;
        };
        let value = value.trim();
        match key.trim() {
            "uin" => uin = value.parse::<u64>().ok(),
            "qqmusic_key" => music_key = Some(value.to_string()),
            "qm_keyst" if music_key.is_none() => music_key = Some(value.to_string()),
            "tmeLoginType" => login_type = value.parse::<u64>().unwrap_or(1),
            _ => {}
        }
    }
    match (uin, music_key) {
        (Some(uin), Some(key)) if !key.is_empty() => Some(TencentLoginToken::new(
            uin,
            key,
            String::new(),
            String::new(),
            None,
            login_type,
        )),
        _ => None,
    }
}

fn error_json(msg: &str) -> String {
    json!({"code": 500, "msg": msg}).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_level_covers_host_vocabulary() {
        assert_eq!(map_level("lossless"), SongQuality::Lossless);
        assert_eq!(map_level("hires"), SongQuality::Hires);
        assert_eq!(map_level("jymaster"), SongQuality::Master);
        assert_eq!(map_level("sky"), SongQuality::Surround);
        assert_eq!(map_level("exhigh"), SongQuality::Exhigh);
        assert_eq!(map_level("standard"), SongQuality::Standard);
        assert_eq!(map_level("higher"), SongQuality::Standard);
        assert_eq!(map_level("unknown"), SongQuality::Standard);
    }

    #[test]
    fn tencent_token_parses_host_cookie() {
        let mut params = HashMap::new();
        params.insert(
            "cookie".to_string(),
            "uin=12345; qqmusic_key=abc; qm_keyst=abc; tmeLoginType=1".to_string(),
        );
        let token = tencent_token_from_params(&params).expect("cookie should parse");
        assert!(token.to_cookie().contains("uin=12345"));
        assert!(token.to_cookie().contains("qqmusic_key=abc"));
    }

    #[test]
    fn tencent_token_missing_fields_returns_none() {
        let mut params = HashMap::new();
        params.insert("cookie".to_string(), "qqmusic_key=abc".to_string());
        assert!(tencent_token_from_params(&params).is_none());
    }

    #[test]
    fn host_song_shape_matches_host_contract() {
        let song = Song {
            id: "0039".to_string(),
            name: "Song".to_string(),
            pic_url: "https://img".to_string(),
            artists: vec![crate::models::SongArtist { id: "1".to_string(), name: "A".to_string() }],
            album: crate::models::SongAlbum { id: "2".to_string(), name: "Al".to_string() },
            duration_ms: 269_000,
        };
        let value = host_song(&song);
        assert_eq!(value["id"], "0039");
        assert_eq!(value["dt"], 269_000);
        assert_eq!(value["ar"][0]["name"], "A");
        assert_eq!(value["al"]["picUrl"], "https://img");
    }
}
