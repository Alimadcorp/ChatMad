use spacetimedb::{reducer, table, Identity, ReducerContext, Table, Timestamp};

#[table(name = user, public)]
pub struct User {
    #[primary_key]
    identity: Identity,
    #[unique]
    name: String,
    pfp: String,
    status: Option<String>,
    lastActive: Timestamp,
    joined: Timestamp,
    aura: i32,
    online: bool,
}

#[table(name = message, public)]
pub struct Message {
    sender: Identity,
    channel: String,
    sent: Timestamp,
    text: String,
}

#[table(name = channel, public)]
pub struct Channel {
    creator: Identity,
    name: String,
    #[unique]
    id: String,
    created: Timestamp,
    description: Option<String>,
    dm: bool,
}

#[table(name = stats, public)]
pub struct Stats {
    users: i32,
    messages: i32,
    channels: i32,
    dms: i32,
    totalAura: i32,
    usersOnline: i32,
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
        ctx.db.user().identity().update(User {
            pfp: url,
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
pub fn create_channel(ctx: &ReducerContext, _name: String, desc: String, id: String, isdm: bool) -> Result<(), String> {
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
            ..user
        });
        //let mut statObj = ctx.db.stats().iter();
    } else {
        let image1 = "https://i.ibb.co/KjY8zr9R/1.png";
        let image2 = "https://i.ibb.co/DDhxzxBW/2.png";
        let image3 = "https://i.ibb.co/wF9j64TF/3.png";
        let image4 = "https://i.ibb.co/Q7CHKpVh/4.png";
        let images= vec![image1.to_string(), image2.to_string(), image3.to_string(), image4.to_string()];
        let to_put: usize = (ctx.timestamp.to_micros_since_unix_epoch() % 4 as i64) as usize;
        let img = images[to_put].clone();
        ctx.db.user().insert(User {
            name: ctx.timestamp.to_string(),
            status: None,
            pfp: img,
            lastActive: ctx.timestamp,
            joined: ctx.timestamp,
            aura: 1,
            identity: ctx.sender,
            online: true,
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