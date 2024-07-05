use std::{
    error::Error,
    ffi::OsString,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize, Serializer};
use tokio::{
    fs::{self, ReadDir},
    io,
    sync::{Mutex, OnceCell},
};

use crate::data::{
    self, ConfigError, ConfigFile, DataManager, OpenError, ResourceManager, WritableConfigFile,
};

pub struct StateWrapper {
    inner: Arc<OnceCell<State>>,
}

impl StateWrapper {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(OnceCell::new()),
        }
    }
    async fn state(&self) -> Result<&State, OpenError> {
        self.inner.get_or_try_init(|| State::new()).await
    }
}

struct State {
    data_dir: PathBuf,
    course_maps: DataManager,
    courses: ResourceManager,
    progress: DataManager,
    active_courses: Mutex<WritableConfigFile>,
    settings: Mutex<WritableConfigFile>,
}

impl State {
    async fn new() -> Result<Self, OpenError> {
        let data_dir = data::get_data_dir(crate::IDENTIFIER).await?;

        let extension = Some(OsString::from("toml"));
        let course_map_path = data_dir.join("Course Maps");
        let course_path = data_dir.join("Courses");
        let progress_path = data_dir.join("Progress");
        let active_courses_path = data_dir.join("Active Courses.toml");
        let settings_path = data_dir.join("Settings.toml");

        Ok(Self {
            data_dir,
            course_maps: DataManager::new(course_map_path, extension.clone()).await?,
            courses: ResourceManager::new(course_path).await?,
            progress: DataManager::new(progress_path, extension).await?,
            active_courses: Mutex::new(WritableConfigFile::new(&active_courses_path).await?),
            settings: Mutex::new(WritableConfigFile::new(&settings_path).await?),
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Course {
    title: String,
    description: Option<String>,
    books: Vec<(String, Textbook)>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Textbook {
    file: PathBuf,
    chapters: Vec<Chapter>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Chapter {
    id: Option<String>,
    sections: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorWrapper {
    message: String,
    cause: serde_error::Error,
}

impl ErrorWrapper {
    fn new<T>(message: String, inner: &T) -> Self
    where
        T: ?Sized + std::error::Error,
    {
        Self {
            message,
            cause: serde_error::Error::new(inner),
        }
    }
}

#[tauri::command]
pub async fn open_data_dir(state: tauri::State<'_, StateWrapper>) -> Result<(), ErrorWrapper> {
    let state = state
        .state()
        .await
        .map_err(|e| ErrorWrapper::new("Unable to open application directories".to_string(), &e))?;

    open::that_detached(&state.data_dir).map_err(|e| {
        ErrorWrapper::new(
            format!("Unable to launch OS opener for {:?}", &state.data_dir),
            &e,
        )
    })?;

    Ok(())
}

/*#[tauri::command]
pub async fn get_courses(
    window: tauri::Window,
    app_handle: tauri::AppHandle,
    state: tauri::State<'_, StateWrapper>,
) -> Result<Vec<Course>, String> {

    todo!();
}*/

/*#[tauri::command]
pub async fn open_data_dir(app_handle: tauri::AppHandle) -> Result<(), String> {
    let path = get_data_dir(app_handle).ok_or("Unable to find application data directory")?;

    ensure_folder_exists(&path)
        .await
        .map_err(|err| format!("Unable to open application data directory: {}", err))?;

    open::that_detached(&path)
        .map_err(|err| format!("Unable to launch system file opener: {}", err))
}*/
