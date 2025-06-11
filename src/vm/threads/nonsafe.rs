// структура для переноса non-sendable
#[derive(Debug, Clone)]
pub struct NonSend<T> {
    value: T
}

// имплементация
impl<T> NonSend<T> where T: Clone {
    pub(crate) fn new(value: T) -> Self {
        NonSend { value }
    }

    pub(crate) fn get(&self) -> T {
        self.value.clone()
    }
}

/*
 имплементация Sync и Send для переноса
 non-sendable между потоками
 */
unsafe impl<T> Sync for NonSend<T> {}
unsafe impl<T> Send for NonSend<T> {}