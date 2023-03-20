use actix_web::dev::ServiceResponse;
use actix_web::{get, App, HttpServer, Responder, HttpResponse, web};
use super::mongo_client::DBClient;
use std::error::Error;
use std::sync::Mutex; 


#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct ClassroomDataRequest{
    name: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct TestQuery{
    id: u64
}

#[get("/test")]
async fn get_test_query(info: web::Query<TestQuery>) -> impl Responder {
    HttpResponse::Ok().body(format!("id = {}", info.id))
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
    let query = query.into_inner();
    match db_client.get_classroom_data(query.name.to_owned()).await {
        Ok(val) => {HttpResponse::Ok().body(val)},
        Err(e) => {HttpResponse::NotFound().body(format!("Error: classroom data not available\nReason:{:?}", e))}
    }
}

pub struct Server{
    host: String,
    port: u16,
    classroom_data: String,
    image_data: String,
}

impl Server{
    pub fn builder() -> ServerBuilder {
        ServerBuilder { 
            host: None,
            port: None,
            classroom_data: None,
            image_data: None }
    }
    pub async fn start(self) -> Result<(), Box<dyn Error>> {
        let mongo_client = web::Data::new(Mutex::new(DBClient::new(self.classroom_data, self.image_data).await?));
        HttpServer::new(move ||{
            App::new()
                .app_data(mongo_client.clone())
                .service(get_classroom_list)
                .service(get_classroom_data)
        })
        .bind((self.host, self.port))?
        .run()
        .await?;
        Ok(())
    }

    pub async fn test_start(self) -> Result<impl actix_service::Service<actix_http::Request, Response = ServiceResponse, Error = actix_web::Error>, Box<dyn Error>> {
        let mongo_client = web::Data::new(Mutex::new(DBClient::new(self.classroom_data, self.image_data).await?));
        let app = actix_web::test::init_service(App::new()
            .app_data(mongo_client.clone())
            .service(get_test_query)
            .service(get_classroom_list)
            .service(get_classroom_data))
            .await;
        Ok(app)
    }
}

pub struct ServerBuilder{
    host: Option<String>,
    port: Option<u16>,
    classroom_data: Option<String>,
    image_data: Option<String>,
}

impl ServerBuilder{
    pub fn host(mut self, value: String) -> Self {
        self.host = Some(value);
        self
    }

    pub fn port(mut self, value: u16) -> Self {
        self.port = Some(value);
        self
    }

    pub fn classroom_data(mut self, value: String) -> Self {
        self.classroom_data = Some(value);
        self
    }

    pub fn image_data(mut self, value: String) -> Self {
        self.image_data = Some(value);
        self
    }

    pub fn build(self) -> Server {
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
    use serde_json::json;
    use serial_test::serial;
    use actix_service::Service;
    use actix_web::http::StatusCode;
    use super::*;

    fn set_env_vars()
    {
        dotenv::dotenv().ok();
    }

    fn valid_classroom_data() -> String {
        json!([{
                "classroom": "УК3 104",
                "description": "Крутая аудитория",
                "images": ["UK3-left.png", "UK3-right.png"]
            },
            {
                "classroom": "УК3 205",
                "description": "Менее крутая аудитория",
                "images": ["UK3-left.png", "UK3-right.png"]
            },
        ]).to_string()
    }

    fn valid_image_data() -> String {
        json!([{
                "image_name": "UK3-left.png",
                "image": "bibabob",
            },
            {
                "image_name": "UK3-right.png",
                "image": "pipupap",
            },
            ]).to_string()
    }

    #[actix_web::test]
    #[serial]
    async fn test_id(){
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .classroom_data(valid_classroom_data())
            .image_data(valid_image_data())
            .build()
            .test_start()
            .await
            .unwrap();
        let req = actix_web::test::TestRequest::get()
            .uri("/test?id=104")
            .to_request();
        //let req = actix_web::test::TestRequest::with_uri("/classroom?name=УК3 104").to_request();
        let res = app.call(req).await.unwrap();
        let res_status = res.status();
        let res_body = actix_web::test::read_body(res).await;
        let res_body = String::from_utf8(res_body.to_vec()).unwrap();
        let awaited_body = "id = 104".to_owned();
        assert_eq!(res_status, StatusCode::OK);
        assert_eq!(res_body, awaited_body);
    }

    #[actix_web::test]
    #[serial]
    async fn test_server_init(){
        set_env_vars();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .classroom_data(valid_classroom_data())
            .image_data(valid_image_data())
            .build()
            .test_start()
            .await
            .unwrap();
    }

    #[actix_web::test]
    #[serial]
    async fn test_classroom_list_ok(){
        set_env_vars();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .classroom_data(valid_classroom_data())
            .image_data(valid_image_data())
            .build()
            .test_start()
            .await
            .unwrap();
        let req = actix_web::test::TestRequest::with_uri("/classroomlist").to_request();
        let res = app.call(req).await.unwrap();
        let res_status = res.status();
        let res_body = actix_web::test::read_body(res).await;
        let res_body = String::from_utf8(res_body.to_vec()).unwrap();
        let awaited_body = json!(["УК3 104", "УК3 205"]).to_string();
        assert_eq!(res_status, StatusCode::OK);
        assert_eq!(res_body, awaited_body);
    }
/*
    #[actix_web::test]
    #[serial]
    async fn test_classroom_data_bad(){
        set_env_vars();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .classroom_data(valid_classroom_data())
            .image_data(valid_image_data())
            .build()
            .test_start()
            .await
            .unwrap();
        let encoded_uri: String = url::form_urlencoded::byte_serialize("/classroom?name=УК3 104".as_bytes()).collect();
        //panic!("URL: {encoded_uri}");
        let bad_req = actix_web::test::TestRequest::with_uri(&encoded_uri).to_request();
        let bad_res = app.call(bad_req).await.unwrap();
        let bad_res_status = bad_res.status();

        assert_ne!(bad_res_status, StatusCode::OK);
    }

    #[actix_web::test]
    #[serial]
    async fn test_classroom_data_ok(){
        set_env_vars();
        let app = Server::builder()
            .host("localhost".to_owned())
            .port(8080)
            .classroom_data(valid_classroom_data())
            .image_data(valid_image_data())
            .build()
            .test_start()
            .await
            .unwrap();
        let encoded_uri: String = url::form_urlencoded::byte_serialize("localhost:8080/classroom?name=УК3 104".as_bytes()).collect();
        //let encoded_uri = urlencoding::encode("/classroom?name=УК3 104").to_owned();
        //panic!("URL: {encoded_uri}");
        let req = actix_web::test::TestRequest::get()
            .uri(&encoded_uri)
            .to_request();
        let res = app.call(req).await.unwrap();
        let res_status = res.status();
        let res_body = actix_web::test::read_body(res).await;
        let res_body = String::from_utf8(res_body.to_vec()).unwrap();
        let awaited_body = json!({
            "classroom": "УК3 104",
            "description": "Крутая аудитория",
            "images": ["bibabob", "pipupap"],
        }).to_string();
        assert_eq!(res_status, StatusCode::OK);
        assert_eq!(res_body, awaited_body);
    }
*/
}

