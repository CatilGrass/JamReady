#[macro_export]
macro_rules! entry_mutex {
    ($mutex:expr, |$guard:ident| $code:expr) => {
        if let Ok(mut $guard) = $mutex.lock() {
            let $guard = &mut $guard;
            $code
        }
    };
}

#[macro_export]
macro_rules! entry_mutex_async {
    ($mutex:expr, |$guard:ident| $code:expr) => {{
        let mut $guard = $mutex.lock().await;
        let $guard = &mut $guard;
        $code
    }};
}

#[macro_export]
macro_rules! connect_once {
    ($addr:expr, |$conn:ident| $code:block) => {{
        use tokio::net::TcpStream;
        use log::{error};
        match TcpStream::connect($addr).await {
            Ok(mut $conn) => {
                if let Some(result) = $code {
                    Some(result)
                } else {
                    None
                }
            },
            Err(e) => {
                error!("Connection failed {:?}", e);
                None
            }
        }
    }}
}