use actix_navigation_service::server::Server;
use tokio;

fn main() {
    let classroom_data = std::fs::read_to_string("classrooms.json").expect("No classrooms.json");
    let image_data = std::fs::read_to_string("images.json").expect("No images.json");
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            Server::builder()
                .image_data(image_data)
                .classroom_data(classroom_data)
                .host("0.0.0.0".to_owned())
                .port(8080)
                .build()
                .start()
                .await
                .unwrap();
        })
}
