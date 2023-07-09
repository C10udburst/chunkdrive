use std::sync::Arc;
use futures::StreamExt;
use serde::Deserialize;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, route};
use yew::ServerRenderer;

use crate::{global::{Global}, services::{service::Service}, inodes::{inode::InodeType, directory::Directory}, stored::Stored};

use super::html::routes::directory_index::{DirectoryIndexProps, DirectoryIndex};


#[derive(Debug, Deserialize, Clone)]
pub struct HttpService {
    port: u16,
    #[serde(default = "default_address")]
    address: String,

    #[serde(default = "default_path")]
    path: String,

    #[serde(default = "fn_false")]
    readonly: bool,

    #[serde(default = "fn_true")]
    see_root: bool,

    #[serde(default = "fn_true")]
    admin: bool
}

#[derive(Debug)]
pub struct ServerData {
    global: Arc<Global>,
    config: HttpService,
}

fn default_address() -> String { "127.0.0.1".to_string() }
fn default_path() -> String { "/".to_string() }
const fn fn_false() -> bool { false }
const fn fn_true() -> bool { true }

impl Service for HttpService {
    fn run(&self, global: Arc<Global>) {
        let data = Arc::new(ServerData { global, config: self.clone() });
        std::thread::spawn(move || {
            match run_blocking(data) {
                Ok(_) => {},
                Err(e) => println!("Failed to run HTTP service: {}", e),
            }
        });
    }
}

fn run_blocking(data: Arc<ServerData>) -> Result<(), String> {
    println!("Starting HTTP service on {}:{}", data.config.address, data.config.port);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let data_clone = data.clone();
    rt.block_on(async {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(data_clone.clone()))
                .service(redirect)
                .service(get)
                .service(post)
        })
        .bind(format!("{}:{}", data.config.address, data.config.port))
        .map_err(|e| format!("Failed to bind to port: {}", e))?
        .run()
        .await
        .map_err(|e| format!("Failed to run server: {}", e))
    })?;

    Ok(())
}

async fn get_inode(data: Arc<ServerData>, path: String) -> Result<InodeType, String> {
    let entry = path.split('/').last().ok_or("Invalid path")?;
    let parts = entry.split('$').collect::<Vec<&str>>();
    let (bucket, descriptor) = match parts.len() {
        2 => (parts[0].to_string(), parts[1].to_string()),
        3 => (parts[0].to_string(), parts[1].to_string()),
        _ => return Err("Invalid path".to_string())
    };
    let stored = Stored::from_url(&bucket, &descriptor)?;
    let inode = stored.get::<InodeType>(data.global.clone()).await?;

    Ok(inode)
}

/* #region Routes */

#[route("/", method = "GET")]
async fn redirect(data: web::Data<Arc<ServerData>>) -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", format!("{}files/", data.config.path)))
        .finish()
}

async fn render_directory(data: Arc<ServerData>, path: String, directory: Directory) -> HttpResponse {
    let path = path.split('/').map(|s| s.to_string()).collect::<Vec<String>>();

    let renderer: ServerRenderer<_> = ServerRenderer::<DirectoryIndex>::with_props(|| {
        DirectoryIndexProps {
            data: data,
            path: path,
            dir: directory
        }
    });
    let html = renderer.render().await;

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

#[route("/files/{path:.*}", method = "GET")]
async fn get(data: web::Data<Arc<ServerData>>, path: web::Path<String>) -> impl Responder {
    if !data.config.see_root && path.is_empty() {
        return HttpResponse::Unauthorized()
            .content_type("text/plain")
            .body("Unauthorized");
    }

    let arc = data.as_ref().clone();
    let path = path.into_inner();
    
    let inode = match path.is_empty() {
        true => arc.global.get_root().to_enum(),
        false => {
            let inode = get_inode(arc.clone(), path.clone()).await;
            match inode {
                Ok(inode) => inode,
                Err(err) => return HttpResponse::ServiceUnavailable()
                    .content_type("text/plain")
                    .body(err),
            }
        }
    };

    let directory = match inode {
        InodeType::Directory(dir) => dir,
        InodeType::File(file) => { // if the path is a file, stream it
            return HttpResponse::Ok()
                .content_type("application/octet-stream")
                .streaming(async_stream::stream! {
                    let mut stream = file.get(arc.global.clone());

                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(chunk) => yield Ok(web::Bytes::from(chunk)),
                            Err(e) => yield Err(std::io::Error::new(std::io::ErrorKind::Other, e))
                        }
                    }
                });
        }
    };

    render_directory(arc, path, directory).await
}

#[route("/files/{path:.*}", method = "POST")]
async fn post(data: web::Data<Arc<ServerData>>, path: web::Path<String>) -> impl Responder {
    if data.config.readonly {
        return HttpResponse::Unauthorized()
            .content_type("text/plain")
            .body("Unauthorized");
    }

    
    HttpResponse::Ok()
        .body("")
}

/* #endregion */