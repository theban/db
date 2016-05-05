extern crate rmp;

use std;

quick_error! {
    #[derive(Debug)]
    pub enum DBError {
        Protocol(err: String) {
            description("Protocol error")
            display("Protocol error: {}", err)
        }
        FileFormat(err: String) {
            description("File fromat error")
            display("File format error: {}", err)
        }
        ParseString(err: String) {
            description("Parse string error")
            display("Parse string error")
            from(rmp::decode::DecodeStringError)
        }
        ParseValue(err: rmp::decode::ValueReadError){
            description("Parse value error")
            display("Parse value error: {}", err)
            from()
            cause(err)
        }
        SendValue(err: rmp::encode::ValueWriteError){
            description("Send value error")
            display("Send value error: {}", err)
            from()
            cause(err)
        }
        UTF8(err: std::string::FromUtf8Error){
            description("UTF decoding error")
            display("UTF encoding error: {}", err)
            from()
            cause(err)
        }
        IO(err: std::io::Error){
            description("IO error")
            display("IO  error: {}", err)
            from()
            cause(err)
        }
    }
}

