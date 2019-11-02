# Bizarro
[![Build Status](https://travis-ci.org/theProgrammerJack/Bizarro.svg?branch=master)](https://travis-ci.org/grebneerg/Bizarro)

A discord bot that mimics users using markov chains.

## Features
* Whenever a user is @ mentioned the bot creates a webhook with the same display name and profile picture and sends 1-5 messages generated from a markov chain.
* Mentioning a role acts like mentioning each individual member with that role.
* Chains are continuously added to as more messages are sent.
* Regenerate and save chains on the fly with `|regen` and `|save`.
* Configurable through `Bizarro.toml`. (See [Bizarro.example.toml](Bizarro.example.toml)).

> *Note: I have not tested this bot on very large servers. It may have problems with too many members or messages.*

## Setting up

### Configuring the bot on discord

On the [discord developer portal](https://discordapp.com/developers/applications/), create an application with a bot user and add it to your server.
I just gave mine administrator permissions, but I think it only needs `Manage Webhooks`, `View Channels`, `Send Messages`, `Manage Messages`, `Read Message History`, and `Mention Everyone` (if you wnat it to be able to do that).

### Running the bot

1. Clone this repository.
2. Make sure you have rust and cargo installed. If you don't, they can be installed from [here](https://www.rust-lang.org/tools/install).
3. In the newly cloned directory, create a `Bizarro.toml` file. Advanced settings are detailed in [`Bizzaro.example.toml`](Bizarro.example.toml), but all that is required to be in this file is the following:
```toml
discord_token = "your discord bot token"
chain_storage_dir = "/the/directory/where/chains/should/be/stored"
```
4. Simply execute `cargo run` in the cloned directory to build and run the bot (This will take a while the first time).
    * This may fail, in which case you likely need to install openssl.
5. Once the bot is online, type `|regen` followed by `|save` to generate and save initial markov chains.

## Things I plan to add:

* Excluding messages from chain generation with regex specified in `Bizarro.toml`.
* More configurable control over the bot mentioning people.
* Precompiled binaries for Linux, Mac, and Windows.
* Possibly support for resending old attachments.
