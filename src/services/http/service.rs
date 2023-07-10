use std::sync::Arc;
use futures::StreamExt;
use serde::Deserialize;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, route};
use actix_multipart::{form::{MultipartForm, bytes::Bytes, text::Text}};
use tokio::io::AsyncReadExt;
use yew::ServerRenderer;

use crate::{global::{Global}, services::{service::Service}, inodes::{inode::InodeType, directory::Directory, file::File}, stored::Stored};

use super::html::routes::directory_index::{DirectoryIndexProps, DirectoryIndex};


#[derive(Debug, Deserialize, Clone)]
pub struct HttpService {
    pub(crate) port: u16,
    #[serde(default = "default_address")]
    pub(crate) address: String,

    #[serde(default = "default_path")]
    pub(crate) path: String,

    #[serde(default = "fn_false")]
    pub(crate) readonly: bool,

    #[serde(default = "fn_true")]
    pub(crate) see_root: bool,

    #[serde(default = "fn_true")]
    pub(crate) admin: bool,

    #[serde(default = "fn_style")]
    pub(crate) style_path: String,
}

#[derive(Debug)]
pub struct ServerData {
    pub global: Arc<Global>,
    pub config: HttpService,
}

fn default_address() -> String { "127.0.0.1".to_string() }
fn default_path() -> String { "/".to_string() }
const fn fn_false() -> bool { false }
const fn fn_true() -> bool { true }
fn fn_style() -> String { "./style.css".to_string() }

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
    println!("Starting HTTP service on http://{}:{}", data.config.address, data.config.port);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let data_clone = data.clone();
    rt.block_on(async {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(data_clone.clone()))
                .service(style)
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

fn get_stored(path: &Vec<String>) -> Result<Stored, String> {
    let entry = path.last().ok_or("Invalid path")?;
    let parts = entry.split('$').collect::<Vec<&str>>();
    let (bucket, descriptor) = match parts.len() {
        2 => (parts[0].to_string(), parts[1].to_string()),
        3 => (parts[0].to_string(), parts[1].to_string()),
        _ => return Err("Invalid path".to_string())
    };
    Stored::from_url(&bucket, &descriptor)
}

async fn get_inode(data: Arc<ServerData>, path: &Vec<String>) -> Result<InodeType, String> {
    let stored = get_stored(path)?;

    let inode = stored.get::<InodeType>(data.global.clone()).await?;

    Ok(inode)
}

async fn render_directory(data: Arc<ServerData>, path: Vec<String>, directory: Directory) -> HttpResponse {
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

/* #region Routes */

#[route("/static/style.css", method = "GET")]
async fn style(data: web::Data<Arc<ServerData>>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/css")
        .streaming(async_stream::stream! {
            let file = match tokio::fs::File::open(&data.config.style_path).await {
                Ok(file) => file,
                Err(e) => {
                    yield Err(e);
                    return;
                }
            };
            let mut buffered_reader = tokio::io::BufReader::new(file);
            let mut buffer = vec![0; 2048];
            loop {
                let bytes_read = match buffered_reader.read(&mut buffer).await {
                    Ok(bytes_read) => bytes_read,
                    Err(e) => {
                        yield Err(e);
                        return;
                    }
                };
                if bytes_read == 0 {
                    return;
                }
                yield Ok(web::Bytes::from(buffer[..bytes_read].to_vec()));
            }
        })
}

#[route("/", method = "GET")]
async fn redirect(data: web::Data<Arc<ServerData>>) -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", format!("{}files/", data.config.path)))
        .finish()
}

#[route("/files/{path:.*}", method = "GET")]
async fn get(data: web::Data<Arc<ServerData>>, path: web::Path<String>) -> impl Responder {
    if !data.config.see_root && path.is_empty() {
        return HttpResponse::Unauthorized()
            .content_type("text/plain")
            .body("Unauthorized");
    }

    let arc = data.as_ref().clone();
    let path = path.into_inner().split("/").map(|part| part.to_string()).filter(|part| !part.is_empty()).collect::<Vec<String>>();
    
    let inode = match path.is_empty() {
        true => arc.global.get_root().to_enum(),
        false => {
            let inode = get_inode(arc.clone(), &path).await;
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
        InodeType::File(file) => {
            // if the path is a file, stream it
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

    // otherwise, render an html index of the directory
    render_directory(arc, path, directory).await
}

#[derive(MultipartForm)]
pub struct Upload {
    file: Option<Bytes>,
    directory_name: Option<Text<String>>,
}

#[route("/files/{path:.*}", method = "POST")]
async fn post(data: web::Data<Arc<ServerData>>, path: web::Path<String>, form: MultipartForm<Upload>) -> impl Responder {
    if data.config.readonly {
        return HttpResponse::Unauthorized()
            .content_type("text/plain")
            .body("Unauthorized");
    }

    if data.config.see_root && path.is_empty() {
        return HttpResponse::Unauthorized()
            .content_type("text/plain")
            .body("Unauthorized");
    }

    let arc = data.as_ref().clone();
    let path = path.into_inner().split("/").map(|part| part.to_string()).filter(|part| !part.is_empty()).collect::<Vec<String>>();

    match &form.file {
        Some(file) => return post_got_file(arc, path, file).await,
        None => {},
    }

    match &form.directory_name {
        Some(directory_name) => return post_got_directory(arc, path, &directory_name.0).await,
        None => {},
    }

    HttpResponse::BadRequest()
        .content_type("text/plain")
        .body("Invalid POST request")
}

async fn post_got_file(arc: Arc<ServerData>, path: Vec<String>, file: &Bytes) -> HttpResponse {
    let filename = match file.file_name.clone() {
        Some(name) => name,
        None => return HttpResponse::BadRequest()
            .content_type("text/plain")
            .body("No filename provided"),
    };

    let mut directory;
    let stored: Option<Stored>;

    if !path.is_empty() {
        stored = match get_stored(&path) {
            Ok(stored) => Some(stored),
            Err(e) => return HttpResponse::BadRequest()
                .content_type("text/plain")
                .body(e),
        };

        directory = match stored.as_ref().unwrap().get::<InodeType>(arc.global.clone()).await {
            Ok(InodeType::Directory(dir)) => dir,
            Ok(_) => return HttpResponse::BadRequest()
                .content_type("text/plain")
                .body("Path is not a directory"),
            Err(e) => return HttpResponse::ServiceUnavailable()
                .content_type("text/plain")
                .body(e),
        };
    } else {
        directory = arc.global.get_root();
        stored = None;
    }
    
    let bytes = file.data.to_vec();

    let file = match File::create(arc.global.clone(), bytes).await {
        Ok(file) => file,
        Err(e) => return HttpResponse::ServiceUnavailable()
            .content_type("text/plain")
            .body(e),
    };

    match directory.add(arc.global.clone(), &filename, file.to_enum()).await {
        Ok(_) => {},
        Err(e) => return HttpResponse::ServiceUnavailable()
            .content_type("text/plain")
            .body(e),
    };

    match stored {
        Some(stored) => {
            match stored.put(arc.global.clone(), directory.to_enum()).await {
                Ok(_) => {},
                Err(e) => return HttpResponse::ServiceUnavailable()
                    .content_type("text/plain")
                    .body(e),
            };
        }
        None => {
            arc.global.save_root(&directory)
        }
    }

    HttpResponse::Found()
        .append_header(("Location", format!("{}files/{}", arc.config.path, path.join("/"))))
        .finish()
}

async fn post_got_directory(arc: Arc<ServerData>, path: Vec<String>, directory_name: &String) -> HttpResponse {
    let mut directory;
    let stored: Option<Stored>;

    if !path.is_empty() {
        stored = match get_stored(&path) {
            Ok(stored) => Some(stored),
            Err(e) => return HttpResponse::BadRequest()
                .content_type("text/plain")
                .body(e),
        };

        directory = match stored.as_ref().unwrap().get::<InodeType>(arc.global.clone()).await {
            Ok(InodeType::Directory(dir)) => dir,
            Ok(_) => return HttpResponse::BadRequest()
                .content_type("text/plain")
                .body("Path is not a directory"),
            Err(e) => return HttpResponse::ServiceUnavailable()
                .content_type("text/plain")
                .body(e),
        };
    } else {
        directory = arc.global.get_root();
        stored = None;
    }

    match directory.add(arc.global.clone(), &directory_name, Directory::new().to_enum()).await {
        Ok(_) => {},
        Err(e) => return HttpResponse::ServiceUnavailable()
            .content_type("text/plain")
            .body(e),
    };

    match stored {
        Some(stored) => {
            match stored.put(arc.global.clone(), directory.to_enum()).await {
                Ok(_) => {},
                Err(e) => return HttpResponse::ServiceUnavailable()
                    .content_type("text/plain")
                    .body(e),
            };
        }
        None => {
            arc.global.save_root(&directory)
        }
    }

    HttpResponse::Found()
        .append_header(("Location", format!("{}files/{}", arc.config.path, path.join("/"))))
        .finish()
}

/* #endregion */