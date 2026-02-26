pub mod auth;
pub mod note;
pub mod ws;
pub mod conversation;
pub mod friendship;
pub mod user;

pub use note::configure as note_configure;
pub use auth::configure as auth_configure;
pub use conversation::configure as conversation_configure;
pub use friendship::configure as friendship_configure;
pub use user::configure as user_configure;

pub use ws::ChatServer;
pub use ws::chat_route;
