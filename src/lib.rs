use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type ObjectId = Uuid;

#[async_trait]
pub trait PersistenceManager {
    type Error: Send + Sync;

    async fn get_by_id<T>(self: Arc<Self>, id: ObjectId) -> Result<T, Self::Error>
    where
        T: Persistent + Deserialize<'async_trait>;
    async fn save<T>(self: Arc<Self>, mut object: T) -> Result<(), Self::Error>
    where
        T: Persistent + Serialize;
}

pub trait Persistent {
    fn id(&self) -> ObjectId;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_json;

    use crate::*;

    struct Widget {
        id: ObjectId,
        pub value: u32,
    }

    impl Widget {
        fn new(value: u32) -> Self {
            Self {
                id: ObjectId::new_v4(),
                value,
            }
        }
    }

    impl Persistent for Widget {
        fn id(&self) -> ObjectId {
            self.id
        }
    }

    struct TransientPersistence {
        objects: HashMap<ObjectId, String>,
    }

    impl TransientPersistence {
        fn new() -> Self {
            Self {
                objects: HashMap::new(),
            }
        }
    }

    #[derive(Debug)]
    enum TransientError {
        NoObject,
        BadObject,
    }

    impl std::fmt::Display for TransientError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            write!(f, "({:?})", self)
        }
    }

    impl std::error::Error for TransientError {}

    #[async_trait]
    impl PersistenceManager for TransientPersistence {
        type Error = TransientError;

        async fn get_by_id<T>(self: Arc<Self>, id: ObjectId) -> Result<T, Self::Error>
        where
            T: Persistent + Deserialize<'async_trait>,
        {
            let json = match self.objects.get(&id) {
                Some(s) => s,
                None => return Err(TransientError::NoObject),
            };

            serde_json::from_str(json).map_err(|_| TransientError::BadObject)
        }

        async fn save<T>(self: Arc<Self>, mut object: T) -> Result<(), Self::Error>
        where
            T: Persistent + Serialize,
        {
            let json = serde_json::to_string(object).map_err(|_| TransientError::BadObject)?;
            self.objects.insert(object.id(), json);
            Ok(())
        }
    }

    #[test]
    fn test_widget() {
        let persistence = TransientPersistence::new();
        let widget = Widget::new(23);

        persistence.save(widget);
        let w2 = persistence.get_by_id(widget.id());

        assert_eq!(widget.id(), w2.id());
        assert_eq!(widget.value, w2.value);
    }
}
