mod storage;
mod user;
mod utils;

use crate::storage::detail::{create_rooted_storage_backend, User};
use crate::user::repo::user;
use crate::utils::db::init_db;
use libunftp::auth::{AuthenticationError, Authenticator, Credentials};
use reqwest::header::HeaderMap;
use salvo::{http::header::CONTENT_TYPE, prelude::*};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

/// FTP服务发送的json请求体
#[derive(Deserialize, Serialize)]
struct UserInfo<'a> {
    account: &'a str,
    password: &'a str,
}

/// 回复给FTP的响应
#[derive(Deserialize, Serialize, Default)]
struct Resp {
    is_exist: bool,
}

/// 接受FTP传来的账号和密码, 查找账号是否存在
#[handler]
async fn find_user(req: &mut Request, res: &mut Response) {
    let client = req.parse_json::<UserInfo>().await.unwrap();

    let is_exist = user(client.account, client.password).await;

    req.headers_mut()
        .insert(CONTENT_TYPE, "application/json".parse().unwrap())
        .unwrap();

    res.render(Json(Resp { is_exist }))
}

#[derive(Debug)]
pub struct RestAuthenticator;

/// 自定义权限验证机制
#[async_trait]
impl Authenticator<User> for RestAuthenticator {
    async fn authenticate(
        &self,
        username: &str,
        creds: &Credentials,
    ) -> Result<User, AuthenticationError> {
        // 只允许账号+密码登录
        let Some(password) = &creds.password else {
            return Err(AuthenticationError::BadUser);
        };

        // 判断是否是管理员
        if username == "root" && creds.password.as_ref().unwrap() == "540hs0qos" {
            return Ok(User {
                username: "root".to_string(),
                // 管理员可以访问根路径, 相当于可以访问所有用户
                root: Some(PathBuf::from("/")),
            });
        }

        // 查找用户是否存在
        let mut json_body = HashMap::new();
        json_body.insert("account", username);
        json_body.insert("password", password);

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/json".parse().unwrap());

        // 像HTTP服务发送账号密码, 判断账号是否有效
        let client = reqwest::Client::new();
        let resp = client
            .post("http://localhost:5800/auth")
            .json(&json_body)
            .headers(headers)
            .send()
            .await
            .unwrap()
            .json::<Resp>()
            .await
            .unwrap();

        if !resp.is_exist {
            return Err(AuthenticationError::BadUser);
        }

        let username = username.to_string();

        // 这里定义每个用户的路径, 相当于/storage/ftp/用户名
        // 如果没有文件夹, 必须要创建这个文件夹
        let path: PathBuf = [r"/", &username].iter().collect();

        Ok(User {
            username,
            root: Some(path),
        })
    }
}

#[tokio::main]
pub async fn main() {
    // 初始化数据库实例
    init_db().await;

    /*
    HTTP服务
     */

    // 移入到线程, 不阻塞当前进程
    std::thread::spawn(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let router = Router::with_path("auth").post(find_user);
                let acceptor = TcpListener::new("0.0.0.0:5800").bind().await;
                tokio::spawn(Server::new(acceptor).serve(router))
                    .await
                    .unwrap();
            });
    });

    /*
    FTP服务
     */
    let server = libunftp::Server::with_authenticator(
        create_rooted_storage_backend(),
        Arc::new(RestAuthenticator {}),
    )
    .passive_ports(50000..65535);

    server.listen("0.0.0.0:2121").await.unwrap();
}
