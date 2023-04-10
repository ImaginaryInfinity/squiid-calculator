# Client-Server Communication Protocol

Squiid is unique in that the part that actually does the calculations (the engine) is completely seperate from the part that the user sees (the frontend). The main thing that makes Squiid Squiid is the engine, and the program was designed with this in mind. While we do provide our own parser and frontend(s), developers do not need to use these and can create their own parsers and frontends with the features that they would like. However, this is not required if you would like a feature to be included in Squiid, and you can simply [submit an issue]() or a [merge request]() <!-- TODO: --> that adds or requests the addition of the feature you would like. 

## Network flow

The network flow is detailed in the diagram below. You can see that the parser is outside of the "required components" cluster, as you have three options for parsing:

1. Build our [Rust-based parser](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid-parser) as a shared object file and use this parser in your frontend, as is detailed in the diagram
2. Build your own parser in the language of your choice and use that as seen in the diagram
3. Build your own parser into your frontend program, completely eliminating the need to use and call a separate shared object file.

Note that a parser is **NOT** required if your frontend only supports entering calculations in reverse polish notation, or postfix notation. More documentation on building a parser can be found <!-- TODO: --> [here](). 

After you choose how you would like to parse user input, you must build your frontend. This is completely up to you except for one constraining factor, the IPC communication library. We use nanomsg for IPC between the frontend and the engine, and at the time of writing this documentation, the available language bindings can be found [here](https://nanomsg.org/documentation.html). Many popular languages are currently supported such as Rust, Python, Java, JavaScript, C, and many others. Make sure that the language you are implementing the frontend in has nanomsg bindings. Other than this constraint, the frontend is entirely up to you. You can make a GUI, a TUI, or any other abomination of a user interface that you can think of.

Once the frontend is set up, it is fairly easy to communicate with the backend server. The following steps will detail how to connect to the server.

<img src="client-server-model.svg">

<!-- graphviz source:
digraph G {
    
    subgraph cluster_required {
        label="Required components"
        style=filled;
		color=lightgrey;
		node [style=filled,color=white];
        engine [shape=box]
        client [shape=diamond]
        client -> engine [style=dashed]
        engine -> client [label="tcp://127.0.0.1:xxxxx" fontsize="8" ]
    }
    
    parser
    client -> parser [dir=both style=dotted]
    
} -->