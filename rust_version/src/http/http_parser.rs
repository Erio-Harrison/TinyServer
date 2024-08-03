use std::collections::HashMap;

enum HttpParserState {
    Method,
    Url,
    Protocol,
    HeaderKey,
    HeaderValue,
    Body,
}

struct HttpRequest {
    method: String,
    url: String,
    protocol: String,
    headers: HashMap<String, String>,
    body: String,
}

struct HttpParser {
    state: HttpParserState,
    request: HttpRequest,
    current_header_key: String,
}

impl HttpParser {
    fn new() -> Self {
        HttpParser {
            state: HttpParserState::Method,
            request: HttpRequest {
                method: String::new(),
                url: String::new(),
                protocol: String::new(),
                headers: HashMap::new(),
                body: String::new(),
            },
            current_header_key: String::new(),
        }
    }

    fn reset(&mut self) {
        self.state = HttpParserState::Method;
        self.request = HttpRequest {
            method: String::new(),
            url: String::new(),
            protocol: String::new(),
            headers: HashMap::new(),
            body: String::new(),
        };
        self.current_header_key.clear();
    }

    fn parse(&mut self, data: &str) -> bool {
        for c in data.chars() {
            match self.state {
                HttpParserState::Method => {
                    if c == ' ' {
                        self.state = HttpParserState::Url;
                    } else {
                        self.request.method.push(c);
                    }
                }
                HttpParserState::Url => {
                    if c == ' ' {
                        self.state = HttpParserState::Protocol;
                    } else {
                        self.request.url.push(c);
                    }
                }
                HttpParserState::Protocol => {
                    if c == '\r' {
                        // Ignore
                    } else if c == '\n' {
                        self.state = HttpParserState::HeaderKey;
                    } else {
                        self.request.protocol.push(c);
                    }
                }
                HttpParserState::HeaderKey => {
                    if c == ':' {
                        self.state = HttpParserState::HeaderValue;
                    } else if c == '\r' {
                        // Ignore
                    } else if c == '\n' {
                        self.state = HttpParserState::Body;
                    } else {
                        self.current_header_key.push(c);
                    }
                }
                HttpParserState::HeaderValue => {
                    if c == '\r' {
                        // Ignore
                    } else if c == '\n' {
                        if !self.current_header_key.is_empty() {
                            self.request.headers.insert(self.current_header_key.clone(), String::new());
                            self.current_header_key.clear();
                        }
                        self.state = HttpParserState::HeaderKey;
                    } else {
                        if self.current_header_key.is_empty() {
                            // First character of value, start capturing
                            self.request.headers.insert(self.current_header_key.clone(), c.to_string());
                        } else {
                            if let Some(value) = self.request.headers.get_mut(&self.current_header_key) {
                                value.push(c);
                            }
                        }
                    }
                }
                HttpParserState::Body => {
                    self.request.body.push(c);
                }
            }
        }
        true
    }
}
