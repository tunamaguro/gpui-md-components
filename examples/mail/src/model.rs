//! メールの状態と操作。UI や GPUI に依存しない。

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Folder {
    Inbox,
    Sent,
    Trash,
    Archive,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Message {
    pub id: u64,
    pub sender: String,
    pub address: String,
    pub subject: String,
    pub body: String,
    pub time: String,
    pub read: bool,
    pub starred: bool,
    pub folder: Folder,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Draft {
    pub to: String,
    pub subject: String,
    pub body: String,
}

#[derive(Clone, Debug)]
pub(crate) struct Undo {
    id: u64,
    folder: Folder,
    index: usize,
}

pub(crate) struct Mail {
    pub messages: Vec<Message>,
    next_id: u64,
}

impl Default for Mail {
    fn default() -> Self {
        let messages = [
            (
                "Avery Vale",
                "avery.vale@example.com",
                "A walk among the stars",
                "The observatory opens on Friday. Would you like to come along?",
                "10:24",
                false,
                true,
            ),
            (
                "Morgan Finch",
                "morgan.finch@example.com",
                "Garden plans",
                "I drew a map of our imaginary community garden. Let me know what you think!",
                "Yesterday",
                false,
                false,
            ),
            (
                "Riley Brook",
                "riley.brook@example.com",
                "The paper boat project",
                "Our paper boat crossed the whole pond. Photos to follow!",
                "Monday",
                true,
                false,
            ),
        ]
        .into_iter()
        .enumerate()
        .map(
            |(index, (sender, address, subject, body, time, read, starred))| Message {
                id: index as u64,
                sender: sender.into(),
                address: address.into(),
                subject: subject.into(),
                body: body.into(),
                time: time.into(),
                read,
                starred,
                folder: Folder::Inbox,
            },
        )
        .collect();
        Self {
            messages,
            next_id: 3,
        }
    }
}

impl Message {
    /// 一覧に出す本文の冒頭。
    pub fn preview(&self) -> String {
        const LENGTH: usize = 80;
        if self.body.chars().count() <= LENGTH {
            return self.body.clone();
        }
        let head: String = self.body.chars().take(LENGTH).collect();
        format!("{}…", head.trim_end())
    }
}

impl Mail {
    pub fn filtered(
        &self,
        folder: Option<Folder>,
        query: &str,
        unread: bool,
        starred: bool,
    ) -> Vec<&Message> {
        let query = query.to_lowercase();
        self.messages
            .iter()
            .filter(|message| {
                folder.is_none_or(|folder| message.folder == folder)
                    && (!unread || !message.read)
                    && (!starred || message.starred)
                    && [
                        message.sender.as_str(),
                        message.subject.as_str(),
                        message.body.as_str(),
                    ]
                    .iter()
                    .any(|text| text.to_lowercase().contains(&query))
            })
            .collect()
    }

    pub fn get(&self, id: u64) -> Option<&Message> {
        self.messages.iter().find(|message| message.id == id)
    }

    pub fn open(&mut self, id: u64) {
        if let Some(message) = self.messages.iter_mut().find(|message| message.id == id) {
            message.read = true;
        }
    }

    pub fn toggle_star(&mut self, id: u64) {
        if let Some(message) = self.messages.iter_mut().find(|message| message.id == id) {
            message.starred = !message.starred;
        }
    }

    pub fn mark_unread(&mut self, id: u64) {
        if let Some(message) = self.messages.iter_mut().find(|message| message.id == id) {
            message.read = false;
        }
    }

    pub fn move_to(&mut self, id: u64, folder: Folder) -> Option<Undo> {
        let index = self.messages.iter().position(|message| message.id == id)?;
        let mut message = self.messages.remove(index);
        let undo = Undo {
            id,
            folder: message.folder,
            index,
        };
        message.folder = folder;
        self.messages.insert(0, message);
        Some(undo)
    }

    pub fn undo(&mut self, undo: &Undo) {
        if let Some(index) = self
            .messages
            .iter()
            .position(|message| message.id == undo.id)
        {
            let mut message = self.messages.remove(index);
            message.folder = undo.folder;
            self.messages
                .insert(undo.index.min(self.messages.len()), message);
        }
    }

    pub fn reply(&self, id: u64) -> Option<Draft> {
        self.get(id).map(|message| Draft {
            to: message.address.clone(),
            subject: if message.subject.starts_with("Re: ") {
                message.subject.clone()
            } else {
                format!("Re: {}", message.subject)
            },
            body: String::new(),
        })
    }

    pub fn send(&mut self, draft: Draft) -> bool {
        if !draft.to.contains('@') {
            return false;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.messages.insert(
            0,
            Message {
                id,
                sender: format!("To: {}", draft.to),
                address: draft.to,
                subject: draft.subject,
                body: draft.body,
                time: "Just now".into(),
                read: true,
                starred: false,
                folder: Folder::Sent,
            },
        );
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_and_filters() {
        let mail = Mail::default();
        assert_eq!(
            mail.filtered(Some(Folder::Inbox), "AVERY", false, false)
                .len(),
            1
        );
        assert_eq!(mail.filtered(None, "garden plans", false, false).len(), 1);
        assert_eq!(mail.filtered(None, "pond", false, false).len(), 1);
        assert_eq!(mail.filtered(None, "", true, true).len(), 1);
        assert!(mail.filtered(None, "missing", false, false).is_empty());
    }

    #[test]
    fn preview_shortens_long_bodies() {
        let mut message = Mail::default().get(1).cloned().unwrap();
        message.body = "短い本文".into();
        assert_eq!(message.preview(), "短い本文");
        message.body = "あ".repeat(100);
        assert_eq!(message.preview().chars().count(), 81);
    }

    #[test]
    fn read_and_star() {
        let mut mail = Mail::default();
        mail.open(0);
        mail.toggle_star(0);
        assert!(
            mail.get(0)
                .is_some_and(|message| message.read && !message.starred)
        );
        mail.mark_unread(0);
        assert!(mail.get(0).is_some_and(|message| !message.read));
    }

    #[test]
    fn delete_and_archive_undo_restore_position() {
        for folder in [Folder::Trash, Folder::Archive] {
            let mut mail = Mail::default();
            let original = mail.messages.clone();
            let undo = mail.move_to(1, folder).expect("message exists");
            assert_eq!(mail.get(1).map(|message| message.folder), Some(folder));
            mail.undo(&undo);
            assert_eq!(mail.messages, original);
        }
    }

    #[test]
    fn send_and_reply() {
        let mut mail = Mail::default();
        let mut draft = mail.reply(0).expect("message exists");
        assert_eq!(draft.to, "avery.vale@example.com");
        assert_eq!(draft.subject, "Re: A walk among the stars");
        draft.body = "Sounds good!".into();
        assert!(mail.send(draft));
        assert_eq!(
            mail.filtered(Some(Folder::Sent), "sounds good", false, false)
                .len(),
            1
        );
        let before = mail.messages.len();
        for to in ["", "invalid"] {
            assert!(!mail.send(Draft {
                to: to.into(),
                subject: String::new(),
                body: String::new(),
            }));
            assert_eq!(mail.messages.len(), before);
        }
    }
}
