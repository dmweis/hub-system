use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{Arc, Mutex},
};

type Handle = Arc<dyn Any + Send + Sync>;

#[derive(Debug, Clone, Default)]
pub struct IocContainer {
    map: Arc<Mutex<HashMap<TypeId, Handle>>>,
}

impl IocContainer {
    pub fn register<T: Any + Send + Sync>(&self, object: T) {
        let type_id = object.type_id();
        self.map.lock().unwrap().insert(type_id, Arc::new(object));
    }

    pub fn get<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        let type_id = TypeId::of::<T>();
        if let Some(object) = self.map.lock().unwrap().get(&type_id) {
            let handle = object.clone();
            handle.downcast::<T>().ok()
        } else {
            None
        }
    }
}
