fn main() {
    // println!("{:?}", squiid_parser::parse("1=@a"));
    let context = zmq::Context::new();
    let socket = context.socket(zmq::REQ).unwrap();
    let mut msg = zmq::Message::new();
    assert!(socket.connect("tcp://localhost:33242").is_ok());
    let rpn_expression = squiid_parser::parse(
        "sqrt(5*(((((1+0.2*(350/661.5)^2)^3.5-1)*(1-(6.875*10^_6)*25500)^_5.2656)+1)^0.286-1))",
    );
    for command_raw in rpn_expression.iter() {
        let command = match command_raw.as_str() {
            "+" => "add",
            "-" => "subtract",
            "*" => "multiply",
            "/" => "divide",
            "^" => "power",
            _ => command_raw,
        };
        println!("{}", command);
        socket.send(command, 0);
        println!("{}", "waiting");
        socket.recv(&mut msg, 0);
        println!("{}", msg.as_str().unwrap());
    }
}
