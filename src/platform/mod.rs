pub(crate) mod netease;
mod tencent;

pub(crate) use netease::NeteaseClient;
pub(crate) use tencent::{TencentClient, TencentMqttLoginSession};

fn collect_into<T, U>(values: Vec<T>) -> Vec<U>
where
    T: Into<U>,
{
    values.into_iter().map(Into::into).collect()
}
