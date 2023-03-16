use actix_web::error::ErrorNotFound;
use mongodb::{Client, options::{ClientOptions, Credential, ServerAddress}, bson::doc};
use std::env;
use std::error::Error;
use futures::stream::TryStreamExt;



#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct ClassroomData{
    classroom: String,
    description: String,
    images: Vec<String>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
struct CampusImage{
    image_name: String,
    image: String,
}


pub struct DBClient{
    inner_client: Client,
}

impl DBClient{
    /// Создает новый клиент Монго-базы
    /// 
    /// Все нужные переменные должны быть указаны в env файле и переданы
    /// в контейнер.
    pub async fn new() -> Result<Self, Box<dyn Error>> {
        let credentials = Credential::builder()
            .username(env::var("MONGODB_USERNAME").unwrap_or("biba".to_owned()))
            .password(env::var("MONGODB_PASSWORD").unwrap_or("boba".to_owned()))
            .build();
        let options = ClientOptions::builder()
            .credential(credentials)
            .hosts(vec![ServerAddress::Tcp {
                host: env::var("DB_CONTAINER_NAME").unwrap_or("localhost".to_owned()),
                port: Some(8080)}])
            .build();
        let client = Client::with_options(options)?;
        client
            .database("admin")
            .run_command(doc!{"ping": 1}, None)
            .await?;
        Ok(Self{inner_client: client})
    }

    pub async fn get_classroom_list(&self) -> Result<String, Box<dyn Error>> {
        let classroom_collection = self.inner_client
            .database("navigation_data")
            .collection::<ClassroomData>("classrooms");
        let cursor = classroom_collection
            .find(None,None)
            .await?;
        let classrooms: Vec<ClassroomData> = cursor.try_collect().await?;
        let classrooms: Vec<String> = classrooms
            .iter()
            .map(|x|x.classroom.to_owned())
            .collect();
        let json_data = serde_json::to_string(&classrooms)?; 
        Ok(json_data)
    }

    async fn get_campus_images(&self, image_names: &Vec<String>) -> Result<Vec<CampusImage>, Box<dyn Error>> {
        let image_collection = self.inner_client
            .database("navigation_data")
            .collection::<CampusImage>("images");
        let cursor = image_collection.
            find(None, None)
            .await?;
        let images: Vec<CampusImage> = cursor.try_collect().await?;
        let needed_images: Vec<CampusImage> = images
            .into_iter()
            .filter(|image|{image_names.contains(&image.image_name)})
            .collect();
        if needed_images.len() > 0{
            return Err(Box::new(ErrorNotFound("No images found")));
        }
        Ok(needed_images)
    }

    pub async fn get_classroom_data(&self, classroom_name: String) -> Result<String, Box<dyn Error>> {
        let classroom_collection = self.inner_client
            .database("navigation_data")
            .collection::<ClassroomData>("classrooms");
        let classroom_cursor = classroom_collection.find(None, None).await?;
        let mut classrooms: Vec<ClassroomData> = classroom_cursor.try_collect().await?;
        
        let mut needed_classroom = match classrooms.iter_mut().find(|classroom|{classroom.classroom == classroom_name}){
            Some(classroom) => classroom,
            None => return Err(Box::new(ErrorNotFound("Classroom not found"))),
        };

        let classroom_images = self.get_campus_images(&needed_classroom.images).await?;
        let classroom_images: Vec<String> = classroom_images
            .into_iter()
            .map(|elem|elem.image)
            .collect();
        needed_classroom.images = classroom_images;
        let result = serde_json::to_string(needed_classroom)?;
        Ok(result)
    }

    pub async fn fill_classroom_data(&self, data: String) -> Result<(), Box<dyn Error>> {
        let classroom_collection = self.inner_client
            .database("navigation_data")
            .collection::<ClassroomData>("classrooms");
        classroom_collection.drop(None).await?;
        todo!()
    }
}

#[cfg(test)]
mod tests{
}
