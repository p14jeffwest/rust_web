
use std::sync::Arc;

use axum::{
    routing::get,
    routing::post,
    Router,
    response::Html,
    response::IntoResponse,
    Json,
    extract::Json as ExtractJson,
};
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;

use serde::{Deserialize, Serialize};
use tower_http::services::ServeDir;

#[derive(Deserialize)]
struct InputData {
    text: String,
}

#[derive(Serialize)]
struct OutputData {
    converted_text: String,
}



async fn hello_rust() -> impl IntoResponse {
    let path = std::path::PathBuf::from("index.html");
    let html = match tokio::fs::read_to_string(&path).await {
        Ok(html) => html,
        Err(e) => {
            log::error!("Unable to read file: {}", e);
            return Html("Error reading file".to_string());
        }
    };    
    Html(html)
}

async fn convert_handler(
        ExtractJson(payload): ExtractJson<InputData>,
        dic: Arc<rust_web::Dictionary>) -> impl IntoResponse {    
    match rust_web::convert_str(&payload.text, &dic.char_dic, &dic.dueum_dic, &dic.word_dic).await {
        Some(converted_text) => {
            let response = OutputData {
                converted_text,
            };
            return Json(response);
        },
        None => {
            // 변환할 수 없는 경우
            let response = OutputData {
                converted_text: "변환할 수 없습니다.".to_string(),
            };
            return Json(response);
        }
    }        
}



#[tokio::main]
async fn main() {  
    // 1. log4rs 설정
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    log::info!("Starting server...");

    // 3. HTTP 및 HTTPS 서버 시작
    let http = tokio::spawn(http_server());
    let https = tokio::spawn(https_server());
    let _ = tokio::join!(http, https);
}


// for http
async fn http_server() {
    // 한자 변환 사전을 만들어 둔다. dic=(char_dic, dueum_dic, word_dic)
    let dic_arc = match rust_web::load_arc_dictionary() {
        Ok(dic) => dic,
        Err(e) => {
            log::error!("사전 로드 실패: {}", e);
            return;
        }        
    };   

    let app = Router::new()
    .route("/", get(hello_rust))
    .route(
        "/convert", 
        post({                
            let  dic_clone = std::sync::Arc::clone(&dic_arc);
            move |payload| convert_handler(payload, dic_clone)                
        }),
    )
    .nest_service("/css", ServeDir::new("css"))
    .nest_service("/js", ServeDir::new("js"))
    ;

    let http_addr = "127.0.0.1:8000".parse::<SocketAddr>().unwrap();    
    let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap();
    println!("HTTPS Listening on {}", http_addr);
    axum::serve(listener, app).await.unwrap();
}


async fn https_server() {
    // 한자 변환 사전을 만들어 둔다. dic=(char_dic, dueum_dic, word_dic)
    let dic_arc = match rust_web::load_arc_dictionary() {
        Ok(dic) => dic,
        Err(e) => {
            log::error!("사전 로드 실패: {}", e);
            return;
        }        
    };   

    let app = Router::new()
    .route("/", get(hello_rust))
    .route(
        "/convert", 
        post({                
            let  dic_clone = std::sync::Arc::clone(&dic_arc);
            move |payload| convert_handler(payload, dic_clone)                
        }),
    )
    .nest_service("/css", ServeDir::new("css"))
    .nest_service("/js", ServeDir::new("js"))
    ;

    // for https    
    let rustls_config = match RustlsConfig::from_pem_file(
        "cert_local/cert.pem",
        "cert_local/key.pem",           
    ).await {
        Ok(config) => config,
        Err(e) => {
            log::error!("TLS 설정 실패: {}", e);
            return;
        }
    };    

    let https_addr = "127.0.0.1:443".parse::<SocketAddr>().unwrap();
    println!("HTTPS Listening on {}", https_addr);
    axum_server::bind_rustls(https_addr, rustls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();

}


