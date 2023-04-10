use nng::Socket;

use crate::protocol::{ResponsePayload, ResponseType, ServerResponse};

// function to check if a string is numeric (includes floats)
pub fn is_string_numeric(str: &str) -> bool {
    for c in str.chars() {
        // If a character is not a number or contains only a decimal point, negative sign, or e, the string is not numeric
        if !c.is_numeric()
            && !(['.', '-', 'e'].contains(&c)
                && str.chars().count() > 1
                && !['-', 'e'].contains(&(str.chars().last().unwrap())))
        {
            return false;
        }
    }
    return true;
}

pub fn send_response(
    socket: &Socket,
    response_type: ResponseType,
    response_payload: ResponsePayload,
) -> Result<(), serde_json::Error> {
    let response = ServerResponse::new(response_type, response_payload);

    let json = serde_json::to_string(&response)?;

    socket.send(json.as_bytes()).unwrap();
    Ok(())
}
