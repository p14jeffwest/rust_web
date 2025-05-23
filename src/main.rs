
use std::sync::Arc;

use axum::{
    extract::Json as ExtractJson, response::{Html, IntoResponse, Redirect}, routing::{get, post}, Json, Router
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

    // 2. HTTP 및 HTTPS 서버 시작
    let http = tokio::spawn(http_server());    
    let https = tokio::spawn(https_server());
    let _ = tokio::join!(http, https);

    // 서버에서 HTTPS 사용할 수 있도록 설정완료 전에 사용하는 코드 
    // let http = tokio::spawn(http_server());        
    // let _ = tokio::join!(http);
}


// HTTP로 들어온 요청을 HTTPS로 리다이렉트한다. 
async fn http_server(){
    let app = Router::new().route("/", get(http_handler));
    let config = Arc::new(rust_web::get_config());
    let http_addr = config.base_http_url.parse::<SocketAddr>().unwrap(); //127.0.0.1:8000 or 0.0.0.1:80  

    println!("HTTP Listening on {}", http_addr);
    axum_server::bind(http_addr)
        .serve(app.into_make_service())
        .await
        .unwrap();   
}

async fn http_handler() -> Redirect {  
    let config = Arc::new(rust_web::get_config());
    let uri = config.https_redirect;      
    Redirect::temporary(uri)

    // let uri = format!("https://127.0.0.1:443");
    // Redirect::temporary(&uri)
}



// for https
async fn https_server() {
    // 1. 한자 변환 사전을 만들어 둔다. 
    let dic_arc = match rust_web::load_arc_dictionary() {
        Ok(dic) => dic,
        Err(e) => {
            log::error!("사전 로드 실패: {}", e);
            return;
        }        
    };   

    //2. cargo run --dev 혹은 cargo run --prod
    let config = Arc::new(rust_web::get_config());
    println!("Running in mode: {}", config.mode);

    //3. https 서버를 시작한다.
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
        config.ssl_cert,  //local cert: cert_local/cert.pem, LightSail cert: /etc/letsencrypt/live/badang.xyz/fullchain.pem
        config.ssl_key,   //local key: cert_local/key.pem, LightSail key: /etc/letsencrypt/live/badang.xyz/privkey.pem        
    ).await {
        Ok(config) => config,
        Err(e) => {
            log::error!("TLS 설정 실패: {}", e);
            return;
        }
    };    

    let https_addr = config.base_https_url.parse::<SocketAddr>().unwrap(); //127.0.0.1:443 or 0.0.0.1:443
    println!("HTTPS Listening on {}", https_addr);
    axum_server::bind_rustls(https_addr, rustls_config)
        .serve(app.into_make_service())
        .await
        .unwrap();
}


