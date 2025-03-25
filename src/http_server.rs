use std::
    io::Cursor
;

use tiny_http::{Header, Method, Request, Response, Server, StatusCode};
use rust_embed::Embed;
use log::{info, error};

use crate::keyboard_conf;

#[derive(Embed)]
#[folder = "src/http_server_assets/"]
struct Assets;

pub const SERVER_ADDR: &str = "127.0.0.1:19456";

pub fn create_http_server() -> anyhow::Result<()> {
    let result = Server::http(SERVER_ADDR);

    match result {
        Ok(server) => {
            std::thread::spawn(move || {
                run_http_server_multi_threaded(&server);
            });
            Ok(())
        }
        Err(err) => Err(anyhow::Error::from_boxed(err))?,
    }
}

fn run_http_server_multi_threaded(server: &Server) {
    for req in server.incoming_requests() {
        on_incoming_requests(req);
    }
}

fn on_incoming_requests(req: Request) {
    match req.method() {
        Method::Get => {
            let url = req.url();
            info!("{}", req.url());
            if url.starts_with("/static/") {
                let file_name = &url[8..];
                match Assets::get(file_name) {
                    Some(file) => {
                        let mime = get_file_mime(&file_name);
                        let bytes = file.data;
                        let res = Response::new(
                            StatusCode(200),
                            get_headers_with_content_type(format!("{}; charset=UTF-8", mime).as_str()),
                            Cursor::new(&bytes),
                            Some(bytes.len()),
                            None,
                        );
                        _ = req.respond(res);
                    }
                    None => {
                        _ = req.respond(Response::empty(404));
                    }
                }
                return;
            }
            match url {
                "/" => {
                    let index_bytes = Assets::get("index.html").unwrap().data;
                    let res = Response::new(
                        StatusCode(200),
                        get_headers_with_content_type("text/html; charset=UTF-8"),
                        Cursor::new(&index_bytes),
                        Some(index_bytes.len()),
                        None,
                    );

                    _ = req.respond(res);
                }
                "/config" => {
                    let config_bytes = Assets::get("config.html").unwrap().data;
                    let res = Response::new(
                        StatusCode(200),
                        get_headers_with_content_type("text/html; charset=UTF-8"),
                        Cursor::new(&config_bytes),
                        Some(config_bytes.len()),
                        None,
                    );

                    _ = req.respond(res);
                }
                "/get-conf" => {
                    let conf = keyboard_conf::CONF.lock().unwrap();
                    let json_bytes_result = serde_json::to_vec(&*conf);
                    if json_bytes_result.is_err() {
                        error!("Failed to serialize conf: {}", json_bytes_result.unwrap_err());
                        _ = req.respond(Response::empty(500));
                    } else {
                        let json_bytes = json_bytes_result.unwrap();
                        let len = json_bytes.len();
                        let res = Response::new(
                            StatusCode(200),
                            get_headers_with_content_type("application/json; charset=UTF-8"),
                            Cursor::new(json_bytes),
                            Some(len),
                            None,
                        );
                        _ = req.respond(res);
                    }
                }
                "/favicon.ico" => {
                    let icon_bytes = Assets::get("favicon.ico").unwrap().data;
                    let res = Response::new(
                        StatusCode(200),
                        get_headers_with_content_type("image/x-icon"),
                        Cursor::new(&icon_bytes),
                        Some(icon_bytes.len()),
                        None,
                    );

                    _ = req.respond(res);
                }
                _ => {
                    _ = req.respond(Response::empty(404));
                }
            }
        }
        Method::Post => {
            let url = req.url();
            info!("{}", req.url());
            match url {
                "/set-conf" => {
                    let mut s = String::new();
                    let mut r = req;
                    let rdr = r.as_reader();
                    let result = rdr.read_to_string(&mut s);
                    let req = r;

                    if result.is_err() {
                        info!("Failed to read request body: {}", result.unwrap_err());
                        _ = req.respond(Response::empty(500));
                        return;
                    }

                    let conf_result = serde_json::from_str::<keyboard_conf::KeyboardHookConf>(&s);
                    if conf_result.is_err() {
                        info!("Failed to deserialize json: {}", result.unwrap_err());
                        _ = req.respond(Response::empty(400));
                        return;
                    }

                    let conf = conf_result.unwrap();
                    let write_result = keyboard_conf::write_conf(conf, true);
                    if write_result.is_ok() {
                        let data = b"ok";
                        let data_len = data.len();
                        let res = Response::new(
                            StatusCode(200),
                            get_headers_with_content_type("text/plain; charset=UTF-8"),
                            Cursor::new(data),
                            Some(data_len),
                            None,
                        );
                        _ = req.respond(res);
                    } else {
                        error!("Failed to write conf: {}", result.unwrap_err());
                        _ = req.respond(Response::empty(500));
                    }
                }
                _ => {
                    _ = req.respond(Response::empty(401));
                }
            }
        }
        _ => {}
    };
}

fn get_headers_with_content_type(content_type: &str) -> Vec<Header> {
    vec![Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes()).unwrap()]
}

fn get_file_mime(file_name: &str) -> &str {
    let parts = file_name.split(".");
    let ext = parts.last().unwrap();
    return match ext {
        "html" => "text/html;",
        "css" => "text/css;",
        "js" => "application/javascript;",
        _ => "application/octet-stream",
    };
}
