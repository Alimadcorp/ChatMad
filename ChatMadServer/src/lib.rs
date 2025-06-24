use spacetimedb::{reducer, table, Identity, ReducerContext, Table, Timestamp};

#[table(name = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    name: String,
    #[unique]
    uid: String,
    password: String,
    pfp: String,
    status: Option<String>,
    lastActive: Timestamp,
    joined: Timestamp,
    aura: i32,
    online: bool,
    channels: String,
}

#[table(name = message, public)]
pub struct Message {
    sender: Identity,
    sent: Timestamp,
    #[unique]
    id: String,
    channel: String,
    text: String,
    mention: String,
}

#[table(name = channel, public)]
pub struct Channel {
    name: String,
    creator: String,
    #[unique]
    id: String,
    created: Timestamp,
    description: Option<String>,
    dm: bool,
    photo: String,
    totalMessages: i32,
}

#[reducer]
pub fn login(ctx: &ReducerContext, _uid: String, _password: String) -> Result<(), String> {
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        if user.uid == _uid {
            return Ok(())
        } else {
            if user.password == _password {
                ctx.db.user().identity().update(User {
                    identity: ctx.sender,
                    ..user
                });
                return Ok(())
            } else {
                return Err(("Incorrect password".to_string()))
            }
        }
    }
    Ok(())
}

#[reducer]
pub fn new_user(
    ctx: &ReducerContext,
    _usnm: String,
    _password: String,
    _uid: String,
) -> Result<(), String> {
    let image1 = "https://i.ibb.co/KjY8zr9R/1.png";
    let image2 = "https://i.ibb.co/DDhxzxBW/2.png";
    let image3 = "https://i.ibb.co/wF9j64TF/3.png";
    let image4 = "https://i.ibb.co/Q7CHKpVh/4.png";
    let images = vec![image1, image2, image3, image4];
    let to_put: usize = (ctx.timestamp.to_micros_since_unix_epoch() % 4 as i64) as usize;
    let img = &images[to_put];
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        Err(("User already exists".to_string()))
    } else {
        ctx.db.user().insert(User {
            name: _usnm,
            status: None,
            pfp: img.to_string(),
            password: _password,
            uid: _uid,
            lastActive: ctx.timestamp,
            joined: ctx.timestamp,
            aura: 1,
            identity: ctx.sender,
            online: true,
            channels: "#global".to_string(),
        });
        Ok(())
    }
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
pub fn set_pfp(ctx: &ReducerContext, _url: String) -> Result<(), String> {
    let url = _url;
    if let Some(user) = ctx.db.user().identity().find(ctx.sender) {
        ctx.db.user().identity().update(User { pfp: url, ..user });
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
pub fn create_channel(
    ctx: &ReducerContext,
    _name: String,
    desc: String,
    id: String,
    isdm: bool,
) -> Result<(), String> {
    let chname = validate(_name, 32)?;
    let _id = id;
    ctx.db.channel().insert(Channel {
        creator: ctx.identity(),
        name: chname,
        id: _id,
        created: ctx.timestamp,
        description: Some(desc),
        dm: isdm,
    });
    Ok(())
}

#[reducer]
pub fn send_message(
    ctx: &ReducerContext,
    text: String,
    _channel: String,
    _dm: bool,
) -> Result<(), String> {
    let text = validate(text, 1000)?;
    log::info!("{}", text);
    ctx.db.message().insert(Message {
        sender: ctx.sender,
        channel: _channel,
        text,
        sent: ctx.timestamp,
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
            lastActive: ctx.timestamp,
            ..user
        });
    }
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
