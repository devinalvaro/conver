use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    message: &'static str,
}

impl ParseError {
    pub fn method_type_not_found() -> ParseError {
        ParseError {
            message: "method type (CHAT/JOIN/LEAVE) not found",
        }
    }

    pub fn unknown_method_type() -> ParseError {
        ParseError {
            message: "unknown method type",
        }
    }

    pub fn receiver_type_not_found() -> ParseError {
        ParseError {
            message: "receiver type (USER/GROUP) not found",
        }
    }

    pub fn unknown_receiver_type() -> ParseError {
        ParseError {
            message: "unknown receiver type",
        }
    }

    pub fn username_not_found() -> ParseError {
        ParseError {
            message: "username not found",
        }
    }

    pub fn groupname_not_found() -> ParseError {
        ParseError {
            message: "groupname not found",
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}
