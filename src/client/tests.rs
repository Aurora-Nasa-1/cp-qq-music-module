use super::*;
use crate::error::MusicClientError;
use crate::models::{
    LoginStatus, LoginToken, NeteaseLoginToken, Platform, SongQuality, TencentLoginToken,
};

const JIANG_NAN_SONG_ID: &str = "108914";
const JJ_ARTIST_ID: &str = "3684";
const SECOND_HEAVEN_ALBUM_ID: &str = "10804";
const CLASSIC_OLD_SONGS_PLAYLIST_ID: u64 = 8903867087;
const CLASSIC_OLD_SONGS_PLAYLIST_NAME: &str = "经典老歌500首大全｜回忆青春岁月";

fn mock_tencent_token() -> TencentLoginToken {
    TencentLoginToken {
        music_id: 1234,
        music_key: "music_key".to_owned(),
        refresh_token: "refresh_token".to_owned(),
        refresh_key: "refresh_key".to_owned(),
        expires_at: None,
        login_type: 0,
    }
}

fn mock_netease_token() -> NeteaseLoginToken {
    NeteaseLoginToken::new("music_u", "music_r_u", "csrf", None)
}

#[tokio::test(flavor = "current_thread")]
async fn search_requires_keyword() {
    let client = MusicClient::new();
    let err = client.search().send().await.expect_err("missing keyword should fail");
    assert!(matches!(err, MusicClientError::MissingKeyword));
}

#[tokio::test(flavor = "current_thread")]
async fn search_rejects_empty_keyword() {
    let client = MusicClient::new();
    let err = client.search().keyword("   ").send().await.expect_err("empty keyword should fail");
    assert!(matches!(err, MusicClientError::MissingKeyword));
}

