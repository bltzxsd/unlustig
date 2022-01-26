use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use discord_rich_presence::{
    activity::{self, Assets, Button, Timestamps},
    new_client, DiscordIpc,
};

use log::debug;
use std::fmt::Debug;

/// A Discord RPC client.
///
/// A handle to RPC connection using a trait-object that implements the [`DiscordIpc`] trait.
///
/// [`DiscordIpc`]: discord_rich_presence::DiscordIpc
#[derive(Debug)]
pub struct Discord {
    inner: Box<dyn DiscordIpc + Send + Sync>,
}

impl Discord {
    /// Creates a new Discord RPC client.
    /// Sets up a rich-presence connection with
    /// * State
    /// * Timestamp
    /// * Image
    /// * Buttons
    ///
    /// # Errors
    ///
    /// Returns an error if the internal client fails to connect.
    pub fn init(application_id: &str) -> Result<Self> {
        let mut client = new_client(application_id)?;
        client.connect()?;

        debug!("client connected");
        let timestamp =
            Timestamps::new().start(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as _);

        let button = Button::new("Join Planet Comedy", "https://github.com/bltzxsd/unlustig");

        let assets = Assets::new()
            .large_image("unlustiglogo")
            .large_text("unlustig-rs");

        let activity = activity::Activity::new()
            .state("Making caption memes")
            .timestamps(timestamp)
            .assets(assets)
            .buttons(vec![button]);

        client.set_activity(activity)?;

        Ok(Self {
            inner: Box::new(client),
        })
    }
}

impl Drop for Discord {
    /// Closes the Discord RPC connection.
    fn drop(&mut self) {
        self.inner.close().expect("could not close discord");
    }
}
