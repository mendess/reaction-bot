mod react_here;

use futures::FutureExt;
use pubsub::ControlFlow;
use serde::{Deserialize, Serialize};
use serenity::{
    model::application::{
        command::Command,
        interaction::{Interaction, InteractionResponseType},
    },
    prelude::GatewayIntents,
    Client,
};

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    token: String,
}

async fn register_integration_commands() {
    use pubsub::events::{InteractionCreate, Message, Ready};

    pubsub::subscribe::<InteractionCreate, _>(|ctx, interaction| {
        async move {
            if let Interaction::ApplicationCommand(command) = interaction {
                let react_here::CMD_NAME = command.data.name.as_str() else {
                    return ControlFlow::CONTINUE;
                };
                let result = match command.guild_id {
                    Some(id) => react_here::run(&command.data.options, id).await,
                    None => Err(anyhow::anyhow!("this command can only be run in a server")),
                };
                if let Err(why) = command
                    .create_interaction_response(ctx, |resp| {
                        resp.kind(InteractionResponseType::ChannelMessageWithSource)
                            .interaction_response_data(|msg| {
                                msg.content(match result {
                                    Ok(_) => "done".into(),
                                    Err(e) => e.to_string(),
                                })
                            })
                    })
                    .await
                {
                    println!("failed to respond to slash command: {why}");
                }
            }
            ControlFlow::CONTINUE
        }
        .boxed()
    })
    .await;

    pubsub::subscribe::<Ready, _>(|ctx, _ready| {
        async move {
            if let Err(why) =
                Command::create_global_application_command(ctx, react_here::register).await
            {
                eprintln!(
                    "failed to register {} command: {why:?}",
                    react_here::CMD_NAME
                );
            };
            ControlFlow::BREAK
        }
        .boxed()
    })
    .await;

    pubsub::subscribe::<Message, _>(|ctx, message| {
        async move {
            if let Err(why) = react_here::react_to(message, ctx).await {
                eprintln!("failed to react to message: {why:?}");
            }
            ControlFlow::CONTINUE
        }
        .boxed()
    })
    .await;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config: Config = config::Config::builder()
        .add_source(config::File::with_name("./files/config"))
        .build()?
        .try_deserialize()?;

    let mut client = Client::builder(
        config.token,
        GatewayIntents::GUILD_INTEGRATIONS
            .union(GatewayIntents::GUILD_MESSAGE_REACTIONS)
            .union(GatewayIntents::GUILD_MESSAGES),
    )
    .event_handler(pubsub::event_handler::Handler::new(None))
    .await?;

    register_integration_commands().await;

    pubsub::subscribe::<pubsub::events::Ready, _>(|_, ready| {
        async move {
            println!(
                "Invite me https://discord.com/oauth2/authorize?client_id={}&scope=bot",
                ready.user.id
            );
            ControlFlow::BREAK
        }
        .boxed()
    })
    .await;

    if let Err(e) = client.start().await {
        anyhow::bail!("client error: {e:?}")
    }
    Ok(())
}
