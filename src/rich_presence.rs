use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use discord_rich_presence::{
    activity::{self, Assets, Button, Timestamps},
    new_client, DiscordIpc,
};

use log::debug;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Discord {
    client: Box<dyn DiscordIpc + Send + Sync>,
}

impl Discord {
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

        client.set_activity(activity.clone())?;

        Ok(Self {
            client: Box::new(client),
        })
    }
}

impl Drop for Discord {
    fn drop(&mut self) {
        self.client.close().expect("could not close discord");
    }
}
