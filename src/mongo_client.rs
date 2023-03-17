use actix_web::error::ErrorNotFound;
use mongodb::{Client, options::{ClientOptions, Credential, ServerAddress}, bson::doc, Collection};
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

/// Клиент Монго-базы для сервиса навигации
/// 
/// # Что делает?
///     - Заполняет базу отформатированными данными об аудиториях и картинках
///     - Выдает список всех аудиторий
///     - Выдает данные о местоположении аудиторий
pub struct DBClient{
    inner_client: Client,
    database_name: String,
    image_coll_name: String,
    classroom_coll_name: String,
}

impl DBClient{
    /// Создает новый клиент Монго-базы
    /// 
    /// # Аргументы
    /// - classroom_data: навигационные данные для аудиторий в виде ClassroomData в JSON
    /// - image_data: картинки для аудиторий в виде CampusImage в JSON
    /// 
    /// # Примечание:
    /// Все нужные переменные среды должны быть указаны в env файле и переданы
    /// в контейнер.
    pub async fn new(classroom_data: String, image_data: String) -> Result<Self, Box<dyn Error>> {
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
        
        let inner_client = Self{
            inner_client: client,
            database_name: "navigation_data".to_owned(),
            classroom_coll_name: "classrooms".to_owned(),
            image_coll_name: "images".to_owned()};

        inner_client.ping().await?;
        inner_client.fill_image_data(image_data).await?;
        inner_client.fill_classroom_data(classroom_data).await?;

        Ok(inner_client)
    }
    
    /// Выдает список всех аудиторий в виде JSON-строки
    pub async fn get_classroom_list(&self) -> Result<String, Box<dyn Error>> {
        let classroom_collection = self.get_classroom_collection();
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

    /// Выдает данные о местоположении аудитории в виде JSON-строки
    /// с закодированными в Base64 картинками 
    ///
    /// # Аргументы:
    /// - classroom_name: Имя адуитории
    pub async fn get_classroom_data(&self, classroom_name: String) -> Result<String, Box<dyn Error>> {
        let classroom_collection = self.get_classroom_collection();
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

    /// Проверка подключения клиента к базе
    async fn ping(&self) -> Result<(), Box<dyn Error>> {
        self.inner_client
            .database("admin")
            .run_command(doc!{"ping": 1}, None)
            .await?;
        Ok(())
    }
    
    /// Выдает картинки из базы данных
    ///
    /// # Аргументы:
    /// - image_names: список названий картинок, которые должны быть выданы
    ///
    /// # Примечание:
    /// Если найдена хотя бы одна картинка, то функция не выдает ошибки(может измениться)
    async fn get_campus_images(&self, image_names: &Vec<String>) -> Result<Vec<CampusImage>, Box<dyn Error>> {
        let image_collection = self.get_image_collection();
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
    
    /// Заполнить базу навигационными данными аудиторий
    ///
    /// # Аргументы:
    /// - data: навигационные данные в виде JSON-строки из соответствующего файла
    ///
    /// # Примечание:
    /// Прошлые навиигационные данные стираются, а не дополняются или обновляются
    async fn fill_classroom_data(&self, data: String) -> Result<(), Box<dyn Error>> {
        let classroom_collection = self.get_classroom_collection();
        classroom_collection.drop(None).await?;

        let classroom_data: Vec<ClassroomData> = serde_json::from_str(&data)?;
        classroom_collection.insert_many(classroom_data, None).await?;
        Ok(())
    }

    /// Запонить базу картинками корпусов
    ///
    /// # Аргументы:
    /// - data: картинки в виде JSON-строки из соответствующего файла
    ///
    /// # Примечание:
    /// Прошлые навигационные данные стираются, а не дополняются или обновляются
    async fn fill_image_data(&self, data: String) -> Result<(), Box<dyn Error>> {
        let image_collection = self.get_image_collection();
        image_collection.drop(None).await?;

        let image_data: Vec<CampusImage> = serde_json::from_str(&data)?;
        image_collection.insert_many(image_data, None).await?;
        Ok(())
    }

    /// Выдает хэндл коллекции картинок из базы
    fn get_image_collection(&self) -> Collection<CampusImage> {
        self.inner_client
            .database(&self.database_name)
            .collection::<CampusImage>(&self.image_coll_name)
    }

    /// Выдает хэндл коллекции аудиторий из базы
    fn get_classroom_collection(&self) -> Collection<ClassroomData> {
        self.inner_client
            .database(&self.database_name)
            .collection::<ClassroomData>(&self.classroom_coll_name)
    }
}

#[cfg(test)]
mod tests{
    use super::*;
}
