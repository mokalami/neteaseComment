use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use qr2term::print_qr;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine;
use chrono::{TimeZone, Local};

const API_BASE_URL: &str = "https://netease-delta-ten.vercel.app";

#[derive(Debug, Serialize, Deserialize, Clone)]
struct LoginResponse {
    code: i32,
    #[serde(default)]
    cookie: String,
    #[serde(default)]
    token: String,
    account: Option<Account>,
    profile: Option<Profile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Account {
    id: i64,
    #[serde(rename = "userName")]
    user_name: String,
    #[serde(rename = "type")]
    account_type: i32,
    status: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserProfile {
    code: i32,
    profile: Profile,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Profile {
    nickname: String,
    #[serde(rename = "userId")]
    user_id: i64,
    #[serde(rename = "avatarUrl")]
    avatar_url: String,
    #[serde(rename = "backgroundUrl")]
    background_url: Option<String>,
    signature: Option<String>,
    #[serde(rename = "createTime")]
    create_time: i64,
    #[serde(rename = "userName")]
    #[serde(default)]
    user_name: String,
    #[serde(rename = "accountType")]
    #[serde(default)]
    account_type: i32,
    #[serde(rename = "vipType")]
    #[serde(default)]
    vip_type: i32,
    #[serde(default)]
    followed: bool,
    #[serde(default)]
    follows: i32,
    #[serde(default)]
    followeds: i32,
    #[serde(rename = "eventCount")]
    #[serde(default)]
    event_count: i32,
    #[serde(rename = "playlistCount")]
    #[serde(default)]
    playlist_count: i32,
    #[serde(rename = "playlistBeSubscribedCount")]
    #[serde(default)]
    playlist_be_subscribed_count: i32,
    #[serde(default)]
    province: i32,
    #[serde(default)]
    city: i32,
    #[serde(default)]
    birthday: i64,
    #[serde(default)]
    gender: i32,
    description: Option<String>,
    #[serde(rename = "detailDescription")]
    detail_description: Option<String>,
    #[serde(rename = "defaultAvatar")]
    #[serde(default)]
    default_avatar: bool,
    #[serde(rename = "expertTags")]
    expert_tags: Option<Vec<String>>,
    experts: Option<serde_json::Value>,
    #[serde(rename = "djStatus")]
    #[serde(default)]
    dj_status: i32,
    #[serde(default)]
    mutual: bool,
    #[serde(rename = "remarkName")]
    remark_name: Option<String>,
    #[serde(rename = "authStatus")]
    #[serde(default)]
    auth_status: i32,
    #[serde(default)]
    blacklist: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserRecord {
    code: i32,
    #[serde(rename = "allData")]
    all_data: Vec<SongData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SongData {
    score: i64,
    song: Song,
}

#[derive(Debug, Serialize, Deserialize)]
struct Song {
    name: String,
    id: i64,
    // 添加其他需要的字段
}

#[derive(Debug, Serialize, Deserialize)]
struct PlaylistResponse {
    code: i32,
    playlist: Vec<Playlist>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Playlist {
    id: i64,
    name: String,
    #[serde(rename = "trackCount")]
    track_count: i32,
    #[serde(rename = "playCount")]
    play_count: i64,
    creator: Creator,
    description: Option<String>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Creator {
    nickname: String,
    #[serde(rename = "userId")]
    user_id: i64,
    #[serde(rename = "avatarUrl")]
    avatar_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Comment {
    commentId: i64,
    user: CommentUser,
    content: String,
    time: i64,
    likedCount: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CommentUser {
    userId: i64,
    nickname: String,
    avatarUrl: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserComment {
    song_id: i64,
    song_name: String,
    comment_id: i64,
    content: String,
    time: i64,
    liked_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct FollowsResponse {
    code: i32,
    #[serde(default)]
    #[serde(rename = "followeds")]
    follow: Vec<Follow>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Follow {
    nickname: String,
    userId: i64,
    avatarUrl: String,
    signature: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommentResponse {
    code: i32,
    comments: Vec<Comment>,
    total: i32,
}

// 添加新的结构体用于二维码登录
#[derive(Debug, Serialize, Deserialize)]
struct QrKeyResponse {
    code: i32,
    #[serde(rename = "data")]
    key_data: QrKeyData,
}

#[derive(Debug, Serialize, Deserialize)]
struct QrKeyData {
    #[serde(rename = "unikey")]
    key: String,
    qrimg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QrCreateResponse {
    code: i32,
    #[serde(rename = "data")]
    data: QrCreateData,
}

#[derive(Debug, Serialize, Deserialize)]
struct QrCreateData {
    qrurl: String,
    qrimg: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QrCheckResponse {
    code: i32,
    message: String,
    cookie: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CommentOutput {
    用户: UserInfo,
    被回复: Vec<String>,
    挂件数据: Option<serde_json::Value>,
    显示楼层评论: Option<serde_json::Value>,
    状态: i32,
    评论ID: i64,
    内容: String,
    富文本内容: Option<serde_json::Value>,
    内容资源: Option<serde_json::Value>,
    时间: i64,
    时间字符串: String,
    需要显示时间: bool,
    点赞数: i32,
    表情链接: Option<serde_json::Value>,
    评论位置类型: i32,
    父评论ID: i64,
    装饰: serde_json::Map<String, serde_json::Value>,
    回复标记: Option<serde_json::Value>,
    等级: Option<serde_json::Value>,
    用户业务等级: Option<serde_json::Value>,
    IP位置: IpLocation,
}

#[derive(Debug, Serialize, Deserialize)]
struct UserInfo {
    地理位置: Option<serde_json::Value>,
    直播信息: Option<serde_json::Value>,
    是否匿名: i32,
    头像详情: Option<serde_json::Value>,
    用户类型: i32,
    头像链接: String,
    是否关注: bool,
    是否互相关注: bool,
    备注名: Option<String>,
    社交用户ID: Option<serde_json::Value>,
    会员权益: VipInfo,
    昵称: String,
    认证状态: i32,
    专家标签: Option<serde_json::Value>,
    专家: Option<serde_json::Value>,
    会员类型: i32,
    通用身份: Option<serde_json::Value>,
    用户ID: i64,
}

#[derive(Debug, Serialize, Deserialize)]
struct VipInfo {
    associator: Option<serde_json::Value>,
    musicPackage: Option<serde_json::Value>,
    redplus: Option<serde_json::Value>,
    redVipAnnualCount: i32,
    redVipLevel: i32,
    relationType: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct IpLocation {
    IP: Option<String>,
    地理位置: String,
    用户ID: Option<serde_json::Value>,
}

struct NeteaseMusicClient {
    client: reqwest::Client,
    cookie: Option<String>,
}

impl NeteaseMusicClient {
    fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cookie: None,
        }
    }

    async fn login(&mut self, phone: &str, password: &str) -> Result<()> {
        let url = format!("{}/login/cellphone", API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .query(&[("phone", phone), ("password", password)])
            .send()
            .await?;

        let cookies: Vec<String> = response
            .headers()
            .get_all("set-cookie")
            .iter()
            .filter_map(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .collect();

        let response_data = response.json::<LoginResponse>().await?;

        if response_data.code == 200 {
            let cookie = cookies.join("; ");
            self.cookie = Some(cookie.clone());
            
            let response_with_cookie = LoginResponse {
                cookie,
                ..response_data
            };
            
            fs::write("login_info.json", serde_json::to_string(&response_with_cookie)?)?;
            println!("登录成功！");
            Ok(())
        } else {
            Err(anyhow::anyhow!("登录失败：状态码 {}", response_data.code))
        }
    }

    async fn get_user_profile(&self, uid: i64) -> Result<UserProfile> {
        let url = format!("{}/user/detail", API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .query(&[("uid", uid.to_string())])
            .header("Cookie", self.cookie.as_ref().unwrap())
            .send()
            .await?
            .json::<UserProfile>()
            .await?;

        Ok(response)
    }

    async fn get_user_record(&self, uid: i64) -> Result<UserRecord> {
        let url = format!("{}/user/record", API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .query(&[("uid", uid.to_string()), ("type", "0".to_string())])
            .header("Cookie", self.cookie.as_ref().unwrap())
            .send()
            .await?
            .json::<UserRecord>()
            .await?;

        Ok(response)
    }

    // 获取用户歌单
    async fn get_user_playlists(&self, uid: i64, limit: Option<i32>, offset: Option<i32>) -> Result<PlaylistResponse> {
        let url = format!("{}/user/playlist", API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("uid", uid.to_string()),
                ("limit", limit.unwrap_or(30).to_string()),
                ("offset", offset.unwrap_or(0).to_string()),
            ])
            .header("Cookie", self.cookie.as_ref().unwrap())
            .send()
            .await?
            .json::<PlaylistResponse>()
            .await?;

        Ok(response)
    }

    // 获取用户关注列表
    async fn get_user_follows(&self, uid: i64, limit: Option<i32>, offset: Option<i32>) -> Result<FollowsResponse> {
        let url = format!("{}/user/follows", API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("uid", uid.to_string()),
                ("limit", limit.unwrap_or(30).to_string()),
                ("offset", offset.unwrap_or(0).to_string()),
            ])
            .header("Cookie", self.cookie.as_ref().unwrap())
            .send()
            .await?
            .json::<FollowsResponse>()
            .await?;

        Ok(response)
    }

    // 获取用户粉丝列表
    async fn get_user_followeds(&self, uid: i64, limit: Option<i32>, offset: Option<i32>) -> Result<FollowsResponse> {
        let url = format!("{}/user/followeds", API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("uid", uid.to_string()),
                ("limit", limit.unwrap_or(30).to_string()),
                ("offset", offset.unwrap_or(0).to_string()),
            ])
            .header("Cookie", self.cookie.as_ref().unwrap())
            .send()
            .await?
            .json::<FollowsResponse>()
            .await?;

        Ok(response)
    }

    // 关注/取消关注用户
    async fn follow_user(&self, uid: i64, follow: bool) -> Result<serde_json::Value> {
        let url = format!("{}/follow", API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("id", uid.to_string()),
                ("t", if follow { "1" } else { "0" }.to_string()),
            ])
            .header("Cookie", self.cookie.as_ref().unwrap())
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response)
    }

    // 获取歌曲评论
    async fn get_song_comments(&self, song_id: i64, limit: i32, offset: i32) -> Result<CommentResponse> {
        let url = format!("{}/comment/music", API_BASE_URL);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("id", song_id.to_string()),
                ("limit", limit.to_string()),
                ("offset", offset.to_string()),
            ])
            .header("Cookie", self.cookie.as_ref().unwrap())
            .send()
            .await?
            .json::<CommentResponse>()
            .await?;

        Ok(response)
    }

    // 并发获取用户在歌曲下的评论
    async fn get_user_comments_for_songs(&self, songs: &[SongData], target_uid: i64) -> Result<()> {
        use futures::stream::{self, StreamExt};
        use tokio::time::{sleep, Duration};

        // 创建 comments 目录用于保存评论文件
        fs::create_dir_all("comments")?;

        // 设置并发数为 50
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(50));

        // 创建进度条
        let m = MultiProgress::new();
        let total_progress = std::sync::Arc::new(m.add(ProgressBar::new(songs.len() as u64)));
        total_progress.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} 首歌曲 ({percent}%)")
                .unwrap_or_else(|_| ProgressStyle::default_bar())
                .progress_chars("##-")
        );

        // 创建歌曲处理流
        let song_stream = stream::iter(songs.iter().enumerate()).map({
            let total_progress = total_progress.clone();
            move |(song_index, song)| {
                let client = self.clone();
                let semaphore = semaphore.clone();
                let total_progress = total_progress.clone();
                let song_progress = m.add(ProgressBar::new(100));
                
                song_progress.set_style(
                    ProgressStyle::default_bar()
                        .template("[{elapsed_precise}] {prefix:.green} {bar:40.yellow/red} {pos}/{len} 页评论")
                        .unwrap_or_else(|_| ProgressStyle::default_bar())
                        .progress_chars("##-")
                );
                
                song_progress.set_prefix(format!("歌曲 {}/{}", song_index + 1, songs.len()));

                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    let mut song_comments = Vec::new();
                    
                    // 每页获取 100 条评论，最多获取 100 页
                    for offset in (0..10000).step_by(100) {
                        sleep(Duration::from_millis(50)).await;
                        
                        match client.get_song_comments(song.song.id, 100, offset).await {
                            Ok(response) => {
                                let comments = response.comments.clone();
                                let user_comments: Vec<CommentOutput> = response.comments
                                    .into_iter()
                                    .filter(|comment| comment.user.userId == target_uid)
                                    .map(|comment| {
                                        let time = Local.timestamp_millis_opt(comment.time).unwrap();
                                        CommentOutput {
                                            用户: UserInfo {
                                                地理位置: None,
                                                直播信息: None,
                                                是否匿名: 0,
                                                头像详情: None,
                                                用户类型: 0,
                                                头像链接: comment.user.avatarUrl,
                                                是否关注: false,
                                                是否互相关注: false,
                                                备注名: None,
                                                社交用户ID: None,
                                                会员权益: VipInfo {
                                                    associator: None,
                                                    musicPackage: None,
                                                    redplus: None,
                                                    redVipAnnualCount: -1,
                                                    redVipLevel: 0,
                                                    relationType: 0,
                                                },
                                                昵称: comment.user.nickname,
                                                认证状态: 0,
                                                专家标签: None,
                                                专家: None,
                                                会员类型: 0,
                                                通用身份: None,
                                                用户ID: comment.user.userId,
                                            },
                                            被回复: Vec::new(),
                                            挂件数据: None,
                                            显示楼层评论: None,
                                            状态: 0,
                                            评论ID: comment.commentId,
                                            内容: comment.content,
                                            富文本内容: None,
                                            内容资源: None,
                                            时间: comment.time,
                                            时间字符串: time.format("%Y-%m-%d").to_string(),
                                            需要显示时间: true,
                                            点赞数: comment.likedCount,
                                            表情链接: None,
                                            评论位置类型: 0,
                                            父评论ID: 0,
                                            装饰: serde_json::Map::new(),
                                            回复标记: None,
                                            等级: None,
                                            用户业务等级: None,
                                            IP位置: IpLocation {
                                                IP: None,
                                                地理位置: String::new(),
                                                用户ID: None,
                                            },
                                        }
                                    })
                                    .collect();
                                
                                song_comments.extend(user_comments);

                                // 每 5 页更新一次进度条
                                if offset % 500 == 0 {
                                    song_progress.inc(5);
                                }

                                // 如果返回的评论数小于请求数，说明已到达末尾
                                if comments.len() < 100 {
                                    break;
                                }
                            }
                            Err(e) => {
                                eprintln!("获取歌曲 {} 的评论失败: {}", song.song.id, e);
                                break;
                            }
                        }
                    }

                    // 保存当前歌曲的评论到单独的文件
                    if !song_comments.is_empty() {
                        if let Ok(json_str) = serde_json::to_string_pretty(&song_comments) {
                            let file_path = format!("comments/song_{}.json", song.song.id);
                            if let Err(e) = fs::write(&file_path, json_str) {
                                eprintln!("保存评论文件失败: {}", e);
                            }
                        }
                    }

                    song_progress.finish_with_message(format!("歌曲 {} 完成", song.song.name));
                    total_progress.inc(1);
                    
                    Ok::<_, anyhow::Error>(song_comments)
                }
            }
        });

        // 并发处理所有歌曲，最多 50 个并发
        let mut buffered = song_stream.buffer_unordered(50);
        
        while let Some(result) = buffered.next().await {
            if let Ok(_comments) = result {
                // 评论已经保存到文件，这里不需要额外处理
            }
        }

        total_progress.finish_with_message("所有歌曲评论获取完成！");

        Ok(())
    }

    // 获取二维码 key
    async fn get_qr_key(&self) -> Result<String> {
        let url = format!("{}/login/qr/key", API_BASE_URL);
        let timestamp = chrono::Local::now().timestamp_millis().to_string();
        let response = self
            .client
            .get(&url)
            .query(&[("timestamp", &timestamp)])
            .send()
            .await?
            .json::<QrKeyResponse>()
            .await?;

        if response.code == 200 {
            Ok(response.key_data.key)
        } else {
            Err(anyhow::anyhow!("获取二维码key失败"))
        }
    }

    // 生成二维码
    async fn create_qr(&self, key: &str) -> Result<String> {
        let url = format!("{}/login/qr/create", API_BASE_URL);
        let timestamp = chrono::Local::now().timestamp_millis().to_string();
        let response = self
            .client
            .get(&url)
            .query(&[
                ("key", key),
                ("qrimg", "true"),
                ("timestamp", &timestamp)
            ])
            .send()
            .await?
            .json::<QrCreateResponse>()
            .await?;

        if response.code == 200 {
            if let Some(qr_img) = response.data.qrimg {
                // 使用新的 base64 解码方法
                let qr_data = BASE64_STANDARD.decode(
                    qr_img.split(',').nth(1).unwrap_or("")
                )?;
                
                // 克隆数据用于文件写入
                fs::write("qr_code.png", &qr_data)?;
                println!("\n二维码已保存到 qr_code.png");
                
                // 直接打印 URL 的二维码
                print_qr(response.data.qrurl.as_bytes())?;
                
                Ok(response.data.qrurl)
            } else {
                Ok(response.data.qrurl)
            }
        } else {
            Err(anyhow::anyhow!("生成二维码失败"))
        }
    }

    // 检查二维码状态
    async fn check_qr(&self, key: &str) -> Result<QrCheckResponse> {
        let url = format!("{}/login/qr/check", API_BASE_URL);
        let timestamp = chrono::Local::now().timestamp_millis().to_string();
        let response = self
            .client
            .get(&url)
            .query(&[
                ("key", key),
                ("timestamp", &timestamp)
            ])
            .send()
            .await?
            .json::<QrCheckResponse>()
            .await?;

        Ok(response)
    }

    // 二维码登录流程
    async fn login_by_qr(&mut self) -> Result<()> {
        println!("开始二维码登录流程...");
        
        // 获取二维码 key
        let key = self.get_qr_key().await?;
        
        // 生成二维码
        let qr_img = self.create_qr(&key).await?;
        println!("\n请使用网易云音乐 App 扫描二维码：\n{}", qr_img);
        
        // 循环检查扫码状态
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            let check_resp = self.check_qr(&key).await?;
            
            match check_resp.code {
                800 => {
                    println!("二维码已过期，请重新运行程序");
                    return Err(anyhow::anyhow!("二维码已过期"));
                }
                801 => {
                    println!("等待扫码中...");
                }
                802 => {
                    println!("扫码成功，请在手机上确认登录");
                }
                803 => {
                    println!("登录成功！");
                    if let Some(cookie) = check_resp.cookie {
                        self.cookie = Some(cookie.clone());
                        // 保存登录信息
                        let login_info = LoginResponse {
                            code: 200,
                            cookie,
                            token: String::new(),
                            account: None,
                            profile: None,
                        };
                        fs::write("login_info.json", serde_json::to_string(&login_info)?)?;
                        return Ok(());
                    }
                }
                _ => {
                    println!("未知状态：{}", check_resp.message);
                }
            }
        }
    }
}

// 为 NeteaseMusicClient 实现 Clone
impl Clone for NeteaseMusicClient {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            cookie: self.cookie.clone(),
        }
    }
}

async fn get_user_input(prompt: &str) -> Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = NeteaseMusicClient::new();

    // 检查是否存在保存的登录信息
    if !Path::new("login_info.json").exists() {
        println!("请使用二维码登录网易云音乐");
        client.login_by_qr().await?;
    } else {
        let login_info: LoginResponse = serde_json::from_str(&fs::read_to_string("login_info.json")?)?;
        client.cookie = Some(login_info.cookie);
        println!("使用已保存的登录信息");
    }

    let uid = get_user_input("请输入要查询的用户 UID: ").await?
        .parse::<i64>()
        .context("无效的 UID")?;

    // 获取用户详情
    let profile = client.get_user_profile(uid).await?;
    println!("\n用户详情:");
    println!("昵称: {}", profile.profile.nickname);
    println!("用户ID: {}", profile.profile.user_id);
    println!("签名: {}", profile.profile.signature.unwrap_or_default());
    println!("关注数: {}", profile.profile.follows);
    println!("粉丝数: {}", profile.profile.followeds);
    println!("动态数: {}", profile.profile.event_count);
    println!("歌单数: {}", profile.profile.playlist_count);

    // 获取用户歌单
    let playlists = client.get_user_playlists(uid, Some(10), None).await?;
    println!("\n用户歌单:");
    for (index, playlist) in playlists.playlist.iter().enumerate() {
        println!(
            "{}. {} (ID: {}) - 播放次数: {}",
            index + 1,
            playlist.name,
            playlist.id,
            playlist.play_count
        );
    }

    // 获取听歌榜单
    let record = client.get_user_record(uid).await?;
    println!("\n听歌榜单:");
    for (index, song_data) in record.all_data.iter().enumerate() {
        println!(
            "{}. {} (ID: {}) - 播放次数: {}",
            index + 1,
            song_data.song.name,
            song_data.song.id,
            song_data.score
        );
    }

    // 获取用户关注列表
    let follows = client.get_user_follows(uid, Some(5), None).await?;
    println!("\n关注表(前5个):");
    for (index, follow) in follows.follow.iter().enumerate() {
        println!(
            "{}. {} (ID: {})",
            index + 1,
            follow.nickname,
            follow.userId
        );
    }

    // 获取用户粉丝列表
    let followeds = client.get_user_followeds(uid, Some(5), None).await?;
    println!("\n粉丝列表(前5个):");
    for (index, followed) in followeds.follow.iter().enumerate() {
        println!(
            "{}. {} (ID: {}) {}",
            index + 1,
            followed.nickname,
            followed.userId,
            followed.signature.as_deref().unwrap_or("")
        );
    }

    // 获取用户在这些歌曲下的评论
    println!("\n开始获取用户在这些歌曲下的评论...");
    client.get_user_comments_for_songs(&record.all_data, uid).await?;

    Ok(())
}
