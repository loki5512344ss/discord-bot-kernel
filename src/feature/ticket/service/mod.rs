//! Ticket service
pub mod forum;
use serenity::all::{ChannelId, CreateAttachment, CreateMessage, Http, MessageId};
use sqlx::SqlitePool;
use std::sync::OnceLock;
use tracing::{info, warn};
static POOL: OnceLock<SqlitePool> = OnceLock::new();
pub fn set_pool(p: SqlitePool) {
    let _ = POOL.set(p);
}
pub fn pool() -> Option<&'static SqlitePool> {
    POOL.get()
}
#[rustfmt::skip]
async fn ensure_tables(p: &SqlitePool) {
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS ticket_blacklist (guild_id TEXT NOT NULL, user_id TEXT NOT NULL, reason TEXT NOT NULL, PRIMARY KEY (guild_id, user_id))").execute(p).await;
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS ticket_claims (channel_id TEXT PRIMARY KEY, guild_id TEXT NOT NULL, claimed_by TEXT NOT NULL)").execute(p).await;
    let _ = sqlx::query("CREATE TABLE IF NOT EXISTS tickets (channel_id TEXT PRIMARY KEY, guild_id TEXT NOT NULL, opener_id TEXT NOT NULL)").execute(p).await;
}
#[rustfmt::skip]
pub async fn is_blacklisted(p: &SqlitePool, g: u64, u: u64) -> Option<String> {
    ensure_tables(p).await;
    let row: Option<(String,)> = sqlx::query_as("SELECT reason FROM ticket_blacklist WHERE guild_id=? AND user_id=?").bind(g.to_string()).bind(u.to_string()).fetch_optional(p).await.unwrap_or(None);
    row.map(|r| r.0)
}
#[rustfmt::skip]
pub async fn add_blacklist(p: &SqlitePool, g: u64, u: u64, r: &str) -> anyhow::Result<()> {
    ensure_tables(p).await;
    sqlx::query("INSERT OR REPLACE INTO ticket_blacklist(guild_id,user_id,reason) VALUES(?,?,?)").bind(g.to_string()).bind(u.to_string()).bind(r).execute(p).await?;
    info!(guild=g, user=u, "blacklist added");
    Ok(())
}
#[rustfmt::skip]
pub async fn remove_blacklist(p: &SqlitePool, g: u64, u: u64) -> bool {
    ensure_tables(p).await;
    let res = sqlx::query("DELETE FROM ticket_blacklist WHERE guild_id=? AND user_id=?").bind(g.to_string()).bind(u.to_string()).execute(p).await;
    matches!(res, Ok(r) if r.rows_affected()>0)
}
#[rustfmt::skip]
pub async fn record_ticket(p: &SqlitePool, c: u64, g: u64, o: u64) {
    ensure_tables(p).await;
    let _ = sqlx::query("INSERT OR REPLACE INTO tickets(channel_id,guild_id,opener_id) VALUES(?,?,?)").bind(c.to_string()).bind(g.to_string()).bind(o.to_string()).execute(p).await;
}
#[rustfmt::skip]
pub async fn get_opener(p: &SqlitePool, c: u64) -> Option<u64> {
    ensure_tables(p).await;
    let row: Option<(String,)> = sqlx::query_as("SELECT opener_id FROM tickets WHERE channel_id=?").bind(c.to_string()).fetch_optional(p).await.unwrap_or(None);
    row.and_then(|(s,)| s.parse().ok())
}
#[rustfmt::skip]
pub async fn get_claim(p: &SqlitePool, c: u64) -> Option<u64> {
    ensure_tables(p).await;
    let row: Option<(String,)> = sqlx::query_as("SELECT claimed_by FROM ticket_claims WHERE channel_id=?").bind(c.to_string()).fetch_optional(p).await.unwrap_or(None);
    row.and_then(|(s,)| s.parse().ok())
}
#[rustfmt::skip]
pub async fn try_claim(p: &SqlitePool, c: u64, g: u64, cl: u64) -> bool {
    ensure_tables(p).await;
    if get_claim(p,c).await.is_some() { return false; }
    let res = sqlx::query("INSERT OR IGNORE INTO ticket_claims(channel_id,guild_id,claimed_by) VALUES(?,?,?)").bind(c.to_string()).bind(g.to_string()).bind(cl.to_string()).execute(p).await;
    matches!(res, Ok(r) if r.rows_affected()>0)
}
#[rustfmt::skip]
pub async fn cleanup_ticket(p: &SqlitePool, c: u64) {
    let _ = sqlx::query("DELETE FROM tickets WHERE channel_id=?").bind(c.to_string()).execute(p).await;
    let _ = sqlx::query("DELETE FROM ticket_claims WHERE channel_id=?").bind(c.to_string()).execute(p).await;
}
pub async fn add_user_to_ticket(
    http: &Http,
    channel_id: ChannelId,
    user_id: u64,
) -> anyhow::Result<()> {
    let uid = serenity::all::UserId::new(user_id);
    let is_thread = matches!(http.get_channel(channel_id).await, Ok(serenity::all::Channel::Guild(gc)) if gc.thread_metadata.is_some());
    if is_thread {
        channel_id.add_thread_member(http, uid).await?;
    } else {
        let overwrite = serenity::all::PermissionOverwrite {
            allow: serenity::all::Permissions::VIEW_CHANNEL
                | serenity::all::Permissions::SEND_MESSAGES
                | serenity::all::Permissions::READ_MESSAGE_HISTORY,
            deny: serenity::all::Permissions::empty(),
            kind: serenity::all::PermissionOverwriteType::Member(uid),
        };
        channel_id.create_permission(http, overwrite).await?;
    }
    Ok(())
}
pub async fn remove_user_from_ticket(
    http: &Http,
    channel_id: ChannelId,
    user_id: u64,
) -> anyhow::Result<()> {
    let uid = serenity::all::UserId::new(user_id);
    let is_thread = matches!(http.get_channel(channel_id).await, Ok(serenity::all::Channel::Guild(gc)) if gc.thread_metadata.is_some());
    if is_thread {
        channel_id.remove_thread_member(http, uid).await?;
    } else {
        channel_id
            .delete_permission(http, serenity::all::PermissionOverwriteType::Member(uid))
            .await?;
    }
    Ok(())
}
#[rustfmt::skip]
pub async fn fetch_transcript(h: &Http, ch: ChannelId) -> String {
    let mut all=Vec::new();
    let mut before:Option<MessageId>=None;
    loop {
        let mut b=serenity::all::GetMessages::new().limit(100);
        if let Some(id)=before { b=b.before(id); }
        let msgs=match ch.messages(h,b).await { Ok(m)=>m, Err(e)=>{warn!(error=%e,"fetch_transcript failed"); break;}};
        if msgs.is_empty(){break;}
        before=msgs.last().map(|m| m.id);
        let len=msgs.len();
        all.extend(msgs);
        if len<100{break;}
    }
    all.sort_by_key(|m| m.id);
    let mut out=String::new();
    for m in all {
        out.push_str(&format!("{} ({}):\n",m.author.name,m.author.id));
        if !m.content.is_empty(){out.push_str(&m.content); out.push('\n');}
        for e in &m.embeds {
            if let Some(t)=&e.title{out.push_str(&format!("[Embed title: {}]\n",t));}
            if let Some(d)=&e.description{out.push_str(&format!("[Embed: {}]\n",d));}
        }
        out.push_str("---\n");
    }
    if out.is_empty(){out.push_str("No messages in ticket.\n");}
    out
}
#[rustfmt::skip]
pub async fn upload_hastebin(t: &str) -> Option<String> {
    let c=reqwest::Client::new();
    let b=serde_json::json!({"document":t});
    let r=c.post("https://hastebin.com/documents").json(&b).send().await.ok()?;
    if !r.status().is_success(){return None;}
    let v:serde_json::Value=r.json().await.ok()?;
    let k=v.get("key")?.as_str()?;
    Some(format!("https://hastebin.com/{k}"))
}
#[rustfmt::skip]
pub async fn deliver_transcript(h: &Http, ch: ChannelId, closer: u64, tr: String, url: Option<String>) {
    const LOG: u64=1509503717127356446;
    let lc=ChannelId::new(LOG);
    if let Some(u)=url.clone(){
        let _=lc.send_message(h,CreateMessage::new().content(format!("Ticket <#{ch}> closed by <@{closer}>\nTranscript: {u}"))).await;
    } else {
        let a=CreateAttachment::bytes(tr.clone().into_bytes(),"transcript.txt");
        let _=lc.send_message(h,CreateMessage::new().content(format!("Ticket <#{ch}> closed by <@{closer}>")).add_file(a)).await;
    }
    if let Some(p)=pool(){
        if let Some(o)=get_opener(p,ch.get()).await{
            if let Ok(dm)=serenity::all::UserId::new(o).create_dm_channel(h).await{
                if let Some(u)=url{
                    let _=dm.send_message(h,CreateMessage::new().content(format!("Your ticket <#{ch}> was closed. Transcript: {u}"))).await;
                } else {
                    let a=CreateAttachment::bytes(tr.into_bytes(),"transcript.txt");
                    let _=dm.send_message(h,CreateMessage::new().content(format!("Your ticket <#{ch}> was closed.")).add_file(a)).await;
                }
            }
        }
    }
}