#[tokio::test(flavor = "current_thread")]
async fn search_rejects_platform_token_mismatch() {
    let client = MusicClient::new();
    let tencent_token = mock_tencent_token();
    let err = client
        .search()
        .keyword("江南")
        .platform(Platform::Netease)
        .login(&tencent_token)
        .send()
        .await
        .expect_err("tencent token on netease platform should fail");

    assert!(matches!(
        err,
        MusicClientError::AuthTokenPlatformMismatch {
            expected_platform: "netease",
            token_platform: "tencent",
        }
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn search_rejects_platform_token_mismatch_reverse() {
    let client = MusicClient::new();
    let netease_token = mock_netease_token();
    let err = client
        .search()
        .keyword("江南")
        .platform(Platform::Tencent)
        .login(&netease_token)
        .send()
        .await
        .expect_err("netease token on tencent platform should fail");

    assert!(matches!(
        err,
        MusicClientError::AuthTokenPlatformMismatch {
            expected_platform: "tencent",
            token_platform: "netease",
        }
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn detail_rejects_platform_token_mismatch() {
    let client = MusicClient::new();
    let tencent_token = mock_tencent_token();
    let err = client
        .detail()
        .artist()
        .id("1")
        .platform(Platform::Netease)
        .login(&tencent_token)
        .send()
        .await
        .expect_err("tencent token on netease platform should fail");

    assert!(matches!(
        err,
        MusicClientError::AuthTokenPlatformMismatch {
            expected_platform: "netease",
            token_platform: "tencent",
        }
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn discover_rejects_platform_token_mismatch() {
    let client = MusicClient::new();
    let tencent_token = mock_tencent_token();
    let err = client
        .discover()
        .hotkey()
        .platform(Platform::Netease)
        .login(&tencent_token)
        .send()
        .await
        .expect_err("tencent token on netease platform should fail");

    assert!(matches!(
        err,
        MusicClientError::AuthTokenPlatformMismatch {
            expected_platform: "netease",
            token_platform: "tencent",
        }
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn playback_rejects_platform_token_mismatch() {
    let client = MusicClient::new();
    let tencent_token = mock_tencent_token();
    let err = client
        .playback()
        .lyric()
        .id("1")
        .platform(Platform::Netease)
        .login(&tencent_token)
        .send()
        .await
        .expect_err("tencent token on netease platform should fail");

    assert!(matches!(
        err,
        MusicClientError::AuthTokenPlatformMismatch {
            expected_platform: "netease",
            token_platform: "tencent",
        }
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn detail_requires_id() {
    let client = MusicClient::new();
    let err = client.detail().artist().send().await.expect_err("missing id should fail");
    assert!(matches!(err, MusicClientError::MissingId));
}

#[tokio::test(flavor = "current_thread")]
async fn detail_rejects_empty_id() {
    let client = MusicClient::new();
    let err = client.detail().artist().id("   ").send().await.expect_err("empty id should fail");
    assert!(matches!(err, MusicClientError::MissingId));
}

#[tokio::test(flavor = "current_thread")]
async fn detail_album_rejects_empty_id() {
    let client = MusicClient::new();
    let err = client.detail().album().id("").send().await.expect_err("empty id should fail");
    assert!(matches!(err, MusicClientError::MissingId));
}

#[tokio::test(flavor = "current_thread")]
async fn detail_toplist_requires_id() {
    let client = MusicClient::new();
    let err = client.detail().toplist().send().await.expect_err("missing id should fail");
    assert!(matches!(err, MusicClientError::MissingId));
}

#[tokio::test(flavor = "current_thread")]
async fn discover_playlist_list_requires_category() {
    let client = MusicClient::new();
    let err =
        client.discover().playlist_list().send().await.expect_err("missing category should fail");
    assert!(matches!(err, MusicClientError::MissingCategory));
}

#[tokio::test(flavor = "current_thread")]
async fn discover_playlist_list_rejects_empty_category() {
    let client = MusicClient::new();
    let err = client
        .discover()
        .playlist_list()
        .category(" ")
        .send()
        .await
        .expect_err("empty category should fail");
    assert!(matches!(err, MusicClientError::MissingCategory));
}

#[tokio::test(flavor = "current_thread")]
async fn auth_refresh_requires_token() {
    let client = MusicClient::new();
    let err = client.login().refresh().send().await.expect_err("missing refresh token should fail");
    assert!(matches!(err, MusicClientError::MissingRefreshToken));
}

#[tokio::test(flavor = "current_thread")]
async fn auth_refresh_rejects_platform_token_mismatch() {
    let client = MusicClient::new();
    let tencent_token = mock_tencent_token();
    let err = client
        .login()
        .refresh()
        .platform(Platform::Netease)
        .token(&tencent_token)
        .send()
        .await
        .expect_err("tencent token on netease platform should fail");

    assert!(matches!(
        err,
        MusicClientError::AuthTokenPlatformMismatch {
            expected_platform: "netease",
            token_platform: "tencent",
        }
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn auth_refresh_rejects_platform_token_mismatch_reverse() {
    let client = MusicClient::new();
    let netease_token = mock_netease_token();
    let err = client
        .login()
        .refresh()
        .platform(Platform::Tencent)
        .token(&netease_token)
        .send()
        .await
        .expect_err("netease token on tencent platform should fail");

    assert!(matches!(
        err,
        MusicClientError::AuthTokenPlatformMismatch {
            expected_platform: "tencent",
            token_platform: "netease",
        }
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn playlist_detail_requires_id() {
    let client = MusicClient::new();
    let err = client.playlist().detail().send().await.expect_err("missing id should fail");
    assert!(matches!(err, MusicClientError::MissingId));
}

#[tokio::test(flavor = "current_thread")]
async fn playlist_list_requires_category() {
    let client = MusicClient::new();
    let err = client.playlist().list().send().await.expect_err("missing category should fail");
    assert!(matches!(err, MusicClientError::MissingCategory));
}

#[tokio::test(flavor = "current_thread")]
async fn playlist_list_rejects_empty_category() {
    let client = MusicClient::new();
    let err =
        client.playlist().list().category("").send().await.expect_err("empty category should fail");
    assert!(matches!(err, MusicClientError::MissingCategory));
}

#[tokio::test(flavor = "current_thread")]
async fn playback_lyric_rejects_empty_id() {
    let client = MusicClient::new();
    let err = client.playback().lyric().id("").send().await.expect_err("empty id should fail");
    assert!(matches!(err, MusicClientError::MissingId));
}

#[tokio::test(flavor = "current_thread")]
async fn playback_url_rejects_empty_id() {
    let client = MusicClient::new();
    let err = client.playback().url().id(" ").send().await.expect_err("empty id should fail");
    assert!(matches!(err, MusicClientError::MissingId));
}

#[tokio::test(flavor = "current_thread")]
async fn playlist_list_rejects_invalid_tencent_category() {
    let client = MusicClient::new();
    let err = client
        .playlist()
        .list()
        .platform(Platform::Tencent)
        .category("not-a-number")
        .send()
        .await
        .expect_err("invalid category should fail");
    assert!(
        matches!(err, MusicClientError::InvalidCategoryId(category) if category == "not-a-number")
    );
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Netease API and may change over time"]
async fn live_search_should_match_expected_ids() {
    let client = MusicClient::new();

    let songs = client
        .search()
        .song()
        .keyword("江南")
        .limit(10)
        .send()
        .await
        .expect("search song should succeed");
    assert!(!songs.songs.is_empty(), "song result should not be empty");
    assert_eq!(songs.songs[0].id, JIANG_NAN_SONG_ID);

    let artists = client
        .search()
        .artist()
        .keyword("林俊杰")
        .limit(10)
        .send()
        .await
        .expect("search artist should succeed");
    assert!(!artists.artists.is_empty(), "artist result should not be empty");
    assert_eq!(artists.artists[0].id, JJ_ARTIST_ID);

    let albums = client
        .search()
        .album()
        .keyword("第二天堂")
        .limit(10)
        .send()
        .await
        .expect("search album should succeed");
    assert!(!albums.albums.is_empty(), "album result should not be empty");
    assert_eq!(albums.albums[0].id, SECOND_HEAVEN_ALBUM_ID);

    let playlists = client
        .search()
        .playlist()
        .keyword("林俊杰")
        .limit(10)
        .send()
        .await
        .expect("search playlist should succeed");
    assert!(!playlists.playlists.is_empty(), "playlist result should not be empty");
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Netease API and may change over time"]
async fn live_playback_should_have_lyric_and_url() {
    let client = MusicClient::new();

    let lyric = client
        .playback()
        .lyric()
        .id(JIANG_NAN_SONG_ID)
        .send()
        .await
        .expect("lyric request should succeed");
    assert!(!lyric.lyric.trim().is_empty(), "lyric should not be empty");

    let url = client
        .playback()
        .url()
        .id(JIANG_NAN_SONG_ID)
        .level(SongQuality::Surround)
        .send()
        .await
        .expect("url request should succeed");
    assert!(!url.url.trim().is_empty(), "playback url should not be empty");
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "requires live Netease API and may change over time"]
async fn live_discover_and_detail_should_match_expected_values() {
    let client = MusicClient::new();
    let limit = 5;

    let playlist_list = client
        .discover()
        .playlist_list()
        .category("流行")
        .limit(limit)
        .send()
        .await
        .expect("playlist list should succeed");
    assert_eq!(playlist_list.playlists.len() as u64, limit);

    let detail = client
        .detail()
        .playlist()
        .id(CLASSIC_OLD_SONGS_PLAYLIST_ID)
        .send()
        .await
        .expect("playlist detail should succeed");
    assert_eq!(detail.name, CLASSIC_OLD_SONGS_PLAYLIST_NAME);
}

fn print_qr_for_scan(raw: &str) {
    use std::io::Write as _;

    use image::GenericImageView;

    const TARGET_COLS: u32 = 32;
    const QUIET_ZONE_MODULES: u32 = 4;
    const PREFIX: &str = "data:image/png;base64,";
    let base64_data = if let Some(data) = raw.strip_prefix(PREFIX) {
        data
    } else {
        println!("{raw}");
        return;
    };

    let bytes = {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD
            .decode(base64_data.as_bytes())
            .expect("qrcode base64 should be valid")
    };

    let image = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
        .expect("qrcode should be a valid png");
    let (width, height) = image.dimensions();
    let scale = width.div_ceil(TARGET_COLS).max(2);
    let scaled_width = width / scale;
    let scaled_height = height / scale;
    let quiet_cols = (QUIET_ZONE_MODULES * 2) as usize;
    let quiet_row = " ".repeat((scaled_width as usize) * 2 + quiet_cols * 2);

    println!();
    for _ in 0..QUIET_ZONE_MODULES {
        println!("{quiet_row}");
    }
    for y in 0..scaled_height {
        print!("{}", " ".repeat(quiet_cols));
        for x in 0..scaled_width {
            let px = image.get_pixel(x * scale, y * scale);
            let brightness = (u32::from(px[0]) + u32::from(px[1]) + u32::from(px[2])) / 3;
            if brightness > 128 {
                print!("  ");
            } else {
                print!("██");
            }
        }
        print!("{}", " ".repeat(quiet_cols));
        println!();
    }
    for _ in 0..QUIET_ZONE_MODULES {
        println!("{quiet_row}");
    }
    println!();
    let _ = std::io::stdout().flush();
}

#[tokio::test(flavor = "current_thread")]
#[ignore = "manual qq login requires terminal qr scan"]
async fn manual_qq_login_via_console_qr() {
    let client = MusicClient::new();
    let session = client
        .login()
        .session()
        .platform(Platform::Tencent)
        .send()
        .await
        .expect("should fetch qq qrcode");

    print_qr_for_scan(session.qr_code());

    let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_secs(180);
    loop {
        let status = match session.status().await {
            Ok(status) => status,
            Err(MusicClientError::TencentMqttLogin(err)) => {
                println!("transient mqtt status error: {err}");
                LoginStatus::WaitingScan
            }
            Err(err) => panic!("qq login status should return: {err:?}"),
        };

        match status {
            LoginStatus::WaitingScan => println!("waiting scan..."),
            LoginStatus::WaitingConfirm => println!("waiting confirm..."),
            LoginStatus::QrCodeExpired => panic!("qrcode expired"),
            LoginStatus::Success(LoginToken::Tencent(_)) => {
                println!("qq login success: token received (redacted)");
                break;
            }
            LoginStatus::Success(LoginToken::Netease(_)) => panic!("unexpected netease token"),
        }

        assert!(tokio::time::Instant::now() < deadline, "manual qq login timed out");
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
