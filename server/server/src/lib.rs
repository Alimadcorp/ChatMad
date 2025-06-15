use spacetimedb::{reducer, table, Identity, ReducerContext, Table, Timestamp};

#[table(name = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    #[unique]
    name: String,
    status: Option<String>,
    lastActive: Timestamp,
    aura: i32,
    online: bool,
}

#[table(name = message, public)]
pub struct Message {
    sender: Identity,
    channel: String,
    sent: Timestamp,
    text: String,
    dm: bool,
}

#[table(name = channel, public)]
pub struct Channel {
    creator: Identity,
    name: String,
    #[unique]
    id: String,
    created: Timestamp,
    description: Option<String>,
}

#[reducer]
pub fn set_name(ctx: &ReducerContext, _name: String) -> Result<(), String> {
    let aname = validate(_name, 32)?;
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User {
            name: aname,
            ..user
        });
        Ok(())
    } else {
        Err("Cannot set name for unknown user".to_string())
    }
}

#[reducer]
pub fn set_status(ctx: &ReducerContext, _status: String) -> Result<(), String> {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User {
            status: Some(_status),
            ..user
        });
        Ok(())
    } else {
        Err("Cannot set status for unknown user".to_string())
    }
}

#[reducer]
pub fn send_message(
    ctx: &ReducerContext,
    text: String,
    _channel: String,
    _dm: bool,
) -> Result<(), String> {
    let text = validate(text, 32)?;
    log::info!("{}", text);
    ctx.db.message().insert(Message {
        sender: ctx.sender,
        channel: _channel,
        text,
        sent: ctx.timestamp,
        dm: _dm,
    });
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User {
            aura: user.aura + 1,
            ..user
        });
    }
    Ok(())
}

fn validate(input: String, len: i32) -> Result<String, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        Err("String cannot be empty.".to_string())
    } else if trimmed.len() as i32 > len {
        Err("String is too long".to_string())
    } else {
        Ok(trimmed.to_string())
    }
}


#[reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User {
            online: true,
            ..user
        });
    } else {
        ctx.db.user().insert(User {
            name: ctx.timestamp.to_string(),
            status: None,
            lastActive: ctx.timestamp,
            aura: 1,
            identity: ctx.sender,
            online: true,
        });
    }
}

#[reducer]
pub fn create_channel(ctx: &ReducerContext, _name: String, desc: String) -> Result<(), String> {
    let chname = validate(_name, 32)?;
    let _id = ctx.timestamp.to_micros_since_unix_epoch().to_string();
    ctx.db.channel().insert(Channel {
        creator: ctx.identity(),
        name: chname,
        id: _id,
        created: ctx.timestamp,
        description: Some(desc)
    });
    Ok(())
}

#[reducer(client_disconnected)]
pub fn identity_disconnected(ctx: &ReducerContext) {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User {
            online: false,
            lastActive: ctx.timestamp,
            ..user
        });
    } else {
        log::warn!(
            "Disconnect event for unknown user with identity {:?}",
            ctx.sender
        );
    }
}
