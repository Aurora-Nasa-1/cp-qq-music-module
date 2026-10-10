# CPPlayer 音源模块（QQ 音乐）

把本库封装成 **CPPlayer 的音源模块（Provider 插件）**，支持 **Android** 与 **桌面端
（Windows / Linux / macOS）**。

- 宿主契约：`CPPlayer-KMP/docs/dev/PROVIDER_DEV_GUIDE.md`
- 宿主实现：`CPPlayer-KMP/core/src/jvmMain/kotlin/cp/player/core/provider/JniProvider.kt`
- 本模块代码：[`src/jni.rs`](../src/jni.rs)

> 本模块只提供**接口封装**，不提供、存储或分发任何音乐内容。请遵守当地法律法规与平台
> 服务条款，尊重版权并支持正版。

---

## 1. 它是什么

一个 `type: "jni"` 的模块包（zip）：宿主用 `System.load()` 加载本库编译出的动态库，
通过 JNI 调用三个导出函数，本模块把宿主的**标准化方法名**翻译成 [`MusicClient`] 调用
（固定 `Platform::Tencent`），再把统一模型转回**宿主期望的 JSON 形状**。

```
宿主 MusicApiService
  └─ ProviderManager.callApi(标准方法名, params)      // 按 manifest.apiMap 映射
       └─ JniProvider.nativeCallApi(method, paramsJson)   // JNI
            └─ src/jni.rs dispatch()：方法名 → MusicClient(QQ 音乐)
                 └─ 统一模型 → 宿主 JSON（code / data / songs / ar / al / lrc …）
```

### 1.1 JNI 导出符号（与宿主类名硬绑定）

JNI 按 `Java_<包名下划线化>_<类名>_<方法名>` 查找符号。宿主类是
`cp.player.core.provider.JniProvider`，因此本库必须导出：

| 导出函数 | 作用 |
|---|---|
| `Java_cp_player_core_provider_JniProvider_startNativeServer` | 初始化（本模块不起独立服务，仅建 tokio 运行时） |
| `Java_cp_player_core_provider_JniProvider_nativeCallApi` | **核心**：分发一次 API 调用，返回 JSON |
| `Java_cp_player_core_provider_JniProvider_analyzeAudioFile` | 音频分析（本模块不支持，返回错误 JSON） |

> ⚠️ 宿主后端包名历史上改过（`cp.player.provider` → `cp.player.kmp.provider` →
> `cp.player.core.provider`）。**宿主再改包名时必须同步改 `src/jni.rs` 里的函数名并重新
> 编译分发**，否则动态库仍能被 `System.load()` 加载成功，但首次方法调用会抛
> `UnsatisfiedLinkError`（症状是「模块显示已加载，一调用就崩」）。

---

## 2. 构建

产物是可直接导入的 zip。

### 2.1 Android（多 ABI）

依赖 `cargo-ndk` 与 Android NDK：

```bash
cargo install cargo-ndk
export ANDROID_NDK_HOME=/path/to/ndk   # 或放在 $ANDROID_HOME/ndk/<ver>

./scripts/build_module_android.sh
# -> target/cp_qq_module_android.zip
```

包内布局（宿主按设备 ABI 自动挑选）：

```
manifest.json
lib/arm64-v8a/libnetease_qq_music_api.so
lib/armeabi-v7a/libnetease_qq_music_api.so
lib/x86_64/libnetease_qq_music_api.so
```

脚本已带 `-C link-arg=-Wl,-z,max-page-size=16384`（Android 15+ 的 16KB 页面要求）。

### 2.2 Windows 桌面

需要 MSVC 工具链（VS 或 Build Tools 的「使用 C++ 的桌面开发」工作负载）+ Windows SDK。
脚本会自行定位工具链并设置 `PATH/INCLUDE/LIB`，**不需要**「Developer Command Prompt」：

```powershell
.\scripts\build_module_desktop.ps1
# -> target\cp_qq_module_windows_x64.zip
```

若自动定位失败，可显式指定：

```powershell
.\scripts\build_module_desktop.ps1 -MsvcRoot "D:\VS\VC\Tools\MSVC" -SdkRoot "C:\Program Files (x86)\Windows Kits\10"
```

### 2.3 Linux / macOS 桌面

```bash
./scripts/build_module_desktop.sh
# -> target/cp_qq_module_linux_x86_64.zip / cp_qq_module_macos_arm64.zip
```

桌面包为**单架构**，布局是 `manifest.json` + 根目录下的动态库（无 `lib/<abi>/`）。

### 2.4 手动构建

```bash
cargo build --release --lib --features jni
```

不加 `--features jni` 时**不会**编译 JNI 入口（普通库用法不受影响）。

---

## 3. 导入与使用

应用内 **设置 → 音源管理 → 导入模块**，选择上面的 zip。导入后在音源列表里切到
「QQ 音乐」，用 **扫码** 登录即可。

---

## 4. 支持的方法

