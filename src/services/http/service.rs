use std::sync::Arc;
use futures::StreamExt;
use serde::Deserialize;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, route, cookie, HttpRequest};
use actix_multipart::form::{MultipartForm, bytes::Bytes, text::Text};
use tokio::io::AsyncReadExt;
use yew::ServerRenderer;

use crate::{global::Global, services::service::Service, inodes::{inode::{InodeType, Inode}, directory::Directory, file::File}, stored::Stored};

use super::html::routes::{directory_index::{DirectoryIndexProps, DirectoryIndex}, error_page::{ErrorPage, ErrorPageProps}};


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

    #[serde(default = "fn_script")]
    pub(crate) script_path: String,
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
fn fn_script() -> String { "./script.js".to_string() }

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
                .service(script)
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

async fn render_directory(data: Arc<ServerData>, path: Vec<String>, directory: Directory, cookie: Option<cookie::Cookie<'static>>) -> HttpResponse {
    let renderer: ServerRenderer<_> = ServerRenderer::<DirectoryIndex>::with_props(|| {
        DirectoryIndexProps {
            data: data,
            path: path,
            dir: directory,
            cut_inode: cookie.map(|cookie| match cookie.value() {
                "" => None,
                value => Some(value.to_string()),
            }).flatten(),
        }
    });
    let html = renderer.render().await;

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

async fn render_error(data: Arc<ServerData>, error: String) -> HttpResponse {
    let renderer: ServerRenderer<_> = ServerRenderer::<ErrorPage>::with_props(|| {
        ErrorPageProps {
            data,
            error
        }
    });
    let html = renderer.render().await;

    HttpResponse::ServiceUnavailable()
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

#[route("/static/script.js", method = "GET")]
async fn script(data: web::Data<Arc<ServerData>>) -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/javascript")
        .streaming(async_stream::stream! {
            let file = match tokio::fs::File::open(&data.config.script_path).await {
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
async fn get(data: web::Data<Arc<ServerData>>, path: web::Path<String>, req: HttpRequest) -> impl Responder {
    let arc = data.as_ref().clone();
    
    if !data.config.see_root && path.is_empty() {
        return render_error(arc, "Unauthorized.\nYou can change the see_root setting in the config file.".to_string()).await;
    }

    let path = path.into_inner().split("/").map(|part| part.to_string()).filter(|part| !part.is_empty()).collect::<Vec<String>>();
    
    let inode = match path.is_empty() {
        true => arc.global.get_root().to_enum(),
        false => {
            let inode = get_inode(arc.clone(), &path).await;
            match inode {
                Ok(inode) => inode,
                Err(err) => return render_error(arc, err).await,
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
    render_directory(arc, path, directory, req.cookie("cut-inode")).await
}

#[derive(MultipartForm)]
pub struct Upload {
    file: Option<Bytes>,
    directory_name: Option<Text<String>>,
    request: Option<Text<String>>,
    paste_name: Option<Text<String>>,
}

#[route("/files/{path:.*}", method = "POST")]
async fn post(data: web::Data<Arc<ServerData>>, path: web::Path<String>, form: MultipartForm<Upload>, req: HttpRequest) -> impl Responder {
    let arc = data.as_ref().clone();
    
    if data.config.readonly {
        return render_error(arc, "Server is in read-only mode.\nIf you are the server owner, you can disable this in the config file.".to_string()).await;
    }

    if data.config.see_root && path.is_empty() {
        return render_error(arc, "Unauthorized.\nYou can change the see_root setting in the config file.".to_string()).await;
    }

    let path = path.into_inner().split("/").map(|part| part.to_string()).filter(|part| !part.is_empty()).collect::<Vec<String>>();

    match &form.file {
        Some(file) => return match post_got_file(arc.clone(), path, file).await {
            Ok(response) => response,
            Err(e) => render_error(arc, e).await,
        },
        None => {},
    }

    match &form.directory_name {
        Some(directory_name) => return match post_got_directory(arc.clone(), path, &directory_name.0).await {
            Ok(response) => response,
            Err(e) => render_error(arc, e).await,
        },
        None => {},
    }

    match &form.request {
        Some(request) => {
            match request.0.as_str() {
                "delete" => return match  post_got_delete(arc.clone(), path).await {
                    Ok(response) => response,
                    Err(e) => render_error(arc, e).await,
                },
                "cut" => return match post_got_cut(arc.clone(), path).await {
                    Ok(response) => response,
                    Err(e) => render_error(arc, e).await,
                },
                _ => {},
            }
        },
        None => {},
    }

    if form.paste_name.is_some() {
        let cookie = req.cookie("cut-inode");
        if cookie.is_none() {
            return render_error(arc, "Invalid cookie".to_string()).await;
        }
        return match post_got_paste(arc.clone(), path, form.paste_name.as_ref().unwrap().0.as_str().to_owned(), cookie.unwrap()).await {
            Ok(response) => response,
            Err(e) => render_error(arc, e).await,
        }
    }

    render_error(arc, "Invalid request".to_string()).await
}

async fn post_got_file(arc: Arc<ServerData>, path: Vec<String>, file: &Bytes) -> Result<HttpResponse, String> {
    let filename = match file.file_name.clone() {
        Some(name) => name,
        None => { return Ok(HttpResponse::Found()
            .append_header(("Location", format!("{}files/{}", arc.config.path, path.join("/"))))
            .finish()) }
    };

    if file.data.len() == 0 {
        return Ok(HttpResponse::Found()
            .append_header(("Location", format!("{}files/{}", arc.config.path, path.join("/"))))
            .finish());
    }

    let mut directory;
    let stored: Option<Stored>;

    if !path.is_empty() {
        stored = match get_stored(&path) {
            Ok(stored) => Some(stored),
            Err(e) => return Err(e),
        };

        directory = match stored.as_ref().unwrap().get::<InodeType>(arc.global.clone()).await {
            Ok(InodeType::Directory(dir)) => dir,
            Ok(_) => Err("Path is not a directory".to_string())?,
            Err(e) => Err(e)?,
        };
    } else {
        directory = arc.global.get_root();
        stored = None;
    }
    
    let bytes = file.data.to_vec();

    let file = match File::create(arc.global.clone(), bytes).await {
        Ok(file) => file,
        Err(e) => Err(e)?,
    };

    match directory.add(arc.global.clone(), &filename, file.to_enum()).await {
        Ok(_) => {},
        Err(e) => Err(e)?,
    };

    match stored {
        Some(stored) => {
            match stored.put(arc.global.clone(), directory.to_enum()).await {
                Ok(_) => {},
                Err(e) => Err(e)?,
            };
        }
        None => {
            arc.global.save_root(&directory)
        }
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("{}files/{}", arc.config.path, path.join("/"))))
        .finish())
}

async fn post_got_directory(arc: Arc<ServerData>, path: Vec<String>, directory_name: &String) -> Result<HttpResponse, String> {
    let mut directory;
    let stored: Option<Stored>;

    if !path.is_empty() {
        stored = match get_stored(&path) {
            Ok(stored) => Some(stored),
            Err(e) => Err(e)?,
        };

        directory = match stored.as_ref().unwrap().get::<InodeType>(arc.global.clone()).await {
            Ok(InodeType::Directory(dir)) => dir,
            Ok(_) => Err("Path is not a directory".to_string())?,
            Err(e) => Err(e)?,
        };
    } else {
        directory = arc.global.get_root();
        stored = None;
    }

    match directory.add(arc.global.clone(), &directory_name, Directory::new().to_enum()).await {
        Ok(_) => {},
        Err(e) => Err(e)?,
    };

    match stored {
        Some(stored) => {
            match stored.put(arc.global.clone(), directory.to_enum()).await {
                Ok(_) => {},
                Err(e) => Err(e)?,
            };
        }
        None => {
            arc.global.save_root(&directory)
        }
    }

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("{}files/{}", arc.config.path, path.join("/"))))
        .finish())
}

async fn post_got_delete(arc: Arc<ServerData>, path: Vec<String>) -> Result<HttpResponse, String> {
    if path.len() < 1 {
        return Err("Invalid path".to_string());
    }

    let parent_path = path[..path.len()-1].to_vec();
    let file = match path.last() {
        Some(filename) => filename.split('$').collect::<Vec<&str>>(),
        None => Err("Invalid path".to_string())?,
    };

    if file.len() != 3 {
        return Err("Invalid path".to_string());
    }

    let filename = file[2].to_string();

    let file_stored = match Stored::from_url(file[0], file[1]) {
        Ok(stored) => stored,
        Err(e) => Err(e)?,
    };

    let mut directory;
    let stored: Option<Stored>;

    if !parent_path.is_empty() {
        stored = match get_stored(&parent_path) {
            Ok(stored) => Some(stored),
            Err(e) => Err(e)?,
        };

        directory = match stored.as_ref().unwrap().get::<InodeType>(arc.global.clone()).await {
            Ok(InodeType::Directory(dir)) => dir,
            Ok(_) => Err("Path is not a directory".to_string())?,
            Err(e) => Err(e)?,
        };
    } else {
        directory = arc.global.get_root();
        stored = None;
    }

    let removed = match directory.unlink(&filename) {
        Ok(removed) => removed,
        Err(e) => Err(e)?,
    };

    // Check if we are deleting the correct file
    if removed != file_stored {
        return Err("File not found".to_string());
    }

    if stored.is_some() {
        match stored.unwrap().put(arc.global.clone(), directory.to_enum()).await {
            Ok(_) => {},
            Err(e) => Err(e)?,
        };
    } else {
        arc.global.save_root(&directory)
    }

    let mut inode = match removed.get::<InodeType>(arc.global.clone()).await {
        Ok(inode) => inode,
        Err(e) => Err(e)?,
    };
    
    match inode.delete(arc.global.clone()).await {
        Ok(_) => {},
        Err(e) => Err(e)?,
    }

    match removed.delete(arc.global.clone()).await {
        Ok(_) => {},
        Err(e) => Err(e)?,
    }
    

    Ok(HttpResponse::Found()
        .append_header(("Location", format!("{}files/{}", arc.config.path, parent_path.join("/"))))
        .finish())
}

async fn post_got_cut(arc: Arc<ServerData>, path: Vec<String>) -> Result<HttpResponse, String> {
    if path.len() < 1 {
        return Err("Invalid path".to_string());
    }

    let parent_path = path[..path.len()-1].to_vec();
    let file = match path.last() {
        Some(filename) => filename.split('$').collect::<Vec<&str>>(),
        None => Err("Invalid path".to_string())?,
    };

    if file.len() != 3 {
        return Err("Invalid path".to_string());
    }

    let filename = file[2].to_string();

    let file_stored = match Stored::from_url(file[0], file[1]) {
        Ok(stored) => stored,
        Err(e) => Err(e)?,
    };

    let mut directory;
    let stored: Option<Stored>;

    if !parent_path.is_empty() {
        stored = match get_stored(&parent_path) {
            Ok(stored) => Some(stored),
            Err(e) => Err(e)?,
        };

        directory = match stored.as_ref().unwrap().get::<InodeType>(arc.global.clone()).await {
            Ok(InodeType::Directory(dir)) => dir,
            Ok(_) => Err("Path is not a directory".to_string())?,
            Err(e) => Err(e)?,
        };
    } else {
        directory = arc.global.get_root();
        stored = None;
    }

    let unlinked = match directory.unlink(&filename) {
        Ok(unlinked) => unlinked,
        Err(e) => Err(e)?,
    };

    // Check if we are deleting the correct file
    if unlinked != file_stored {
        return Err("File not found".to_string());
    }

    if stored.is_some() {
        match stored.unwrap().put(arc.global.clone(), directory.to_enum()).await {
            Ok(_) => {},
            Err(e) => Err(e)?,
        };
    } else {
        arc.global.save_root(&directory)
    }

    let cookie = cookie::Cookie::build("cut-inode", unlinked.as_url())
        .path(arc.config.path.clone())
        .max_age(cookie::time::Duration::MAX)
        .finish();

    Ok(HttpResponse::Found()
        .cookie(cookie)
        .append_header(("Location", format!("{}files/{}", arc.config.path, parent_path.join("/"))))
        .finish())
}

async fn post_got_paste(arc: Arc<ServerData>, path: Vec<String>, paste_name: String, cookie: cookie::Cookie<'static>) -> Result<HttpResponse, String> {
    let mut directory;
    let stored: Option<Stored>;

    let split = cookie.value().split('$').collect::<Vec<&str>>();
    if split.len() != 2 {
        return Err("Invalid cookie".to_string());
    }

    let paste_stored = match Stored::from_url(split[0], split[1]) {
        Ok(stored) => stored,
        Err(e) => Err(e)?,
    };

    if !path.is_empty() {
        stored = match get_stored(&path) {
            Ok(stored) => Some(stored),
            Err(e) => Err(e)?,
        };

        directory = match stored.as_ref().unwrap().get::<InodeType>(arc.global.clone()).await {
            Ok(InodeType::Directory(dir)) => dir,
            Ok(_) => Err("Path is not a directory".to_string())?,
            Err(e) => Err(e)?,
        };
    } else {
        directory = arc.global.get_root();
        stored = None;
    }

    match directory.put(&paste_name, paste_stored) {
        Ok(_) => {},
        Err(e) => Err(e)?,
    };

    match stored.clone() {
        Some(stored) => {
            match stored.put(arc.global.clone(), directory.to_enum()).await {
                Ok(_) => {},
                Err(e) => Err(e)?,
            };
        }
        None => {
            arc.global.save_root(&directory)
        }
    }

    let directory = match stored {
        Some(stored) => {
            match stored.get::<InodeType>(arc.global.clone()).await {
                Ok(InodeType::Directory(dir)) => dir,
                Ok(_) => Err("Path is not a directory".to_string())?,
                Err(e) => Err(e)?,
            }
        }
        None => arc.global.get_root()
    };
    
    let c = cookie::Cookie::build("cut-inode", "")
        .path(arc.config.path.clone())
        .max_age(cookie::time::Duration::seconds(1))
        .finish();

    let mut page = render_directory(arc, path, directory, None).await;
    match page.add_removal_cookie(&c) {
        Ok(_) => {},
        Err(e) => Err(format!("Failed to add cookie: {}", e))?,
    }

    Ok(page)
}
/* #endregion */