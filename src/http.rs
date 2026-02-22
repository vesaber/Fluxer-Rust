use reqwest::{ header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE}, StatusCode, };
use serde::de::DeserializeOwned;
use serde_json::json;
use crate::error::ClientError;
use crate::model::*;

pub struct Http {
    pub client: reqwest::Client,
    pub base_url: String,
    token: String,
}

impl Http {
    pub fn new(token: &str, base_url: String) -> Self {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Bot {}", token);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value).unwrap());
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        Self {
            client: reqwest::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
            base_url,
            token: token.to_string(),
        }
    }

    pub fn get_token(&self) -> &str {
        &self.token
    }

    async fn request_json<T: DeserializeOwned>(
        &self,
        req: reqwest::RequestBuilder,
    ) -> Result<T, ClientError> {
        let resp = req.send().await.map_err(ClientError::Http)?;
        let status = resp.status();
        if status == StatusCode::NO_CONTENT {
            return Err(ClientError::Api("Expected body but got 204".into()));
        }
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api(format!("HTTP {}: {}", status, text)));
        }
        resp.json::<T>().await.map_err(ClientError::Http)
    }

    async fn request_empty(&self, req: reqwest::RequestBuilder) -> Result<(), ClientError> {
        let resp = req.send().await.map_err(ClientError::Http)?;
        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(ClientError::Api(format!("HTTP {}: {}", status, text)));
        }
        Ok(())
    }

    pub async fn get_gateway(&self) -> Result<String, ClientError> {
        let url = format!("{}/gateway/bot", self.base_url);
        let res = self
            .request_json::<GatewayBotResponse>(self.client.get(&url))
            .await?;
        Ok(res.url)
    }

    pub async fn get_me(&self) -> Result<User, ClientError> {
        let url = format!("{}/users/@me", self.base_url);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn get_user(&self, user_id: &str) -> Result<User, ClientError> {
        let url = format!("{}/users/{}", self.base_url, user_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn get_current_user_guilds(&self) -> Result<Vec<Guild>, ClientError> {
        let url = format!("{}/users/@me/guilds", self.base_url);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn get_channel(&self, channel_id: &str) -> Result<Channel, ClientError> {
        let url = format!("{}/channels/{}", self.base_url, channel_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn edit_channel(
        &self,
        channel_id: &str,
        payload: &ChannelCreatePayload,
    ) -> Result<Channel, ClientError> {
        let url = format!("{}/channels/{}", self.base_url, channel_id);
        self.request_json(self.client.patch(&url).json(payload)).await
    }

    pub async fn delete_channel(&self, channel_id: &str) -> Result<(), ClientError> {
        let url = format!("{}/channels/{}", self.base_url, channel_id);
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn trigger_typing(&self, channel_id: &str) -> Result<(), ClientError> {
        let url = format!("{}/channels/{}/typing", self.base_url, channel_id);
        self.request_empty(self.client.post(&url).body("{}")).await
    }

    pub async fn get_messages(
        &self,
        channel_id: &str,
        query: GetMessagesQuery,
    ) -> Result<Vec<Message>, ClientError> {
        let url = format!(
            "{}/channels/{}/messages{}",
            self.base_url,
            channel_id,
            query.to_query_string()
        );
        self.request_json(self.client.get(&url)).await
    }

    pub async fn get_message(
        &self,
        channel_id: &str,
        message_id: &str,
    ) -> Result<Message, ClientError> {
        let url = format!(
            "{}/channels/{}/messages/{}",
            self.base_url, channel_id, message_id
        );
        self.request_json(self.client.get(&url)).await
    }

    pub async fn send_message(
        &self,
        channel_id: &str,
        content: &str,
    ) -> Result<Message, ClientError> {
        let url = format!("{}/channels/{}/messages", self.base_url, channel_id);
        let body = json!({ "content": content });
        self.request_json(self.client.post(&url).json(&body)).await
    }

    pub async fn send_message_advanced(
        &self,
        channel_id: &str,
        payload: &MessageCreatePayload,
    ) -> Result<Message, ClientError> {
        let url = format!("{}/channels/{}/messages", self.base_url, channel_id);
        self.request_json(self.client.post(&url).json(payload)).await
    }

    pub async fn send_embed(
        &self,
        channel_id: &str,
        content: Option<&str>,
        embeds: Vec<Embed>,
    ) -> Result<Message, ClientError> {
        let payload = MessageCreatePayload {
            content: content.map(|s| s.to_string()),
            embeds: Some(embeds),
            ..Default::default()
        };
        self.send_message_advanced(channel_id, &payload).await
    }

    // pub async fn reply_to_message(
    //     &self,
    //     channel_id: &str,
    //     message_id: &str,
    //     content: &str,
    // ) -> Result<Message, ClientError> {
    //     let payload = MessageCreatePayload {
    //         content: Some(content.to_string()),
    //         message_reference: Some(MessageReference {
    //             message_id: message_id.to_string(),
    //             channel_id: None,
    //             guild_id: None,
    //             fail_if_not_exists: Some(true),
    //         }),
    //         ..Default::default()
    //     };
    //     self.send_message_advanced(channel_id, &payload).await
    // }

    pub async fn edit_message(
        &self,
        channel_id: &str,
        message_id: &str,
        content: &str,
    ) -> Result<Message, ClientError> {
        let url = format!(
            "{}/channels/{}/messages/{}",
            self.base_url, channel_id, message_id
        );
        let body = json!({ "content": content });
        self.request_json(self.client.patch(&url).json(&body)).await
    }

    pub async fn edit_message_advanced(
        &self,
        channel_id: &str,
        message_id: &str,
        payload: &MessageCreatePayload,
    ) -> Result<Message, ClientError> {
        let url = format!(
            "{}/channels/{}/messages/{}",
            self.base_url, channel_id, message_id
        );
        self.request_json(self.client.patch(&url).json(payload)).await
    }

    pub async fn delete_message(
        &self,
        channel_id: &str,
        message_id: &str,
    ) -> Result<(), ClientError> {
        let url = format!(
            "{}/channels/{}/messages/{}",
            self.base_url, channel_id, message_id
        );
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn bulk_delete_messages(
        &self,
        channel_id: &str,
        message_ids: Vec<&str>,
    ) -> Result<(), ClientError> {
        let url = format!(
            "{}/channels/{}/messages/bulk-delete",
            self.base_url, channel_id
        );
        let body = json!({ "message_ids": message_ids });
        self.request_empty(self.client.post(&url).json(&body)).await
    }

    pub async fn add_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<(), ClientError> {
        let encoded = urlencoded(emoji);
        let url = format!(
            "{}/channels/{}/messages/{}/reactions/{}/@me",
            self.base_url, channel_id, message_id, encoded
        );
        self.request_empty(self.client.put(&url).body("")).await
    }

    pub async fn remove_own_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<(), ClientError> {
        let encoded = urlencoded(emoji);
        let url = format!(
            "{}/channels/{}/messages/{}/reactions/{}/@me",
            self.base_url, channel_id, message_id, encoded
        );
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn remove_user_reaction(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
        user_id: &str,
    ) -> Result<(), ClientError> {
        let encoded = urlencoded(emoji);
        let url = format!(
            "{}/channels/{}/messages/{}/reactions/{}/{}",
            self.base_url, channel_id, message_id, encoded, user_id
        );
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn get_reactions(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<Vec<User>, ClientError> {
        let encoded = urlencoded(emoji);
        let url = format!(
            "{}/channels/{}/messages/{}/reactions/{}",
            self.base_url, channel_id, message_id, encoded
        );
        self.request_json(self.client.get(&url)).await
    }

    pub async fn clear_reactions(
        &self,
        channel_id: &str,
        message_id: &str,
    ) -> Result<(), ClientError> {
        let url = format!(
            "{}/channels/{}/messages/{}/reactions",
            self.base_url, channel_id, message_id
        );
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn clear_reactions_for_emoji(
        &self,
        channel_id: &str,
        message_id: &str,
        emoji: &str,
    ) -> Result<(), ClientError> {
        let encoded = urlencoded(emoji);
        let url = format!(
            "{}/channels/{}/messages/{}/reactions/{}",
            self.base_url, channel_id, message_id, encoded
        );
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn get_pins(&self, channel_id: &str) -> Result<PinsResponse, ClientError> {
        let url = format!("{}/channels/{}/messages/pins", self.base_url, channel_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn pin_message(
        &self,
        channel_id: &str,
        message_id: &str,
    ) -> Result<(), ClientError> {
        let url = format!(
            "{}/channels/{}/pins/{}",
            self.base_url, channel_id, message_id
        );
        self.request_empty(self.client.put(&url).body("")).await
    }

    pub async fn unpin_message(
        &self,
        channel_id: &str,
        message_id: &str,
    ) -> Result<(), ClientError> {
        let url = format!(
            "{}/channels/{}/pins/{}",
            self.base_url, channel_id, message_id
        );
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn get_invite(&self, invite_code: &str) -> Result<Invite, ClientError> {
        let url = format!(
            "{}/invites/{}?with_counts=true",
            self.base_url, invite_code
        );
        self.request_json(self.client.get(&url)).await
    }

    pub async fn create_invite(
        &self,
        channel_id: &str,
        payload: &CreateInvitePayload,
    ) -> Result<Invite, ClientError> {
        let url = format!("{}/channels/{}/invites", self.base_url, channel_id);
        self.request_json(self.client.post(&url).json(payload)).await
    }

    pub async fn delete_invite(&self, invite_code: &str) -> Result<(), ClientError> {
        let url = format!("{}/invites/{}", self.base_url, invite_code);
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn get_channel_invites(&self, channel_id: &str) -> Result<Vec<Invite>, ClientError> {
        let url = format!("{}/channels/{}/invites", self.base_url, channel_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn get_guild_invites(&self, guild_id: &str) -> Result<Vec<Invite>, ClientError> {
        let url = format!("{}/guilds/{}/invites", self.base_url, guild_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn get_guild(&self, guild_id: &str) -> Result<Guild, ClientError> {
        let url = format!("{}/guilds/{}", self.base_url, guild_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn edit_guild(
        &self,
        guild_id: &str,
        payload: &EditGuildPayload,
    ) -> Result<Guild, ClientError> {
        let url = format!("{}/guilds/{}", self.base_url, guild_id);
        self.request_json(self.client.patch(&url).json(payload)).await
    }

    pub async fn delete_guild(&self, guild_id: &str) -> Result<(), ClientError> {
        let url = format!("{}/guilds/{}", self.base_url, guild_id);
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn get_guild_channels(&self, guild_id: &str) -> Result<Vec<Channel>, ClientError> {
        let url = format!("{}/guilds/{}/channels", self.base_url, guild_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn create_channel(
        &self,
        guild_id: &str,
        payload: &ChannelCreatePayload,
    ) -> Result<Channel, ClientError> {
        let url = format!("{}/guilds/{}/channels", self.base_url, guild_id);
        self.request_json(self.client.post(&url).json(payload)).await
    }

    pub async fn get_guild_member(
        &self,
        guild_id: &str,
        user_id: &str,
    ) -> Result<Member, ClientError> {
        let url = format!("{}/guilds/{}/members/{}", self.base_url, guild_id, user_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn get_guild_members(
        &self,
        guild_id: &str,
        limit: Option<u16>,
        after: Option<&str>,
    ) -> Result<Vec<Member>, ClientError> {
        let mut url = format!("{}/guilds/{}/members?", self.base_url, guild_id);
        if let Some(l) = limit {
            url.push_str(&format!("limit={}&", l.min(1000)));
        }
        if let Some(a) = after {
            url.push_str(&format!("after={}", a));
        }
        self.request_json(self.client.get(&url)).await
    }

    pub async fn kick_member(
        &self,
        guild_id: &str,
        user_id: &str,
    ) -> Result<(), ClientError> {
        let url = format!("{}/guilds/{}/members/{}", self.base_url, guild_id, user_id);
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn edit_member(
        &self,
        guild_id: &str,
        user_id: &str,
        payload: &EditMemberPayload,
    ) -> Result<Member, ClientError> {
        let url = format!("{}/guilds/{}/members/{}", self.base_url, guild_id, user_id);
        self.request_json(self.client.patch(&url).json(payload)).await
    }

    // pub async fn add_member_role(&self, guild_id: &str, user_id: &str, role_id: &str) -> Result<(), ClientError> {
    //     let url = format!("{}/guilds/{}/members/{}/roles/{}", self.base_url, guild_id, user_id, role_id);
    //     self.request_empty(self.client.put(&url).body("")).await
    // }

    // pub async fn remove_member_role(&self, guild_id: &str, user_id: &str, role_id: &str) -> Result<(), ClientError> {
    //     let url = format!("{}/guilds/{}/members/{}/roles/{}", self.base_url, guild_id, user_id, role_id);
    //     self.request_empty(self.client.delete(&url)).await
    // }

    pub async fn ban_member(
        &self,
        guild_id: &str,
        user_id: &str,
        reason: &str,
    ) -> Result<(), ClientError> {
        let url = format!("{}/guilds/{}/bans/{}", self.base_url, guild_id, user_id);
        let body = json!({ "reason": reason });
        self.request_empty(self.client.put(&url).json(&body)).await
    }

    pub async fn unban_member(
        &self,
        guild_id: &str,
        user_id: &str,
    ) -> Result<(), ClientError> {
        let url = format!("{}/guilds/{}/bans/{}", self.base_url, guild_id, user_id);
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn get_guild_bans(&self, guild_id: &str) -> Result<Vec<serde_json::Value>, ClientError> {
        let url = format!("{}/guilds/{}/bans", self.base_url, guild_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn get_guild_roles(&self, guild_id: &str) -> Result<Vec<Role>, ClientError> {
        let url = format!("{}/guilds/{}/roles", self.base_url, guild_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn create_role(
        &self,
        guild_id: &str,
        payload: &CreateRolePayload,
    ) -> Result<Role, ClientError> {
        let url = format!("{}/guilds/{}/roles", self.base_url, guild_id);
        self.request_json(self.client.post(&url).json(payload)).await
    }

    pub async fn edit_role(
        &self,
        guild_id: &str,
        role_id: &str,
        payload: &EditRolePayload,
    ) -> Result<Role, ClientError> {
        let url = format!("{}/guilds/{}/roles/{}", self.base_url, guild_id, role_id);
        self.request_json(self.client.patch(&url).json(payload)).await
    }

    pub async fn delete_role(
        &self,
        guild_id: &str,
        role_id: &str,
    ) -> Result<(), ClientError> {
        let url = format!("{}/guilds/{}/roles/{}", self.base_url, guild_id, role_id);
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn get_guild_emojis(&self, guild_id: &str) -> Result<Vec<Emoji>, ClientError> {
        let url = format!("{}/guilds/{}/emojis", self.base_url, guild_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn delete_guild_emoji(
        &self,
        guild_id: &str,
        emoji_id: &str,
    ) -> Result<(), ClientError> {
        let url = format!("{}/guilds/{}/emojis/{}", self.base_url, guild_id, emoji_id);
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn get_channel_webhooks(
        &self,
        channel_id: &str,
    ) -> Result<Vec<Webhook>, ClientError> {
        let url = format!("{}/channels/{}/webhooks", self.base_url, channel_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn get_guild_webhooks(&self, guild_id: &str) -> Result<Vec<Webhook>, ClientError> {
        let url = format!("{}/guilds/{}/webhooks", self.base_url, guild_id);
        self.request_json(self.client.get(&url)).await
    }

    pub async fn create_webhook(
        &self,
        channel_id: &str,
        name: &str,
        avatar: Option<&str>,
    ) -> Result<Webhook, ClientError> {
        let url = format!("{}/channels/{}/webhooks", self.base_url, channel_id);
        let mut body = json!({ "name": name });
        if let Some(av) = avatar {
            body["avatar"] = serde_json::Value::String(av.to_string());
        }
        self.request_json(self.client.post(&url).json(&body)).await
    }

    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<(), ClientError> {
        let url = format!("{}/webhooks/{}", self.base_url, webhook_id);
        self.request_empty(self.client.delete(&url)).await
    }

    pub async fn execute_webhook(
        &self,
        webhook_id: &str,
        webhook_token: &str,
        payload: &WebhookExecutePayload,
    ) -> Result<Option<Message>, ClientError> {
        let url = format!(
            "{}/webhooks/{}/{}?wait=true",
            self.base_url, webhook_id, webhook_token
        );
        self.request_json(self.client.post(&url).json(payload)).await
    }
}

fn urlencoded(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            let mut buf = [0u8; 4];
            c.encode_utf8(&mut buf);
            let bytes = &buf[..c.len_utf8()];
            bytes
                .iter()
                .map(|b| match b {
                    b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9'
                    | b'-' | b'_' | b'.' | b'~' | b':' => {
                        char::from(*b).to_string()
                    }
                    other => format!("%{:02X}", other),
                })
                .collect::<Vec<_>>()
        })
        .collect()
}