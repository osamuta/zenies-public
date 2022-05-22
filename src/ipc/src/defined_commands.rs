use super::*;

impl Command {
    pub fn ping() -> Self {
        Command {
            command: String::from(common_constants::DEFINED_COMMAND_PING),
            messages: None,
        }
    }

    pub fn greet(name: &str, version: &str) -> Self {
        Command {
            command: String::from(common_constants::DEFINED_COMMAND_GREET),
            messages: Some(
                [
                    (String::from("name"), String::from(name)),
                    (String::from("version"), String::from(version)),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
        }
    }

    pub fn gracefull_shutdown() -> Self {
        Command {
            command: String::from(common_constants::DEFINED_COMMAND_SHUTDOWN),
            messages: None,
        }
    }

    pub fn response(response: HashMap<String, String>) -> Self {
        Command {
            command: String::from(common_constants::DEFINED_COMMAND_RESPONSE),
            messages: Some(response),
        }
    }

    pub fn status() -> Self {
        Command {
            command: String::from(common_constants::DEFINED_COMMAND_GET_STATUS),
            messages: None,
        }
    }

    pub fn reload(config_path: &str) -> Self {
        Command {
            command: String::from(common_constants::DEFINED_COMMAND_SHUTDOWN),
            messages: Some(
                [(String::from("path"), String::from(config_path))]
                    .iter()
                    .cloned()
                    .collect(),
            ),
        }
    }
}
