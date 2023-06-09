use std::{collections::HashMap, sync::OnceLock};

use anyhow::anyhow;
use json_db::GlobalDatabase;
use serenity::{
    builder::CreateApplicationCommand,
    model::{
        id::{ChannelId, GuildId},
        prelude::{
            command::CommandOptionType, interaction::application_command::CommandDataOption,
            EmojiId, Message, ReactionType,
        },
        Permissions,
    },
    prelude::Context,
};

pub const CMD_NAME: &str = "react-here";

static GUILDS: GlobalDatabase<HashMap<GuildId, ChannelId>> =
    json_db::Database::const_new("files/guilds.json");

pub async fn react_to(msg: &Message, ctx: &Context) -> anyhow::Result<()> {
    async fn should_react(msg: &Message) -> anyhow::Result<bool> {
        let Some(gid) = msg.guild_id else {
                return Ok(false)
            };
        Ok(GUILDS
            .load()
            .await?
            .get(&gid)
            .is_some_and(|id| id == &msg.channel_id))
    }

    static REACTIONS: OnceLock<[ReactionType; 5]> = OnceLock::new();

    let reactions = || {
        REACTIONS.get_or_init(|| {
            [
                ReactionType::Unicode("❤️".into()),
                ReactionType::Unicode("👍".into()),
                ReactionType::Unicode("👌".into()),
                ReactionType::Unicode("👎".into()),
                ReactionType::Custom {
                    animated: false,
                    id: EmojiId(1114991930099642389),
                    name: Some("dust".into()),
                },
            ]
        })
    };

    if should_react(msg).await? {
        for reaction in reactions() {
            eprintln!("reacting with {reaction}");
            msg.react(ctx, reaction.clone()).await?;
        }
    }
    Ok(())
}

pub async fn run(options: &[CommandDataOption], guild_id: GuildId) -> anyhow::Result<()> {
    let channel_id: ChannelId = options[0]
        .value
        .as_ref()
        .ok_or_else(|| anyhow!("argument not provided"))?
        .as_str()
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(|| anyhow!("argument was not a channelId"))?
        .into();

    GUILDS
        .load()
        .await?
        .entry(guild_id)
        .and_modify(|ch| *ch = channel_id)
        .or_insert(channel_id);

    Ok(())
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name(CMD_NAME)
        .description("set the channel the bot will react to")
        .dm_permission(false)
        .default_member_permissions(Permissions::MANAGE_GUILD)
        .create_option(|option| {
            option
                .name("channel")
                .description("the channel the bot will react to")
                .kind(CommandOptionType::Channel)
                .required(true)
        })
}
