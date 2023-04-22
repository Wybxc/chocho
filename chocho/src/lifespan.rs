//! 生命周期管理。

use std::{future::Future, pin::Pin};

use once_cell::sync::Lazy;
use std::sync::Mutex;

type Finalizer = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

static FINALIZERS: Lazy<Mutex<Vec<Finalizer>>> = Lazy::new(|| Mutex::new(vec![]));

/// 注册一个生命周期结束时执行的函数。
pub fn finalizer<Fut>(f: impl FnOnce() -> Fut + Send + 'static)
where
    Fut: Future<Output = ()> + Send + 'static,
{
    let mut finalizers = FINALIZERS.lock().expect("Failed locking FINALIZERS");
    finalizers.push(Box::new(move || Box::pin(f())));
}

/// 执行所有注册的生命周期结束时执行的函数。
///
/// 执行顺序为注册顺序的逆序。
pub async fn do_finalize() {
    let finalizers = {
        let mut finalizers = FINALIZERS.lock().expect("Failed locking FINALIZERS");
        finalizers.drain(..).rev().collect::<Vec<_>>()
    };
    for f in finalizers {
        f().await;
    }
}
