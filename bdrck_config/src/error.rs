error_chain! {
    foreign_links {
        Decode(::msgpack::decode::Error);
        Encode(::msgpack::encode::Error);
        EnvVar(::std::env::VarError);
        Io(::std::io::Error);
    }
}
