use nu::{Primitive, ReturnValue, ShellError, Spanned, Value};
use serde::{Deserialize, Serialize};
use std::io;

/// A wrapper for proactive notifications to the IDE (eg. diagnostics). These must
/// follow the JSON 2.0 RPC spec

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpc<T> {
    jsonrpc: String,
    pub method: String,
    pub params: Vec<T>,
}
impl<T> JsonRpc<T> {
    pub fn new<U: Into<String>>(method: U, params: Vec<T>) -> Self {
        JsonRpc {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params,
        }
    }
}

fn send_response<T: Serialize>(result: Vec<T>) {
    let response = JsonRpc::new("response", result);
    let response_raw = serde_json::to_string(&response).unwrap();
    println!("{}", response_raw);
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "method")]
#[allow(non_camel_case_types)]
pub enum NuCommand {
    init { params: Vec<Spanned<Value>> },
    filter { params: Value },
    quit,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut inc_by = 1;

    loop {
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let command = serde_json::from_str::<NuCommand>(&input);

                match command {
                    Ok(NuCommand::init { params }) => {
                        for param in params {
                            match param {
                                Spanned {
                                    item: Value::Primitive(Primitive::Int(i)),
                                    ..
                                } => {
                                    inc_by = i;
                                }
                                _ => {
                                    send_response(vec![ReturnValue::Value(Value::Error(
                                        Box::new(ShellError::string("Unrecognized type in params")),
                                    ))]);
                                }
                            }
                        }
                    }
                    Ok(NuCommand::filter { params }) => match params {
                        Value::Primitive(Primitive::Int(i)) => {
                            send_response(vec![ReturnValue::Value(Value::int(i + inc_by))]);
                        }
                        Value::Primitive(Primitive::Bytes(b)) => {
                            send_response(vec![ReturnValue::Value(Value::bytes(
                                b + inc_by as u128,
                            ))]);
                        }
                        _ => {
                            send_response(vec![ReturnValue::Value(Value::Error(Box::new(
                                ShellError::string("Unrecognized type in stream"),
                            )))]);
                        }
                    },
                    Ok(NuCommand::quit) => {
                        break;
                    }
                    Err(_) => {
                        send_response(vec![ReturnValue::Value(Value::Error(Box::new(
                            ShellError::string("Unrecognized type in stream"),
                        )))]);
                    }
                }
            }
            Err(_) => {
                send_response(vec![ReturnValue::Value(Value::Error(Box::new(
                    ShellError::string("Unrecognized type in stream"),
                )))]);
            }
        }
    }

    Ok(())
}