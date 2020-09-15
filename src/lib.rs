use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type ObjectId = Uuid;

#[async_trait]
pub trait PersistenceManager {
    type Error: Send + Sync;

    async fn get_by_id<T>(&'async_trait self, id: ObjectId) -> Result<T, Self::Error>
    where
        T: Persistent + Deserialize<'async_trait>;
    async fn save<T>(&mut self, object: &T) -> Result<(), Self::Error>
    where
        T: Persistent + Serialize;
}

pub trait Persistent: Send + Sync {
    fn id(&self) -> ObjectId;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde_derive::{Deserialize, Serialize};
    use serde_json;

    use crate::*;

    #[derive(Deserialize, Serialize)]
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

    #[derive(Deserialize, Serialize)]
    struct Thingy {
        id: ObjectId,
        pub value: String,
    }

    impl Thingy {
        fn new<S>(value: S) -> Self
        where
            S: ToString,
        {
            Self {
                id: ObjectId::new_v4(),
                value: value.to_string(),
            }
        }
    }

    impl Persistent for Thingy {
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

        async fn get_by_id<T>(&'async_trait self, id: ObjectId) -> Result<T, Self::Error>
        where
            T: Persistent + Deserialize<'async_trait>,
        {
            let json = match self.objects.get(&id) {
                Some(s) => s,
                None => return Err(TransientError::NoObject),
            };

            serde_json::from_str(json).map_err(|_| TransientError::BadObject)
        }

        async fn save<T>(&mut self, object: &T) -> Result<(), Self::Error>
        where
            T: Persistent + Serialize,
        {
            let json = serde_json::to_string(object).map_err(|_| TransientError::BadObject)?;
            self.objects.insert(object.id(), json);
            Ok(())
        }
    }

    #[async_std::test]
    async fn test_widget() -> Result<(), TransientError> {
        let mut persistence = TransientPersistence::new();
        let widget = Widget::new(23);

        persistence.save(&widget).await?;
        let w2: Widget = persistence.get_by_id(widget.id()).await?;

        assert_eq!(widget.id(), w2.id());
        assert_eq!(widget.value, w2.value);

        Ok(())
    }

    #[async_std::test]
    async fn test_thingy() -> Result<(), TransientError> {
        let mut persistence = TransientPersistence::new();
        let thingy = Thingy::new("twenty-three");

        persistence.save(&thingy).await?;
        let t2: Thingy = persistence.get_by_id(thingy.id()).await?;

        assert_eq!(thingy.id(), t2.id());
        assert_eq!(thingy.value, t2.value);

        Ok(())
    }
}
