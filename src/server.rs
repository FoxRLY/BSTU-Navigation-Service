use actix_web::{get, App, HttpServer, Responder, HttpResponse, web};
use super::mongo_client::DBClient;
use std::error::Error;
use std::sync::Mutex;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ClassroomDataRequest{
    name: String,
}

#[get("/classroomlist")]
async fn get_classroom_list(db_client: web::Data<Mutex<DBClient>>) -> impl Responder {
    let db_client = db_client.lock().unwrap();
    match db_client.get_classroom_list().await {
        Ok(val) => {HttpResponse::Ok().body(val)},
        Err(e) => {HttpResponse::NotFound().body(format!("Error: classroom list not available\nReason:{:?}", e))}
    }
}

#[get("/classroom")]
async fn get_classroom_data(query: web::Query<ClassroomDataRequest>, db_client: web::Data<Mutex<DBClient>>) -> impl Responder {
    let db_client = db_client.lock().unwrap();
    match db_client.get_classroom_data(query.name.to_owned()).await {
        Ok(val) => {HttpResponse::Ok().body(val)},
        Err(e) => {HttpResponse::NotFound().body(format!("Error: classroom data not available\nReason:{:?}", e))}
    }
}

struct Server{
    host: String,
    port: u16,
    classroom_data: String,
    image_data: String,
}

impl Server{
    fn builder() -> ServerBuilder {
        ServerBuilder { 
            host: None,
            port: None,
            classroom_data: None,
            image_data: None }
    }
    async fn start(self) -> Result<(), Box<dyn Error>> {
        let mongo_client = web::Data::new(Mutex::new(DBClient::new(self.classroom_data, self.image_data).await?));
        HttpServer::new(move ||{
            App::new()
                .app_data(web::Data::new(mongo_client.clone()))
                .service(get_classroom_list)
                .service(get_classroom_data)
        })
        .bind(("localhost", 8080))?
        .run()
        .await?;
        Ok(())
    }
}

struct ServerBuilder{
    host: Option<String>,
    port: Option<u16>,
    classroom_data: Option<String>,
    image_data: Option<String>,
}

impl ServerBuilder{
    fn host(mut self, value: String) -> Self {
        self.host = Some(value);
        self
    }

    fn port(mut self, value: u16) -> Self {
        self.port = Some(value);
        self
    }

    fn classroom_data(mut self, value: String) -> Self {
        self.classroom_data = Some(value);
        self
    }

    fn image_data(mut self, value: String) -> Self {
        self.image_data = Some(value);
        self
    }

    fn build(self) -> Server {
        Server {
            host: self.host.unwrap_or("localhost".to_string()),
            port: self.port.unwrap_or(8080),
            classroom_data: self.classroom_data.unwrap_or("[]".to_string()),
            image_data: self.image_data.unwrap_or("[]".to_string())}
    }
}
/*
#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Server::builder()
        .host("localhost".to_owned())
        .port(8080)
        .classroom_data("[]".to_owned())
        .image_data("[]".to_owned())
        .build()
        .start()
        .await?;
    Ok(())
}
*/
#[cfg(test)]
mod tests{
    use serial_test::serial;
    use actix_service::Service;
    use actix_web::http::StatusCode;
    use super::*;

    #[actix_web::test]
    #[serial]
    async fn test_classroom_list_ok(){
        let mongo_client = web::Data::new(Mutex::new(DBClient::new("[]".to_owned(), "[]".to_owned()).await.unwrap()));
        let app = actix_web::test::init_service(App::new()
            .app_data(mongo_client.clone())
            .service(get_classroom_list)
            .service(get_classroom_data)
            ).await;
        let req = actix_web::test::TestRequest::with_uri("/classroomlist").to_request();
        let res = app.call(req).await.unwrap();
        let res_status = res.status();
        let res_body = actix_web::test::read_body(res).await;
        let res_body = String::from_utf8(res_body.to_vec()).unwrap();
        assert_eq!(res_status, StatusCode::OK);
        assert_eq!(res_body, "[]");
    }
/*
    #[actix_web::test]
    #[serial]
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
*/
}