宿主方法名 → 本库调用。未列出的方法请视为不支持（见 §5）。

| 宿主方法 | 本库调用 | 备注 |
|---|---|---|
| `cloudsearch` | `search().song()/artist()/album()/playlist()` | 按 `type` 分派：1 单曲 / 100 歌手 / 10 专辑 / 1000 歌单 |
| `song/detail` | `detail().song().ids()` | |
| `song/url/v1`、`song/url/v1/302`、`song/download/url/v1` | `playback().url()` | 无版权时返回 `url: null`（不算错误） |
| `lyric/new` | `playback().lyric()` | 行级 LRC + 翻译；**无逐字歌词** |
| `playlist/detail` | `playlist().detail()` | 含 `tracks` |
| `playlist/track/all` | `playlist().detail()` | |
| `album` | `detail().album()` | |
| `artist/detail`、`artist/songs`、`artist/top/song` | `detail().artist()` | 三者共用一次调用，返回 `data.artist` + `songs` |
| `toplist`、`toplist_detail` | `discover().toplist_list()` | |
| `personalized` | `discover().recommend_playlist()` | |
| `playlist/catlist` | `playlist().categories()` | |
| `top_playlist`、`playlist/list` | `playlist().list()` | 取 `cat`/`category` 参数 |
| `search/hot/detail` | `discover().hotkey()` | |
| `search/suggest` | `discover().suggests()` | |
| `login/qr/key`、`login/qr/create` | `login().session()` | 创建/复用扫码会话，返回 `data.qrimg` |
| `login/qr/check` | `session.status()` | 映射为 800/801/802/803（803 含 `cookie`） |

> 曲目时长：返回的每首歌都带 `dt`（毫秒）。网易云取上游 `dt`；QQ 音乐取 `interval`
> （秒）× 1000 —— 上游未给该字段时为 `0`（宿主按「未知时长」处理）。

### 4.1 登录态传递

宿主把凭据放在 `params["cookie"]`，形如：

```
uin=123456; qqmusic_key=xxx; qm_keyst=xxx; tmeLoginType=1
```

本模块解析成 `TencentLoginToken` 后注入构建器；解析失败按未登录处理。

---

## 5. 不支持的方法

模块的 `manifest.json` 的 `apiMap` 已把**不支持的方法**声明为 `"unsupported"`
（共 116 项）。宿主会**直接短路**（返回 `code = -1`，记为 WARNING 而非 ERROR），
不会发到本模块 —— 这样诊断页不会被「本来就不支持的功能」拖成红色。

主要原因：本库是**统一只读封装**，只覆盖搜索 / 详情 / 播放 / 发现 / 歌单 / 扫码登录；
下列能力在上游库里就不存在，需要扩展本库才能支持：

- **账号类**：`user/playlist`、`user/detail`、`likelist`、`like`、`recommend/songs`、
  `user/record`、`user/follows` …（⇒ 「我的歌单」「我喜欢的音乐」在 QQ 音源下为空）
- **社交类**：`comment/*`、`msg/*`、`pl/count`、`send/text`
- **MV / 视频 / 电台 / 云盘 / 签到 / 听歌打卡**
- **写操作**：`playlist/create|delete|subscribe`、`album/sub`、`artist/sub` …
- **一起听**：`listentogether/*`
- `login/status`（本库无该端点；登录态由宿主侧 cookie 判断）
- `login`（邮箱）、`login/cellphone`（手机号）—— 本模块只支持**扫码**登录

---

## 6. 已知限制

1. **无逐字歌词**（`yrc`）。本库只返回行级 LRC 与翻译。
2. **`analyzeAudioFile` 未实现**（宿主视为可选能力）。
3. **QQ 音源下账号相关页面为空**（见 §5）。

---

## 7. 调试

本模块**不打日志**（宿主会把每次调用的方法名与响应长度写进「设置 → 调试 → 健康状态」）。
要单独验证模块，可用宿主同款声明写个最小 JVM 测试：

```java
package cp.player.core.provider;

public class JniProvider {
    public native void startNativeServer(String host, int port);
    public native String nativeCallApi(String method, String paramsJson);
    public native String analyzeAudioFile(String path);
}
```

```java
System.load("C:/path/to/netease_qq_music_api.dll");
JniProvider p = new JniProvider();
p.startNativeServer("127.0.0.1", 3000);
System.out.println(p.nativeCallApi("cloudsearch", "{\"keywords\":\"晴天\",\"type\":\"1\"}"));
```

---

## 8. 相关文件

| 文件 | 作用 |
|---|---|
| `src/jni.rs` | JNI 导出 + 方法分发 + JSON 形状转换 |
| `packaging/manifest.json` | 模块清单（Android 版；桌面脚本会改写 `entryPoint` 并去掉 `supportedAbis`） |
| `scripts/build_module_android.sh` | Android 打包 |
| `scripts/build_module_desktop.sh` | Linux / macOS 打包 |
| `scripts/build_module_desktop.ps1` | Windows 打包 |
