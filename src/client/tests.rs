use super::*;
use crate::error::MusicClientError;
use crate::models::{Platform, SongQuality};

mod section_validation {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn search_requires_keyword() {
        let client = MusicClient::new();
        let err = client.search().send().await.expect_err("missing keyword should fail");
        assert!(matches!(err, MusicClientError::MissingKeyword));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn search_rejects_empty_keyword() {
        let client = MusicClient::new();
        let err =
            client.search().keyword("   ").send().await.expect_err("empty keyword should fail");
        assert!(matches!(err, MusicClientError::MissingKeyword));
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
        let err =
            client.detail().artist().id("   ").send().await.expect_err("empty id should fail");
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
        let err = client
            .discover()
            .playlist_list()
            .send()
            .await
            .expect_err("missing category should fail");
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
        let err =
            client.login().refresh().send().await.expect_err("missing refresh token should fail");
        assert!(matches!(err, MusicClientError::MissingRefreshToken));
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
        let err = client
            .playlist()
            .list()
            .category("")
            .send()
            .await
            .expect_err("empty category should fail");
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
}

mod section_live_smoke {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn search_returns_correct_response_netease() {
        let client = MusicClient::new();
        let response = client
            .search()
            .keyword("江南")
            .platform(Platform::Netease)
            .send()
            .await
            .expect("search should succeed");
        assert_eq!(response.songs[0].id, "108914");
        assert_eq!(response.songs[0].album.id, "10804");
        assert_eq!(response.songs[0].album.name, "第二天堂");
        assert_eq!(response.songs[0].artists[0].name, "林俊杰");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn search_returns_correct_response_tencent() {
        let client = MusicClient::new();
        let response = client
            .search()
            .keyword("江南")
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("search should succeed");
        assert_eq!(response.songs[0].id, "004TXEXY2G2c7C");
        assert_eq!(response.songs[0].album.id, "000y5gq7449K9I");
        assert_eq!(response.songs[0].album.name, "第二天堂");
        assert_eq!(response.songs[0].artists[0].name, "林俊杰");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn search_artist_returns_correct_response_netease() {
        let client = MusicClient::new();
        let response = client
            .search()
            .artist()
            .keyword("林俊杰")
            .platform(Platform::Netease)
            .send()
            .await
            .expect("search should succeed");
        assert_eq!(response.artists[0].id, "3684");
        assert_eq!(response.artists[0].name, "林俊杰");
        assert!(!response.artists[0].pic_url.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn search_artist_returns_correct_response_tencent() {
        let client = MusicClient::new();
        let response = client
            .search()
            .artist()
            .keyword("林俊杰")
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("search should succeed");
        assert_eq!(response.artists[0].id, "001BLpXF2DyJe2");
        assert_eq!(response.artists[0].name, "林俊杰");
        assert!(!response.artists[0].pic_url.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn search_album_returns_correct_response_netease() {
        let client = MusicClient::new();
        let response = client
            .search()
            .album()
            .keyword("江南")
            .platform(Platform::Netease)
            .send()
            .await
            .expect("search should succeed");
        assert_eq!(response.albums[0].id, "10804");
        assert_eq!(response.albums[0].name, "第二天堂");
        assert!(!response.albums[0].pic_url.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn search_album_returns_correct_response_tencent() {
        let client = MusicClient::new();
        let response = client
            .search()
            .album()
            .keyword("江南")
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("search should succeed");
        assert_eq!(response.albums[0].id, "000y5gq7449K9I");
        assert_eq!(response.albums[0].name, "第二天堂");
        assert!(!response.albums[0].pic_url.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn search_playlist_returns_correct_response_netease() {
        let client = MusicClient::new();
        let response = client
            .search()
            .playlist()
            .keyword("江南")
            .platform(Platform::Netease)
            .send()
            .await
            .expect("search should succeed");
        assert_eq!(response.playlists[0].id, "8814710412");
        assert_eq!(response.playlists[0].name, "烟雨江南 | 你一句春不晚 我到了真江南");
        assert!(!response.playlists[0].pic_url.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn search_playlist_returns_correct_response_tencent() {
        let client = MusicClient::new();
        let response = client
            .search()
            .playlist()
            .keyword("江南")
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("search should succeed");
        assert_eq!(response.playlists[0].id, "1790504159");
        assert_eq!(response.playlists[0].name, "青春记忆 | 90后校园岁月的流行歌曲");
        assert!(response.playlists[0].pic_url.starts_with("http://qpic.y.qq.com/music_cover/"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_returns_correct_response_artist_netease() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .artist()
            .id("3684")
            .platform(Platform::Netease)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.songs[0].id, "108485");
        assert_eq!(response.songs[0].album.id, "10770");
        assert_eq!(response.songs[0].album.name, "JJ陆");
        assert_eq!(response.name, "林俊杰");
        assert!(response.description.starts_with("JJ林俊杰的创作来自最深的情感"))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_returns_correct_response_artist_tencent() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .artist()
            .id("001BLpXF2DyJe2")
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.songs[0].id, "001N8e5Q4Gjxda");
        assert_eq!(response.songs[0].album.id, "002g6zv02X7SNi");
        assert_eq!(response.songs[0].album.name, "JJ陆");
        assert_eq!(response.name, "林俊杰");
        assert!(response.description.starts_with("JJ林俊杰的创作来自最深的情感"))
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_returns_correct_response_album_netease() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .album()
            .id("10770")
            .platform(Platform::Netease)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.songs[0].id, "108458");
        assert_eq!(response.songs[0].name, "SIXOLOGY");
        assert_eq!(response.name, "JJ陆");
        assert!(response.description.starts_with("JJ林俊杰 2008最新创作大碟 JJ陆"));
        assert!(!response.pic_url.is_empty())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_returns_correct_response_album_tencent() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .album()
            .id("002g6zv02X7SNi")
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.songs[0].id, "002vb5JD04eGw3");
        assert_eq!(response.songs[0].name, "Sixology");
        assert_eq!(response.name, "JJ陆");
        assert!(response.description.starts_with("JJ林俊杰 2008最新创作大碟 JJ陆"));
        assert!(!response.pic_url.is_empty())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_returns_correct_response_playlist_netease() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .playlist()
            .id(8814710412)
            .platform(Platform::Netease)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.songs[0].id, "1392942840");
        assert_eq!(response.songs[0].name, "烟雨行舟 (新版)");
        assert_eq!(response.name, "烟雨江南 | 你一句春不晚 我到了真江南");
        assert!(response.description.starts_with("恬静而淡雅"));
        assert!(!response.pic_url.is_empty())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_returns_correct_response_playlist_tencent() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .playlist()
            .id(1790504159)
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.songs[0].id, "004TXEXY2G2c7C");
        assert_eq!(response.songs[0].name, "江南");
        assert_eq!(response.name, "青春记忆 | 90后校园岁月的流行歌曲");
        assert!(response.description.starts_with("教室里的不再是我们了"));
        assert!(!response.pic_url.is_empty())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_returns_correct_response_toplist_netease() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .toplist()
            .id(19723756)
            .platform(Platform::Netease)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.name, "飙升榜");
        assert_eq!(response.description, "云音乐中每天热度上升最快的100首单曲，每日更新。");
        assert!(!response.pic_url.is_empty())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_returns_correct_response_toplist_tencent() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .toplist()
            .id(62)
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.name, "飙升榜");
        assert!(response.description.starts_with("1. 榜单定义：QQ音乐站内播"));
        assert!(!response.pic_url.is_empty())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn playback_returns_correct_response_netease_url() {
        let client = MusicClient::new();
        let response = client
            .playback()
            .url()
            .id("108914")
            .level(SongQuality::Master)
            .platform(Platform::Netease)
            .send()
            .await
            .expect("playback should succeed");
        assert!(response.url.starts_with("http://m802.music.126.net/"));
        assert_eq!(response.level, SongQuality::Standard);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn playback_returns_correct_response_tencent_url() {
        let client = MusicClient::new();
        let response = client
            .playback()
            .url()
            .id("004Gq0xE1YC8xp")
            .level(SongQuality::Master)
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("playback should succeed");
        assert!(response.url.starts_with("https://isure.stream.qqmusic.qq.com/"));
        assert_eq!(response.level, SongQuality::Standard);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn playback_returns_correct_response_netease_lyric() {
        let client = MusicClient::new();
        let response = client
            .playback()
            .lyric()
            .id("108914")
            .platform(Platform::Netease)
            .send()
            .await
            .expect("playback should succeed");
        assert!(response.lyric.starts_with("[00:37.010] 风到这里就是粘"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn playback_returns_correct_response_tencent_lyric() {
        let client = MusicClient::new();
        let response = client
            .playback()
            .lyric()
            .id("004Gq0xE1YC8xp")
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("playback should succeed");
        assert!(response.lyric.starts_with("[00:00.270]素颜 - 许嵩/何曼婷"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_returns_correct_response_netease_hotkey() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .hotkey()
            .platform(Platform::Netease)
            .send()
            .await
            .expect("discover should succeed");
        assert_eq!(response.hotkey.len(), 20);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_returns_correct_response_tencent_hotkey() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .hotkey()
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("discover should succeed");
        assert_eq!(response.hotkey.len(), 20);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_returns_correct_response_netease_recommend() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .recommend_playlist()
            .platform(Platform::Netease)
            .send()
            .await
            .expect("discover should succeed");
        assert_eq!(response.playlists.len(), 6);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_returns_correct_response_tencent_recommend() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .recommend_playlist()
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("discover should succeed");
        assert_eq!(response.playlists.len(), 6);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_returns_correct_response_netease_toplist() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .toplist_list()
            .platform(Platform::Netease)
            .send()
            .await
            .expect("discover should succeed");
        assert_eq!(response.toplists[0].name, "飙升榜");
        assert_eq!(response.toplists[7].name, "网易云中文说唱榜");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_returns_correct_response_tencent_toplist() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .toplist_list()
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("discover should succeed");
        assert_eq!(response.toplists[0].name, "飙升榜");
        assert_eq!(response.toplists[1].name, "热歌榜");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_returns_correct_response_netease_categories() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .playlist_categories()
            .platform(Platform::Netease)
            .send()
            .await
            .expect("discover should succeed");
        assert_eq!(response.categories[0].name, "全部歌单");
        assert_eq!(response.categories[1].name, "综艺");
        assert_eq!(response.categories[0].id, "全部歌单");
        assert_eq!(response.categories[1].id, "综艺");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_returns_correct_response_tencent_categories() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .playlist_categories()
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("discover should succeed");
        assert_eq!(response.categories[0].name, "官方歌单");
        assert_eq!(response.categories[1].name, "私藏");
        assert_eq!(response.categories[0].id, "3317");
        assert_eq!(response.categories[1].id, "3417");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_returns_correct_response_netease_playlist() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .playlist_list()
            .category("全部歌单")
            .platform(Platform::Netease)
            .send()
            .await
            .expect("discover should succeed");
        assert!(response.more);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn discover_returns_correct_response_tencent_playlist() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .playlist_list()
            .category("3317")
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("discover should succeed");
        assert!(response.more);
    }
}
