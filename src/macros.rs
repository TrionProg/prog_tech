#[macro_export]
macro_rules! try_send {
    [ $sender:expr, $message:expr ] => {
        if $sender.send( $message ).is_err() {
            //error!( "Can not send {}", stringify!($message) );
            panic!( "Can not send {}", stringify!($message) );
        }
    };
}

#[macro_export]
macro_rules! recv_error {
    ( $expected:path ) => {{
        //error!( "Can not recv {}", stringify!($expected) );
        panic!( "Can not recv {}", stringify!($expected) );
    }};
}