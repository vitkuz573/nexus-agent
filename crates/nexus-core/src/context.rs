use nexus_client::message::Message;

pub struct AgentContext {
    pub messages: Vec<Message>,
    pub working_dir: std::path::PathBuf,
    pub round: usize,
    pub max_rounds: usize,
}

impl AgentContext {
    pub fn new(max_rounds: usize) -> Self {
        Self {
            messages: Vec::new(),
            working_dir: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
            round: 0,
            max_rounds,
        }
    }

    pub fn push_message(&mut self, msg: Message) {
        self.messages.push(msg);
    }

    pub fn can_continue(&self) -> bool {
        self.round < self.max_rounds
    }

    pub fn increment_round(&mut self) {
        self.round += 1;
    }

    pub fn messages_with_system(&self, system_prompt: &str) -> Vec<Message> {
        let mut msgs = vec![Message::system(system_prompt)];
        msgs.extend(self.messages.clone());
        msgs
    }
}
