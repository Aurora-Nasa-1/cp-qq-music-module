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
    async fn detail_playlist_rejects_non_digit_id() {
        let client = MusicClient::new();
        let err = client
            .detail()
            .playlist()
            .id("playlist-id")
            .send()
            .await
            .expect_err("non-digit id should fail");
        assert!(matches!(err, MusicClientError::MissingId));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_toplist_rejects_non_digit_id() {
        let client = MusicClient::new();
        let err = client
            .detail()
            .toplist()
            .id("toplist-id")
            .send()
            .await
            .expect_err("non-digit id should fail");
        assert!(matches!(err, MusicClientError::MissingId));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_song_requires_id() {
        let client = MusicClient::new();
        let err = client.detail().song().send().await.expect_err("missing id should fail");
        assert!(matches!(err, MusicClientError::MissingId));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn detail_song_rejects_empty_ids() {
        let client = MusicClient::new();
        let err = client
            .detail()
            .song()
            .ids(vec!["".to_string(), "   ".to_string()])
            .send()
            .await
            .expect_err("empty ids should fail");
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
    async fn search_suggests_requires_keyword() {
        let client = MusicClient::new();
        let err =
            client.discover().suggests().send().await.expect_err("missing keyword should fail");
        assert!(matches!(err, MusicClientError::MissingKeyword));
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
    async fn playlist_detail_rejects_non_digit_id() {
        let client = MusicClient::new();
        let err = client
            .playlist()
            .detail()
            .id("abc")
            .send()
            .await
            .expect_err("non-digit id should fail");
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
    async fn netease_search_song_returns_correct_response() {
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
    async fn tencent_search_song_returns_correct_response() {
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
    async fn netease_search_artist_returns_correct_response() {
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
    async fn tencent_search_artist_returns_correct_response() {
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
    async fn netease_search_album_returns_correct_response() {
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
    async fn tencent_search_album_returns_correct_response() {
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
    async fn netease_search_playlist_returns_correct_response() {
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
    async fn tencent_search_playlist_returns_correct_response() {
        let client = MusicClient::new();
        let response = client
            .search()
            .playlist()
            .keyword("华语歌曲粤语版：除了“江南”还有这些歌")
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("search should succeed");
        assert_eq!(response.playlists[0].id, "2665887142");
        assert_eq!(response.playlists[0].name, "华语歌曲粤语版：除了“江南”还有这些歌");
        assert!(response.playlists[0].pic_url.starts_with("http://qpic.y.qq.com/music_cover/"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn netease_detail_songs_returns_correct_response() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .song()
            .id("1392942840")
            .ids(vec!["1859245776", "1862710424"])
            .platform(Platform::Netease)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.songs[0].id, "1392942840");
        assert_eq!(response.songs[0].album.id, "81878288");
        assert_eq!(response.songs[0].album.name, "烟雨行舟");
        assert_eq!(response.songs[0].artists[0].id, "28863695");
        assert_eq!(response.songs[1].id, "1859245776");
        assert_eq!(response.songs[1].album.id, "130016223");
        assert_eq!(response.songs[1].album.name, "STAY");
        assert_eq!(response.songs[1].artists[0].id, "32795025");
        assert_eq!(response.songs[2].id, "1862710424");
        assert_eq!(response.songs[2].album.id, "130631571");
        assert_eq!(response.songs[2].artists[0].id, "32795025");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tencent_detail_songs_returns_correct_response() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .song()
            .id("001N8e5Q4Gjxda")
            .ids(vec!["004Gq0xE1YC8xp", "00264YJC1bmylC"])
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.songs[0].id, "001N8e5Q4Gjxda");
        assert_eq!(response.songs[0].album.id, "002g6zv02X7SNi");
        assert_eq!(response.songs[0].album.name, "JJ陆");
        assert_eq!(response.songs[0].artists[0].id, "001BLpXF2DyJe2");
        assert_eq!(response.songs[1].id, "004Gq0xE1YC8xp");
        assert_eq!(response.songs[1].album.id, "0035f8nw11cjkf");
        assert_eq!(response.songs[1].album.name, "素颜");
        assert_eq!(response.songs[1].artists[0].id, "000CK5xN3yZDJt");
        assert_eq!(response.songs[2].id, "00264YJC1bmylC");
        assert_eq!(response.songs[2].album.id, "000aM5Ia10a5d8");
        assert_eq!(response.songs[2].album.name, "2017安徽卫视上星20周年");
        assert_eq!(response.songs[2].artists[0].id, "000CK5xN3yZDJt");
    }

    #[tokio::test(flavor = "current_thread")]
    async fn netease_detail_artist_returns_correct_response() {
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
    async fn tencent_detail_artist_returns_correct_response() {
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
    async fn netease_detail_album_returns_correct_response() {
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
    async fn tencent_detail_album_returns_correct_response() {
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
    async fn netease_detail_playlist_returns_correct_response() {
        let client = MusicClient::new();
        let response = client
            .detail()
            .playlist()
            .id(2098357962)
            .platform(Platform::Netease)
            .send()
            .await
            .expect("detail should succeed");
        assert_eq!(response.songs[0].id, "25642214");
        assert_eq!(response.songs[0].name, "爱错(Live)");
        assert_eq!(response.name, "失败总是贯穿人生");
        assert!(response.description.starts_with("相逢如骤雨初晴"));
        assert!(!response.pic_url.is_empty())
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tencent_detail_playlist_returns_correct_response() {
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
    async fn netease_detail_toplist_returns_correct_response() {
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
    async fn tencent_detail_toplist_returns_correct_response() {
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
    async fn netease_playback_url_returns_correct_response() {
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
        let parsed = reqwest::Url::parse(&response.url).expect("url should be valid");
        let host = parsed.host_str().expect("host should exist");
        assert!(host.ends_with(".music.126.net"), "unexpected host: {host}");
        assert_eq!(response.level, SongQuality::Standard);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tencent_playback_url_returns_correct_response() {
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
    async fn netease_playback_lyric_returns_correct_response() {
        let client = MusicClient::new();
        let response = client
            .playback()
            .lyric()
            .id("3362722900")
            .platform(Platform::Netease)
            .send()
            .await
            .expect("playback should succeed");
        assert!(response.lyric.starts_with("[00:05.000]纯音乐，请欣赏"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tencent_playback_lyric_returns_correct_response() {
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
    async fn netease_discover_hotkey_returns_correct_response() {
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
    async fn tencent_discover_hotkey_returns_correct_response() {
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
    async fn netease_discover_recommend_returns_correct_response() {
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
    async fn tencent_discover_recommend_returns_correct_response() {
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
    async fn netease_discover_toplist_returns_correct_response() {
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
    async fn tencent_discover_toplist_returns_correct_response() {
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
    async fn netease_discover_categories_returns_correct_response() {
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
    async fn tencent_discover_categories_returns_correct_response() {
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
    async fn netease_discover_playlist_returns_correct_response() {
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
    async fn tencent_discover_playlist_returns_correct_response() {
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

    #[tokio::test(flavor = "current_thread")]
    async fn netease_discover_suggests_returns_correct_response() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .suggests()
            .keyword("周杰伦")
            .platform(Platform::Netease)
            .send()
            .await
            .expect("discover should succeed");
        assert!(response.suggests[0].starts_with("周杰伦"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn tencent_discover_suggests_returns_correct_response() {
        let client = MusicClient::new();
        let response = client
            .discover()
            .suggests()
            .keyword("周杰伦")
            .platform(Platform::Tencent)
            .send()
            .await
            .expect("discover should succeed");
        assert!(response.suggests[0].starts_with("周杰伦"));
    }
}
