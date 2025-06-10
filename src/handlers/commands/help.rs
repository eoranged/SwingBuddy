//! Help command handler

use teloxide::{Bot, types::Message, prelude::*};
use crate::utils::errors::Result;

/// Handle /help command
pub async fn handle_help(bot: Bot, msg: Message) -> Result<()> {
    let help_text = "🤖 SwingBuddy Help\n\n\
        /start - Start the bot\n\
        /help - Show this help message\n\
        /events - List upcoming events\n\
        /language - Change language\n\
        /profile - Show your profile\n\n\
        For more information, contact the administrators.";
    
    bot.send_message(msg.chat.id, help_text).await?;
    Ok(())
}