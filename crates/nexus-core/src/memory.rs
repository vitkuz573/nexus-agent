use nexus_client::message::Message;

pub struct Memory {
    short_term: Vec<Message>,
    max_short_term: usize,
}

impl Memory {
    pub fn new(max_short_term: usize) -> Self {
        Self {
            short_term: Vec::new(),
            max_short_term,
        }
    }

    pub fn add(&mut self, msg: Message) {
        self.short_term.push(msg);
        if self.short_term.len() > self.max_short_term {
            self.short_term.remove(0);
        }
    }

    pub fn messages(&self) -> &[Message] {
        &self.short_term
    }

    pub fn clear(&mut self) {
        self.short_term.clear();
    }

    pub fn last_n(&self, n: usize) -> &[Message] {
        let start = self.short_term.len().saturating_sub(n);
        &self.short_term[start..]
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new(100)
    }
}
