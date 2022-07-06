#[macro_export]
macro_rules! version {
    () => {
        env!("CARGO_PKG_VERSION")
    };
}

#[macro_export]
macro_rules! match_key {
    ( $code:pat , $modifier:pat ) => {
        Event::Key(KeyEvent {
            code: $code,
            modifiers: $modifier,
        })
    };
    ( $code:pat ) => {
        Event::Key(KeyEvent {
            code: $code,
            ..
        })
    }
}

#[macro_export]
macro_rules! key {
    ( $bind:ident ) => {
        Event::Key(KeyEvent {
            code: KeyCode::Char($bind),
            ..
        })
    };
}
