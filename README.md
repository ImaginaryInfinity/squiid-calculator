![Squiid Logo](branding/squiidtext.png)

Squiid is a modular calculator written in Rust. It is currently early in development but is intended to replace our previous calculator, ImaginaryInfinity Calculator.

## Features
- Simple terminal user interface using Ratatui
- Supports both RPN and algebraic input
- Plugin support will be added in the future


## Squiid is split into three components:
#### The Front End (This repository)
Contains the user interface as well as the other components as a subtree. This is all that is needed to compile a working build of Squiid.

#### [The Engine](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid-engine)
Contains the backend of the calculator that actually does the math. This only understands RPN/postfix notation.

#### [The Parser](https://gitlab.com/ImaginaryInfinity/squiid-calculator/squiid-parser)
Contains the library that is used to convert algebraic/infix notation to postfix notation that the backend can evaluate.
