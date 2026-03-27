use crate::error::{MusicClientError, MusicClientResult};
use crate::models::{NeteaseLoginToken, Platform, TencentLoginToken};

/// 音乐资源 ID 输入抽象。
///
/// 用于让请求构建器在单个 `id(...)` 方法中同时支持 `u64` 与字符串输入。
pub trait MusicIdInput {
    /// 转为请求内部统一使用的字符串 ID。
    fn into_id_string(self) -> String;
}

impl MusicIdInput for u64 {
    fn into_id_string(self) -> String {
        self.to_string()
    }
}

impl MusicIdInput for String {
    fn into_id_string(self) -> String {
        self
    }
}

impl MusicIdInput for &str {
    fn into_id_string(self) -> String {
        self.to_owned()
    }
}

impl MusicIdInput for &String {
    fn into_id_string(self) -> String {
        self.clone()
    }
}

/// 登录 token 的借用视图。
///
/// 用于在请求构建器中以统一形式接收不同平台的 token 引用，
/// 并在发送请求前执行平台一致性校验。
#[derive(Clone, Copy, Debug)]
pub enum LoginTokenRef<'a> {
    /// 网易云登录 token 引用。
    Netease(&'a NeteaseLoginToken),
    /// QQ 音乐登录 token 引用。
    Tencent(&'a TencentLoginToken),
}

impl<'a> From<&'a NeteaseLoginToken> for LoginTokenRef<'a> {
    fn from(token: &'a NeteaseLoginToken) -> Self {
        Self::Netease(token)
    }
}

impl<'a> From<&'a TencentLoginToken> for LoginTokenRef<'a> {
    fn from(token: &'a TencentLoginToken) -> Self {
        Self::Tencent(token)
    }
}

impl<'a> LoginTokenRef<'a> {
    pub(crate) fn as_netease(self) -> Option<&'a NeteaseLoginToken> {
        match self {
            Self::Netease(token) => Some(token),
            Self::Tencent(_) => None,
        }
    }

    pub(crate) fn as_tencent(self) -> Option<&'a TencentLoginToken> {
        match self {
            Self::Netease(_) => None,
            Self::Tencent(token) => Some(token),
        }
    }
}

pub(crate) fn netease_token<'a>(auth: Option<LoginTokenRef<'a>>) -> Option<&'a NeteaseLoginToken> {
    auth.and_then(LoginTokenRef::as_netease)
}

pub(crate) fn tencent_token<'a>(auth: Option<LoginTokenRef<'a>>) -> Option<&'a TencentLoginToken> {
    auth.and_then(LoginTokenRef::as_tencent)
}

pub(crate) fn validate_auth_platform(
    platform: Platform,
    auth: Option<LoginTokenRef<'_>>,
) -> MusicClientResult<()> {
    match (platform, auth) {
        (Platform::Netease, Some(LoginTokenRef::Tencent(_))) => {
            Err(MusicClientError::AuthTokenPlatformMismatch {
                expected_platform: "netease",
                token_platform: "tencent",
            })
        }
        (Platform::Tencent, Some(LoginTokenRef::Netease(_))) => {
            Err(MusicClientError::AuthTokenPlatformMismatch {
                expected_platform: "tencent",
                token_platform: "netease",
            })
        }
        _ => Ok(()),
    }
}

pub(crate) fn require_keyword(keyword: Option<&str>) -> MusicClientResult<&str> {
    match keyword {
        Some(keyword) if !keyword.trim().is_empty() => Ok(keyword),
        _ => Err(MusicClientError::MissingKeyword),
    }
}

pub(crate) fn require_id(id: Option<&str>) -> MusicClientResult<&str> {
    match id {
        Some(id) if !id.trim().is_empty() => Ok(id),
        _ => Err(MusicClientError::MissingId),
    }
}

pub(crate) fn require_digit_str(id: Option<&str>) -> MusicClientResult<u64> {
    match id {
        Some(id) => {
            let digit_str = id.trim();
            if digit_str.is_empty() || !digit_str.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(MusicClientError::MissingId);
            }
            digit_str.parse::<u64>().map_err(|_| MusicClientError::MissingId)
        }
        _ => Err(MusicClientError::MissingId),
    }
}

pub(crate) fn require_category(category: Option<&str>) -> MusicClientResult<&str> {
    match category {
        Some(category) if !category.trim().is_empty() => Ok(category),
        _ => Err(MusicClientError::MissingCategory),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_digit_str_accepts_trimmed_digits() {
        let id = require_digit_str(Some("  12345  ")).expect("digits should be accepted");
        assert_eq!(id, 12345);
    }

    #[test]
    fn require_digit_str_rejects_non_digit_input() {
        let err = require_digit_str(Some("12ab")).expect_err("non-digit string should be rejected");
        assert!(matches!(err, MusicClientError::MissingId));
    }
}
