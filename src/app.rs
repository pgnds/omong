use mongodb::bson::Document;
use mongodb::sync::Client;
use ratatui::widgets::ListState;
use std::collections::HashSet;

use crate::storage::load_connections;

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
    DeleteConnection { alias: String },
}

pub enum Overlay {
    AddConnectionPopup {
        alias_buffer: String,
        uri_buffer: String,
        focus: PopupField,
        editing_alias: Option<String>,
    },
    FilterPrompt {
        input: String,
    },
    Confirm {
        message: String,
        action: ConfirmAction,
    },
}

pub struct App {
    pub screen: Screen,
    pub overlay: Option<Overlay>,

    pub client: Option<Client>,
    pub error_message: Option<String>,

    pub saved_connections: Vec<(String, String)>,
    pub connections_list_state: ListState,

    pub connection_alias: Option<String>,
    pub database: Option<String>,
    pub collection: Option<String>,

    pub edit_requested: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            screen: Screen::Connections,
            overlay: None,
            client: None,
            error_message: None,
            saved_connections: load_connections(),
            connections_list_state: ListState::default(),
            connection_alias: None,
            database: None,
            collection: None,
            edit_requested: false,
        }
    }
}

#[derive(Default)]
pub struct Filter {
    pub text: String,
    pub doc: Document,
}
