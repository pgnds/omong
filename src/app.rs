use mongodb::bson::{Bson, Document};
use mongodb::sync::Client;
use ratatui::widgets::ListState;
use std::collections::HashSet;
use uuid::Uuid;

use crate::storage::{Connection, load_connections};

pub enum Screen {
    Connections,
    Databases {
        list_state: ListState,
        names: Vec<String>,
    },
    Collections {
        list_state: ListState,
        names: Vec<String>,
    },
    Documents {
        list_state: ListState,
        docs: Vec<Document>,
        page: u64,
        count: u64,
        filter: Filter,
    },
    Detail {
        doc: Document,
        expanded: HashSet<String>,
        list_state: ListState,
        back: Box<Screen>,
    },
}

pub enum PopupField {
    Alias,
    Uri,
}

pub enum ConfirmAction {
    DeleteConnection { id: Uuid },
    DeleteDocument { id: Bson },
    DropCollection { name: String },
    DropDatabase { name: String },
}

pub enum PromptAction {
    NewCollection,
    NewDatabase { collection_name: String },
}

pub enum Overlay {
    AddConnectionPopup {
        alias_buffer: String,
        uri_buffer: String,
        focus: PopupField,
        editing_id: Option<Uuid>,
    },
    FilterPrompt {
        input: String,
    },
    Prompt {
        title: String,
        input: String,
        action: PromptAction,
    },
    Confirm {
        message: String,
        action: ConfirmAction,
        yes_selected: bool,
    },
}

pub enum EditRequest {
    Update { doc: Document },
    Insert,
}

pub enum StatusMessage {
    Error(String),
    Info(String),
}

pub struct App {
    pub screen: Screen,
    pub overlay: Option<Overlay>,

    pub client: Option<Client>,
    pub message: Option<StatusMessage>,

    pub saved_connections: Vec<Connection>,
    pub connections_list_state: ListState,

    pub connection_alias: Option<String>,
    pub database: Option<String>,
    pub collection: Option<String>,

    pub edit_requested: Option<EditRequest>,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Connections,
            overlay: None,
            client: None,
            message: None,
            saved_connections: load_connections(),
            connections_list_state: ListState::default(),
            connection_alias: None,
            database: None,
            collection: None,
            edit_requested: None,
        }
    }
}

#[derive(Default)]
pub struct Filter {
    pub text: String,
    pub doc: Document,
}
