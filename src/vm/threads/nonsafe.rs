/*
перенос нон-сенда в треду
 */
#[derive(Debug, Clone)]
pub struct NonSend<T> {
    value: T
}
impl<T> NonSend<T> where T: Clone {
    pub(crate) fn new(value: T) -> Self {
        NonSend { value }
    }

    pub(crate) fn get(&self) -> T {
        self.value.clone()
    }
}
unsafe impl<T> Sync for NonSend<T> {}
unsafe impl<T> Send for NonSend<T> {}