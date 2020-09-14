use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

pub type ObjectId = Uuid;

pub trait PersistenceManager: Sized {
    type Error: Send + Sync;

    fn persistence<T, P>(self: &Arc<Self>) -> Arc<P>
    where
        T: Persistent,
        P: Persistence<Self, T>;
}

#[async_trait]
pub trait Persistence<M: PersistenceManager, T: Persistent> {
    async fn get_by_id(self: Arc<Self>, id: ObjectId) -> Result<T, M::Error>;
    async fn save(self: Arc<Self>, mut object: T) -> Result<(), M::Error>;
}

pub trait Persistent {
    fn id(&self) -> ObjectId;
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::*;

    struct Widget {
        id: ObjectId,
        value: u32,
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
        objects: HashMap<ObjectId, Box<dyn Persistent>>,
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
    }

    impl std::fmt::Display for TransientError {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
            write!(f, "({:?})", self)
        }
    }

    impl std::error::Error for TransientError {}

    impl PersistenceManager for TransientPersistence {
        type Error = TransientError;
    }

    #[async_trait]
    impl Persistence<TransientPersistence, Widget> for TransientPersistence {
        async fn get_by_id(
            self: Arc<Self>,
            id: ObjectId,
        ) -> Result<Self, <TransientPersistence as PersistenceManager>::Error> {
            // This I doubt will ever work.
            match self.objects.get(id) {
                Some(o) => Ok(o.into()),
                None => Err(),
            }
        }
    }

    #[test]
    fn test_widget() {
        let persistence = TransientPersistence::new();
        let widget = Widget::new(23);
    }
}
