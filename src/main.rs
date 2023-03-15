// Надо сделать:
// 1) Отправку нескольких картинок
// 2) Успешн
use actix_web::{get, App, HttpServer, Responder, HttpResponse};
use actix_files::{NamedFile, Files};

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("bruh")
}

#[get("/file")]
async fn file_serve() -> impl Responder {
    NamedFile::open_async("./main.rs").await.unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(move ||{
        App::new()
            .service(hello)
            .service(file_serve)
            .service(Files::new("/files", ".").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[cfg(test)]
mod tests{
    use actix_service::Service;
    use actix_web::http::StatusCode;
    use super::*;

    #[actix_web::test]
    async fn test_hello_ok(){
        let app = actix_web::test::init_service(App::new().service(hello)).await;
        let req = actix_web::test::TestRequest::with_uri("/").to_request();
        let res = app.call(req).await.unwrap();
        let res_status = res.status();
        let res_body = actix_web::test::read_body(res).await;
        let res_body = String::from_utf8(res_body.to_vec()).unwrap();
        assert_eq!(res_status, StatusCode::OK);
        assert_eq!(res_body, "bruh");
    }

    #[actix_web::test]
    async fn test_hello_bad(){
        let app = actix_web::test::init_service(App::new().service(hello)).await;
        let bad_req = actix_web::test::TestRequest::with_uri("/hello").to_request();
        let bad_res = app.call(bad_req).await.unwrap();
        let bad_res_status = bad_res.status();
        let bad_res_body = actix_web::test::read_body(bad_res).await;
        let bad_res_body = String::from_utf8(bad_res_body.to_vec()).unwrap();
        assert_ne!(bad_res_status, StatusCode::OK);
        assert_ne!(bad_res_body, "bruh");
    }
}
